#[derive(Debug)]
pub struct Map<const WIDTH: usize, const HEIGHT: usize> {
    tiles: [[Tile; HEIGHT]; WIDTH],
    air_levelers: Vec<AirLeveler>,
}

impl<const WIDTH: usize, const HEIGHT: usize> Map<WIDTH, HEIGHT> {
    pub fn new(tiles: [[Tile; HEIGHT]; WIDTH], air_levelers: Vec<AirLeveler>) -> Self {
        Self {
            tiles,
            air_levelers,
        }
    }

    pub fn new_default() -> Self {
        Self {
            tiles: [[Tile::default(); HEIGHT]; WIDTH],
            air_levelers: Vec::new(),
        }
    }

    fn all_tile_coords() -> impl Iterator<Item = (usize, usize)> {
        itertools::iproduct!(0..WIDTH, 0..HEIGHT)
    }

    pub fn collect_air_pressure_map(&self) -> [[f32; HEIGHT]; WIDTH] {
        let mut result = [[0.0; HEIGHT]; WIDTH];

        for (x, y) in Self::all_tile_coords() {
            result[x][y] = self.tiles[x][y]
                .tile_type
                .as_ground()
                .map(|ground| ground.air_pressure)
                .unwrap_or(f32::NAN);
        }

        result
    }

    fn neighbour_tile_coords(
        target_tile_x: usize,
        target_tile_y: usize,
    ) -> impl Iterator<Item = (usize, usize)> + Clone {
        [
            (target_tile_x > 0 && target_tile_y > 0)
                .then(|| (target_tile_x - 1, target_tile_y - 1)),
            (target_tile_x > 0).then(|| (target_tile_x - 1, target_tile_y)),
            (target_tile_x > 0 && target_tile_y < WIDTH - 1)
                .then(|| (target_tile_x - 1, target_tile_y + 1)),
            (target_tile_y > 0).then(|| (target_tile_x, target_tile_y - 1)),
            (target_tile_y < WIDTH - 1).then(|| (target_tile_x, target_tile_y + 1)),
            (target_tile_x < HEIGHT - 1 && target_tile_y > 0)
                .then(|| (target_tile_x + 1, target_tile_y - 1)),
            (target_tile_x < HEIGHT - 1).then(|| (target_tile_x + 1, target_tile_y)),
            (target_tile_x < HEIGHT - 1 && target_tile_y < WIDTH - 1)
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
        let air_pressure_diff = self.calculate_air_pressure_diff(delta_time);
        self.apply_air_pressure_diff(air_pressure_diff);
    }

    // TODO: We need more ait pressure diff info. Per tile we need to know how much is given away to each neighbour
    fn calculate_air_pressure_diff(&self, delta_time: f32) -> [[f32; HEIGHT]; WIDTH] {
        let mut result = [[0.0; HEIGHT]; WIDTH];

        const AIR_PRESSURE_SPREAD_RATE: f32 = 2.0;

        // In this model we will 'give away' air pressure.

        for (x, y) in Self::all_tile_coords() {
            let Some(ground) = &self.tiles[x][y].tile_type.as_ground() else {
                    continue;
                };

            let neighbour_ground_with_lower_air_pressure = self
                // Get all neighbours
                .neighbour_tiles(x, y)
                // Get only the ones that are ground
                .filter_map(|(x, y, tile)| {
                    tile.tile_type
                        .as_ground()
                        .map(|ground_data| (x, y, ground_data))
                })
                // Only keep the ones with lower air pressure
                .filter(|data| data.2.air_pressure < ground.air_pressure);

            let mut given_away = 0.0;

            for (nx, ny, neighbour) in neighbour_ground_with_lower_air_pressure {
                let pressure_delta = ground.air_pressure - neighbour.air_pressure;
                let applied_pressure_delta =
                    (pressure_delta * AIR_PRESSURE_SPREAD_RATE * delta_time)
                        .min(ground.air_pressure / 8.0);
                result[nx][ny] += applied_pressure_delta;
                given_away += applied_pressure_delta;
            }

            result[x][y] -= given_away;
        }

        result
    }

    // fn calculate_oxygen_spread(&self, air_pressure_diff: &[[f32; HEIGHT]; WIDTH]) -> [[f32; HEIGHT]; WIDTH] {
    //     // We will use a model where we 'give away' oxygen
    //     for (x, y) in Self::all_tile_coords() {
    //         let current_pressure_diff = air_pressure_diff[x][y];
    //         if current_pressure_diff >= 0.0 {
    //             continue;
    //         }

    //         // We are losing air pressure to all neighbours that have a lower air pressure.
    //         // We have an oxygen% and can calculate how much air and its %
    //         // goes to the neighbours and we can calulate how the oxygen% in the neighbour will change.
    //         // Our own oxygen% stays the same.

    //         let 
    //     }
    // }

    fn apply_air_pressure_diff(&mut self, diff: [[f32; HEIGHT]; WIDTH]) {
        for (x, y) in Self::all_tile_coords() {
            let Some(ground) = self.tiles[x][y].tile_type.as_ground_mut() else {
                    continue;
                };

            ground.air_pressure += diff[x][y];
        }

        for air_leveler in self.air_levelers.iter() {
            let Some(ground) = self.tiles[air_leveler.x][air_leveler.y].tile_type.as_ground_mut() else {
                continue;
            };

            ground.air_pressure = air_leveler.air_pressure;
        }
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Tile {
    ground_level: f32,
    /// In meters above ground level
    liquid_level: f32,
    liquid_type: LiquidType,
    tile_type: TileType,
}

#[derive(Clone, Copy, Debug)]
pub enum TileType {
    Wall,
    Ground(GroundTileData),
}

impl Default for TileType {
    fn default() -> Self {
        TileType::Ground(Default::default())
    }
}

impl TileType {
    pub fn as_ground(&self) -> Option<&GroundTileData> {
        if let Self::Ground(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn as_ground_mut(&mut self) -> Option<&mut GroundTileData> {
        if let Self::Ground(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GroundTileData {
    air_pressure: f32,
    oxygen: f32,
}

impl Default for GroundTileData {
    fn default() -> Self {
        Self { air_pressure: 1.0, oxygen: 0.21 }
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub enum LiquidType {
    #[default]
    Water,
    Lava,
}

#[derive(Debug)]
pub struct AirLeveler {
    x: usize,
    y: usize,
    air_pressure: f32,
    oxygen: f32,
}

#[cfg(test)]
mod tests {
    use std::{fs::File, path::Path};

    use super::*;

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
    }

    fn create_map_gif<const WIDTH: usize, const HEIGHT: usize>(
        path: impl AsRef<Path>,
        map: &mut Map<WIDTH, HEIGHT>,
        iterations: usize,
        frame_every_nth: usize,
        mut data_step: impl FnMut(&Map<WIDTH, HEIGHT>) -> [[f32; HEIGHT]; WIDTH],
        max_value: f32,
        delta_time: f32,
    ) {
        let mut image = File::create(path).unwrap();
        let mut encoder = gif::Encoder::new(&mut image, WIDTH as u16, HEIGHT as u16, &[]).unwrap();
        encoder.set_repeat(gif::Repeat::Infinite).unwrap();

        for i in 0..iterations {
            if i % frame_every_nth == 0 {
                let data = data_step(&map);

                let mut pixels = vec![0; WIDTH * HEIGHT * 3];
                for (i, (x, y)) in Map::<WIDTH, HEIGHT>::all_tile_coords().enumerate() {
                    if data[x][y].is_nan() {
                        continue;
                    }
                    pixels[i * 3 + 0] =
                        255 - (data[x][y] / max_value * 255.0).clamp(0.0, 255.0) as u8;
                    pixels[i * 3 + 1] =
                        ((data[x][y] / max_value).powf(0.1) * 255.0).clamp(0.0, 255.0) as u8;
                    pixels[i * 3 + 2] = 0;
                }
                encoder
                    .write_frame(&gif::Frame::from_rgb(
                        WIDTH as u16,
                        HEIGHT as u16,
                        &mut pixels,
                    ))
                    .unwrap();
            }

            map.simulate(delta_time)
        }
    }

    #[test]
    fn air_pressure() {
        let mut map = Map::<10, 10>::new_default();
        map.air_levelers.push(AirLeveler {
            x: 0,
            y: 0,
            air_pressure: 1.0,
            oxygen: 0.21,
        });
        map.air_levelers.push(AirLeveler {
            x: 9,
            y: 9,
            air_pressure: 0.9,
            oxygen: 0.21,
        });
        map.tiles[0][0] = Tile {
            tile_type: TileType::Ground(GroundTileData {
                air_pressure: 1.0,
                ..Default::default()
            }),
            ..Default::default()
        };
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

        create_map_gif(
            "target/air_pressure.gif",
            &mut map,
            100,
            1,
            |map| map.collect_air_pressure_map(),
            1.0,
            0.05,
        );

        dbg!(map.calculate_air_pressure_diff(0.05));
    }

    #[test]
    fn all_tile_coords() {
        let coords = Map::<1, 2>::all_tile_coords().collect::<Vec<_>>();
        assert_eq!(coords, vec![(0, 0), (0, 1)])
    }
}
