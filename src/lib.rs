#[derive(Debug, Clone)]
pub struct Map<const WIDTH: usize, const HEIGHT: usize> {
    pub tiles: [[Tile; HEIGHT]; WIDTH],
    pub air_levelers: Vec<AirLeveler>,
}

impl<const WIDTH: usize, const HEIGHT: usize> Map<WIDTH, HEIGHT> {
    pub fn new(tiles: [[Tile; HEIGHT]; WIDTH], air_levelers: Vec<AirLeveler>) -> Self {
        Self {
            tiles,
            air_levelers,
        }
    }

    pub const fn new_default() -> Self {
        Self {
            tiles: [[Tile::new_default(); HEIGHT]; WIDTH],
            air_levelers: Vec::new(),
        }
    }

    fn all_tile_coords() -> impl Iterator<Item = (usize, usize)> {
        (0..WIDTH)
            .map(|x| (0..HEIGHT).map(move |y| (x, y)))
            .flatten()
    }

    pub fn collect_air_pressure_map(&self) -> [[f32; HEIGHT]; WIDTH] {
        let mut result = [[0.0; HEIGHT]; WIDTH];

        for (x, y) in Self::all_tile_coords() {
            result[x][y] = self.tiles[x][y]
                .tile_type
                .as_ground()
                .map(|air| air.total())
                .unwrap_or(f32::NAN);
        }

        result
    }

    pub fn collect_oxygen_map(&self) -> [[f32; HEIGHT]; WIDTH] {
        let mut result = [[0.0; HEIGHT]; WIDTH];

        for (x, y) in Self::all_tile_coords() {
            result[x][y] = self.tiles[x][y]
                .tile_type
                .as_ground()
                .map(|air| air.oxygen)
                .unwrap_or(f32::NAN);
        }

        result
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
        let air_diff = self.calculate_air_diff(delta_time);
        self.apply_air_diff(air_diff);
    }

    fn calculate_air_diff(&self, delta_time: f32) -> [[AirDiff; HEIGHT]; WIDTH] {
        let mut air_diff_result = [[AirDiff::default(); HEIGHT]; WIDTH];

        const PRESSURE_SPREAD_RATE: f32 = 2.0;
        const DIFFUSION_SPREAD_RATE: f32 = 2.0;

        // In this model we will 'give away' air pressure and oxygen.

        for (x, y) in Self::all_tile_coords() {
            let Some(air) = &self.tiles[x][y].tile_type.as_ground() else {
                    continue;
                };

            let neighbour_airs = self
                // Get all neighbours
                .neighbour_tiles(x, y)
                // Get only the ones that are ground
                .filter_map(|(x, y, tile)| {
                    tile.tile_type
                        .as_ground()
                        .map(|air| (x, y, air))
                });

            let mut nitrogen_given_away = 0.0;
            let mut oxygen_given_away = 0.0;

            let nitrogen_fraction = air.nitrogen / air.total();
            let oxygen_fraction = air.oxygen / air.total();

            for (nx, ny, neighbour_air) in neighbour_airs {

                // Move air due to pressure difference
                if neighbour_air.total() < air.total() {
                    // It moves due to the total pressure difference, not the difference between each element separately
                    let pressure_delta = air.total() - neighbour_air.total();
                    let applied_pressure_delta = pressure_delta * PRESSURE_SPREAD_RATE * delta_time;

                    let nitrogen_delta = applied_pressure_delta * nitrogen_fraction;
                    let oxygen_delta = applied_pressure_delta * oxygen_fraction;

                    air_diff_result[nx][ny].nitrogen += nitrogen_delta;
                    air_diff_result[nx][ny].oxygen += oxygen_delta;

                    nitrogen_given_away += nitrogen_delta;
                    oxygen_given_away += oxygen_delta;
                }
            }

            air_diff_result[x][y].nitrogen -= nitrogen_given_away;
            air_diff_result[x][y].oxygen -= oxygen_given_away;
        }

        air_diff_result
    }

    fn apply_air_diff(&mut self, air_diff: [[AirDiff; HEIGHT]; WIDTH]) {
        for (x, y) in Self::all_tile_coords() {
            let Some(air) = self.tiles[x][y].tile_type.as_ground_mut() else {
                    continue;
                };

            air.nitrogen += air_diff[x][y].nitrogen;
            air.oxygen += air_diff[x][y].oxygen;
        }

        for air_leveler in self.air_levelers.iter() {
            let Some(air) = self.tiles[air_leveler.x][air_leveler.y].tile_type.as_ground_mut() else {
                continue;
            };

            air.nitrogen = air_leveler.nitrogen;
            air.oxygen = air_leveler.oxygen;
        }
    }
}

impl<const WIDTH: usize, const HEIGHT: usize> Default for Map<WIDTH, HEIGHT> {
    fn default() -> Self {
        Self::new_default()
    }
}

#[derive(Default, Clone, Copy, Debug)]
struct AirDiff {
    nitrogen: f32,
    oxygen: f32,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Tile {
    pub ground_level: f32,
    /// In meters above ground level
    pub liquid_level: f32,
    pub liquid_type: LiquidType,
    pub tile_type: TileType,
}

impl Tile {
    pub fn new(
        ground_level: f32,
        liquid_level: f32,
        liquid_type: LiquidType,
        tile_type: TileType,
    ) -> Self {
        Self {
            ground_level,
            liquid_level,
            liquid_type,
            tile_type,
        }
    }

    pub const fn new_default() -> Self {
        Self {
            ground_level: 0.0,
            liquid_level: 0.0,
            liquid_type: LiquidType::Water,
            tile_type: TileType::Ground(AirData::new_default()),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TileType {
    Wall,
    Ground(AirData),
}

impl Default for TileType {
    fn default() -> Self {
        TileType::Ground(AirData::new_default())
    }
}

impl TileType {
    pub fn as_ground(&self) -> Option<&AirData> {
        if let Self::Ground(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn as_ground_mut(&mut self) -> Option<&mut AirData> {
        if let Self::Ground(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AirData {
    nitrogen: f32,
    oxygen: f32,
}

impl AirData {
    pub const fn new_default() -> Self {
        Self {
            nitrogen: 1.0,
            oxygen: 0.0,
        }
    }

    pub fn total(&self) -> f32 {
        self.nitrogen + self.oxygen
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub enum LiquidType {
    #[default]
    Water,
    Lava,
}

#[derive(Debug, Clone)]
pub struct AirLeveler {
    pub x: usize,
    pub y: usize,
    pub nitrogen: f32,
    pub oxygen: f32,
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

        let neighbours = dbg!(Map::<10, 1>::neighbour_tile_coords(1, 0).collect::<Vec<_>>());

        assert!(neighbours.contains(&(0, 0)));
        assert!(neighbours.contains(&(2, 0)));
        assert_eq!(neighbours.len(), 2);
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
                        ((1.0 - data[x][y] / max_value).powf(0.1) * 255.0).clamp(0.0, 255.0) as u8;
                    pixels[i * 3 + 1] =
                        ((data[x][y] / max_value).powf(0.1) * 255.0).clamp(0.0, 255.0) as u8;
                    pixels[i * 3 + 2] = if data[x][y] > max_value { 255 } else { 0 };
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
        std::thread::Builder::new()
            .name("TestThread".into())
            .stack_size(16 * 1024 * 1024)
            .spawn(|| {
                let mut map = Map::<10, 10>::new_default();
                map.air_levelers.push(AirLeveler {
                    x: 0,
                    y: 0,
                    nitrogen: 0.79,
                    oxygen: 0.00,
                });
                map.air_levelers.push(AirLeveler {
                    x: 9,
                    y: 9,
                    nitrogen: 0.79,
                    oxygen: 0.21,
                });
                map.air_levelers.push(AirLeveler {
                    x: 2,
                    y: 2,
                    nitrogen:0.79,
                    oxygen: 0.10,
                });
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
                for i in 3..6 {
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
                    "target/total_air_pressure.gif",
                    &mut map.clone(),
                    10000,
                    100,
                    |map| map.collect_air_pressure_map(),
                    1.0,
                    0.05,
                );
                create_map_gif(
                    "target/oxygen.gif",
                    &mut map.clone(),
                    10000,
                    100,
                    |map| map.collect_oxygen_map(),
                    0.21,
                    0.05,
                );
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn all_tile_coords() {
        let coords = Map::<1, 2>::all_tile_coords().collect::<Vec<_>>();
        assert_eq!(coords, vec![(0, 0), (0, 1)])
    }
}
