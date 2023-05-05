#![feature(const_type_id)]

use air::AirDiff;
use liquids::{Water, Lava};
use objects::characters::Character;
use objects::environment_object::EnvironmentObject;
use objects::{building::Building, Object, ObjectProperties};
use tiles::Tile;

pub mod air;
mod facing;
pub mod liquids;
pub mod objects;
pub mod tiles;

pub use facing::Facing;

#[derive(Debug)]
pub struct Map<const WIDTH: usize, const HEIGHT: usize> {
    pub tiles: [[Tile; HEIGHT]; WIDTH],
    next_object_id: usize,
    current_time: f64,

    // These object arrays must be in order of object ID
    environment_objects: Vec<Object<EnvironmentObject>>,
    buildings: Vec<Object<Building>>,
    characters: Vec<Object<Character>>,
}

impl<const WIDTH: usize, const HEIGHT: usize> Map<WIDTH, HEIGHT> {
    pub const fn new_default() -> Self {
        Self {
            tiles: [[Tile::new_default(); HEIGHT]; WIDTH],
            next_object_id: 0,
            current_time: 0.0,
            environment_objects: Vec::new(),
            buildings: Vec::new(),
            characters: Vec::new(),
        }
    }

    pub fn all_tile_coords(&self) -> impl Iterator<Item = (usize, usize)> {
        (0..WIDTH)
            .map(|x| (0..HEIGHT).map(move |y| (x, y)))
            .flatten()
    }

    fn neighbour_tile_coords(
        target_tile_x: usize,
        target_tile_y: usize,
    ) -> impl Iterator<Item = (usize, usize)> + Clone {
        let has_neg_x_neighbour = target_tile_x > 0;
        let has_neg_y_neighbour = target_tile_y > 0;
        let has_pos_x_neighbour = target_tile_x < WIDTH - 1;
        let has_pos_y_neighbour = target_tile_y < HEIGHT - 1;

        [
            (has_neg_x_neighbour && has_neg_y_neighbour)
                .then(|| (target_tile_x - 1, target_tile_y - 1)),
            (has_neg_x_neighbour).then(|| (target_tile_x - 1, target_tile_y)),
            (has_neg_x_neighbour && has_pos_y_neighbour)
                .then(|| (target_tile_x - 1, target_tile_y + 1)),
            (has_neg_y_neighbour).then(|| (target_tile_x, target_tile_y - 1)),
            (has_pos_y_neighbour).then(|| (target_tile_x, target_tile_y + 1)),
            (has_pos_x_neighbour && has_neg_y_neighbour)
                .then(|| (target_tile_x + 1, target_tile_y - 1)),
            (has_pos_x_neighbour).then(|| (target_tile_x + 1, target_tile_y)),
            (has_pos_x_neighbour && has_pos_y_neighbour)
                .then(|| (target_tile_x + 1, target_tile_y + 1)),
        ]
        .into_iter()
        .filter_map(|t| t)
    }

    fn neighbour_tiles(
        &self,
        target_tile_x: usize,
        target_tile_y: usize,
    ) -> impl Iterator<Item = (usize, usize, &Tile)> + Clone {
        Self::neighbour_tile_coords(target_tile_x, target_tile_y)
            .map(|(x, y)| (x, y, &self.tiles[x][y]))
    }

    pub fn simulate(&mut self, delta_time: f32) {
        let mut air_diff = [[AirDiff::default(); HEIGHT]; WIDTH];
        let mut water_diff = [[0.0; HEIGHT]; WIDTH];
        let mut lava_diff = [[0.0; HEIGHT]; WIDTH];

        rayon::scope(|s| {
            s.spawn(|_| air_diff = self.calculate_air_diff(delta_time));
            s.spawn(|_| water_diff = self.calculate_liquid_diff::<Water>(delta_time));
            s.spawn(|_| lava_diff = self.calculate_liquid_diff::<Lava>(delta_time));
        });

        self.apply_air_diff(air_diff, delta_time);
        self.apply_liquid_diff(water_diff, lava_diff);

        self.current_time += delta_time as f64;
    }
}

impl<const WIDTH: usize, const HEIGHT: usize> Default for Map<WIDTH, HEIGHT> {
    fn default() -> Self {
        Self::new_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{liquids::{AnyLiquid, Liquid, LiquidLeveler, LiquidData, Water, Lava}, air::{AirLeveler, OxygenUser, AirPusher}, tiles::TileType};
    use std::{fs::File, path::PathBuf};

    impl<const WIDTH: usize, const HEIGHT: usize> Map<WIDTH, HEIGHT> {
        fn collect_air_pressure_map(&self) -> [[f32; HEIGHT]; WIDTH] {
            let mut result = [[0.0; HEIGHT]; WIDTH];

            for (x, y) in self.all_tile_coords() {
                result[x][y] = self.tiles[x][y]
                    .tile_type
                    .get_ground()
                    .map(|(air, liquids)| air.air_pressure(liquids.get_level::<AnyLiquid>()))
                    .unwrap_or(f32::NAN);
            }

            result
        }

        fn collect_oxygen_map(&self) -> [[f32; HEIGHT]; WIDTH] {
            let mut result = [[0.0; HEIGHT]; WIDTH];

            for (x, y) in self.all_tile_coords() {
                result[x][y] = self.tiles[x][y]
                    .tile_type
                    .get_air()
                    .map(|air| air.oxygen_fraction())
                    .unwrap_or(f32::NAN);
            }

            result
        }

        fn collect_fumes_map(&self) -> [[f32; HEIGHT]; WIDTH] {
            let mut result = [[0.0; HEIGHT]; WIDTH];

            for (x, y) in self.all_tile_coords() {
                result[x][y] = self.tiles[x][y]
                    .tile_type
                    .get_air()
                    .map(|air| air.fumes_fraction())
                    .unwrap_or(f32::NAN);
            }

            result
        }

        fn collect_liquids_map<L: Liquid>(&self) -> [[f32; HEIGHT]; WIDTH] {
            let mut result = [[0.0; HEIGHT]; WIDTH];

            for (x, y) in self.all_tile_coords() {
                result[x][y] = self.tiles[x][y]
                    .tile_type
                    .get_liquids()
                    .map(|liquids| liquids.get_level::<L>())
                    .unwrap_or(f32::NAN);
            }

            result
        }

        fn collect_surface_level_map(&self) -> [[f32; HEIGHT]; WIDTH] {
            let mut result = [[0.0; HEIGHT]; WIDTH];

            for (x, y) in self.all_tile_coords() {
                result[x][y] = self.tiles[x][y]
                    .tile_type
                    .get_liquids()
                    .map(|liquids| self.tiles[x][y].ground_level + liquids.get_level::<AnyLiquid>())
                    .unwrap_or(self.tiles[x][y].ground_level);
            }

            result
        }

        fn collect_ground_level_map(&self) -> [[f32; HEIGHT]; WIDTH] {
            let mut result = [[0.0; HEIGHT]; WIDTH];

            for (x, y) in self.all_tile_coords() {
                result[x][y] = self.tiles[x][y].ground_level;
            }

            result
        }
    }

    #[test]
    fn neighbours() {
        let neighbours = Map::<10, 10>::neighbour_tile_coords(0, 0).collect::<Vec<_>>();

        assert!(neighbours.contains(&(0, 1)));
        assert!(neighbours.contains(&(1, 1)));
        assert!(neighbours.contains(&(1, 0)));
        assert_eq!(neighbours.len(), 3);

        let neighbours = Map::<10, 10>::neighbour_tile_coords(9, 9).collect::<Vec<_>>();

        assert!(neighbours.contains(&(8, 9)));
        assert!(neighbours.contains(&(8, 8)));
        assert!(neighbours.contains(&(9, 8)));
        assert_eq!(neighbours.len(), 3);

        let neighbours = Map::<10, 10>::neighbour_tile_coords(5, 5).collect::<Vec<_>>();

        assert!(neighbours.contains(&(4, 4)));
        assert!(neighbours.contains(&(4, 5)));
        assert!(neighbours.contains(&(4, 6)));
        assert!(neighbours.contains(&(5, 4)));
        assert!(neighbours.contains(&(5, 6)));
        assert!(neighbours.contains(&(6, 4)));
        assert!(neighbours.contains(&(6, 5)));
        assert!(neighbours.contains(&(6, 6)));
        assert_eq!(neighbours.len(), 8);

        let neighbours = dbg!(Map::<10, 1>::neighbour_tile_coords(1, 0).collect::<Vec<_>>());

        assert!(neighbours.contains(&(0, 0)));
        assert!(neighbours.contains(&(2, 0)));
        assert_eq!(neighbours.len(), 2);
    }

    fn all_tile_coords_gif<const WIDTH: usize, const HEIGHT: usize>(
    ) -> impl Iterator<Item = (usize, usize)> {
        (0..HEIGHT)
            .map(|y| (0..WIDTH).map(move |x| (x, y)))
            .flatten()
    }

    struct GifSetup<const WIDTH: usize, const HEIGHT: usize> {
        path: PathBuf,
        max_value: f32,
        min_value: f32,
        gradient: colorgrad::Gradient,
        data_getter: fn(&Map<WIDTH, HEIGHT>) -> [[f32; HEIGHT]; WIDTH],
    }

    fn create_map_gif<const WIDTH: usize, const HEIGHT: usize>(
        map: &mut Map<WIDTH, HEIGHT>,
        iterations: usize,
        frame_every_nth: usize,
        delta_time: f32,
        gif_setups: &[GifSetup<WIDTH, HEIGHT>],
    ) {
        let mut encoders = gif_setups
            .iter()
            .map(|setup| {
                let image = File::create(&setup.path).unwrap();
                let mut encoder =
                    gif::Encoder::new(image, WIDTH as u16, HEIGHT as u16, &[]).unwrap();
                encoder.set_repeat(gif::Repeat::Infinite).unwrap();
                encoder
            })
            .collect::<Vec<_>>();

        for i in 0..iterations {
            if i % frame_every_nth == 0 {
                for (setup, encoder) in gif_setups.iter().zip(encoders.iter_mut()) {
                    let data = (setup.data_getter)(&map);

                    let mut pixels = vec![128; WIDTH * HEIGHT * 3];
                    for (i, (x, y)) in all_tile_coords_gif::<WIDTH, HEIGHT>().enumerate() {
                        if data[x][y].is_nan() {
                            continue;
                        }

                        if data[x][y] < setup.min_value {
                            pixels[i * 3 + 0] = 0;
                            pixels[i * 3 + 1] = 0;
                            pixels[i * 3 + 2] = 0;
                        } else if data[x][y] > setup.max_value {
                            pixels[i * 3 + 0] = 255;
                            pixels[i * 3 + 1] = 255;
                            pixels[i * 3 + 2] = 255;
                        } else {
                            let fraction = (data[x][y] - setup.min_value)
                                / (setup.max_value - setup.min_value);
                            let [r, g, b, _] = setup.gradient.at(fraction as f64).to_rgba8();

                            pixels[i * 3 + 0] = r;
                            pixels[i * 3 + 1] = g;
                            pixels[i * 3 + 2] = b;
                        }
                    }
                    encoder
                        .write_frame(&gif::Frame::from_rgb(
                            WIDTH as u16,
                            HEIGHT as u16,
                            &mut pixels,
                        ))
                        .unwrap();
                }
            }

            map.simulate(delta_time)
        }
    }

    #[test]
    fn air_pressure() {
        std::thread::Builder::new()
            .name("TestThread".into())
            .stack_size(16 * 1024 * 1024)
            .spawn(|| {
                let mut map = Map::<20, 10>::new_default();
                map.push_object::<EnvironmentObject>(AirLeveler {
                    x: 0,
                    y: 9,
                    nitrogen: 0.79 / 2.0,
                    oxygen: 0.21 / 2.0,
                    fumes: 0.0,
                });
                map.push_object::<EnvironmentObject>(AirLeveler {
                    x: 9,
                    y: 0,
                    nitrogen: 0.79,
                    oxygen: 0.21,
                    fumes: 0.0,
                });
                map.push_object::<EnvironmentObject>(OxygenUser {
                    x: 5,
                    y: 5,
                    change_per_sec: 0.0001,
                });
                map.push_object::<EnvironmentObject>(OxygenUser {
                    x: 18,
                    y: 2,
                    change_per_sec: 0.0001,
                });

                map.push_object::<EnvironmentObject>(LiquidLeveler {
                    x: 19,
                    y: 0,
                    target: LiquidData::Water { level: 1.0 },
                });
                map.push_object::<EnvironmentObject>(LiquidLeveler {
                    x: 19,
                    y: 9,
                    target: LiquidData::Lava { level: 1.1 },
                });
                map.push_object::<EnvironmentObject>(AirPusher {
                    x: 18,
                    y: 4,
                    direction: Facing::South,
                    amount: 2.0,
                });
                map.push_object::<EnvironmentObject>(AirPusher {
                    x: 16,
                    y: 8,
                    direction: Facing::West,
                    amount: 2.0,
                });
                map.push_object::<EnvironmentObject>(AirPusher {
                    x: 10,
                    y: 8,
                    direction: Facing::West,
                    amount: 2.0,
                });

                for (x, y) in map.all_tile_coords().filter(|(x, _)| *x >= 10) {
                    map.tiles[x][y].ground_level = -1.1;
                }

                for i in 1..8 {
                    map.tiles[1][i] = Tile {
                        tile_type: TileType::Wall,
                        ..Default::default()
                    };
                }
                for i in 1..8 {
                    map.tiles[i][1] = Tile {
                        tile_type: TileType::Wall,
                        ..Default::default()
                    };
                }
                for i in 3..8 {
                    map.tiles[3][i] = Tile {
                        tile_type: TileType::Wall,
                        ..Default::default()
                    };
                }
                for i in 3..8 {
                    map.tiles[i][3] = Tile {
                        tile_type: TileType::Wall,
                        ..Default::default()
                    };
                }
                for i in 3..7 {
                    map.tiles[7][i] = Tile {
                        tile_type: TileType::Wall,
                        ..Default::default()
                    };
                }
                for i in 3..6 {
                    map.tiles[i][7] = Tile {
                        tile_type: TileType::Wall,
                        ..Default::default()
                    };
                }

                create_map_gif(
                    &mut map,
                    100000,
                    100,
                    0.05,
                    &[
                        GifSetup {
                            path: "target/total_air_pressure.gif".into(),
                            max_value: 1.02,
                            min_value: 0.00,
                            gradient: colorgrad::viridis(),
                            data_getter: |map| map.collect_air_pressure_map(),
                        },
                        GifSetup {
                            path: "target/oxygen.gif".into(),
                            max_value: 0.21,
                            min_value: 0.10,
                            gradient: colorgrad::viridis(),
                            data_getter: |map| map.collect_oxygen_map(),
                        },
                        GifSetup {
                            path: "target/fumes.gif".into(),
                            max_value: 0.005,
                            min_value: 0.00,
                            gradient: colorgrad::viridis(),
                            data_getter: |map| map.collect_fumes_map(),
                        },
                        GifSetup {
                            path: "target/water.gif".into(),
                            max_value: 3.00,
                            min_value: 0.00,
                            gradient: colorgrad::viridis(),
                            data_getter: |map| map.collect_liquids_map::<Water>(),
                        },
                        GifSetup {
                            path: "target/lava.gif".into(),
                            max_value: 3.00,
                            min_value: 0.00,
                            gradient: colorgrad::viridis(),
                            data_getter: |map| map.collect_liquids_map::<Lava>(),
                        },
                        GifSetup {
                            path: "target/surface.gif".into(),
                            max_value: 1.00,
                            min_value: -1.1,
                            gradient: colorgrad::viridis(),
                            data_getter: |map| map.collect_surface_level_map(),
                        },
                        GifSetup {
                            path: "target/ground_level.gif".into(),
                            max_value: 1.00,
                            min_value: -1.1,
                            gradient: colorgrad::viridis(),
                            data_getter: |map| map.collect_ground_level_map(),
                        },
                    ],
                );
            })
            .unwrap()
            .join()
            .unwrap();
    }
}
