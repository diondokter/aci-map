use colored::Colorize;

#[derive(Debug)]
pub struct Map<const WIDTH: usize, const HEIGHT: usize> {
    tiles: [[Tile; HEIGHT]; WIDTH],
}

impl<const WIDTH: usize, const HEIGHT: usize> Map<WIDTH, HEIGHT> {
    pub fn new(tiles: [[Tile; HEIGHT]; WIDTH]) -> Self {
        Self { tiles }
    }

    pub fn new_default() -> Self {
        Self {
            tiles: [[Tile::default(); HEIGHT]; WIDTH],
        }
    }

    pub fn print_air_pressure(&self) {
        let mut highest_value: f32 = f32::NEG_INFINITY;
        let mut lowest_value: f32 = f32::INFINITY;
        
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                let air_pressure = self.tiles[x][y]
                    .tile_type
                    .as_ground()
                    .map(|data| data.air_pressure)
                    .unwrap_or(0.0);
                highest_value = highest_value.max(air_pressure);
                lowest_value = lowest_value.min(air_pressure);
            }
        }

        println!("Max: {highest_value}");
        println!("Min: {lowest_value}");
        print!("|");
        for _ in 0..WIDTH {
            print!("--");
        }
        println!("|");
        for x in 0..WIDTH {
            print!("|");
            for y in 0..HEIGHT {
                let air_pressure = self.tiles[x][y]
                    .tile_type
                    .as_ground()
                    .map(|data| data.air_pressure)
                    .unwrap_or(0.0);

                let percent = (air_pressure - lowest_value) / (highest_value - lowest_value);

                let color_value = (percent * 255.0) as u8;

                print!(
                    "{}",
                    "▄ ".color(colored::Color::TrueColor {
                        r: u8::MAX - color_value,
                        g: color_value,
                        b: 0
                    })
                );
            }
            println!("|");
        }
        print!("|");
        for _ in 0..WIDTH {
            print!("--");
        }
        println!("|");
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

    pub fn calculate_air_pressure_diff(&self, delta_time: f32) -> [[f32; HEIGHT]; WIDTH] {
        let mut result = [[0.0; HEIGHT]; WIDTH];

        const AIR_PRESSURE_SPREAD_RATE: f32 = 1.0;

        // In this model we will 'give away' air pressure.

        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                let Some(tile) = &self.tiles[x][y].tile_type.as_ground() else {
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
                    .filter(|data| data.2.air_pressure < tile.air_pressure);

                let mut given_away = 0.0;

                for (nx, ny, neighbour) in neighbour_ground_with_lower_air_pressure {
                    let pressure_delta = tile.air_pressure - neighbour.air_pressure;
                    let applied_pressure_delta =
                        (pressure_delta * AIR_PRESSURE_SPREAD_RATE * delta_time)
                            .min(tile.air_pressure / 8.0);
                    result[nx][ny] += applied_pressure_delta;
                    given_away += applied_pressure_delta;
                }

                result[x][y] -= given_away;
            }
        }

        result
    }

    pub fn apply_air_pressure_diff(&mut self, diff: [[f32; HEIGHT]; WIDTH]) {
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                let Some(tile) = self.tiles[x][y].tile_type.as_ground_mut() else {
                    continue;
                };

                tile.air_pressure += diff[x][y];
            }
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

#[derive(Default, Clone, Copy, Debug)]
pub struct GroundTileData {
    air_pressure: f32,
    oxygen: f32,
}

#[derive(Default, Clone, Copy, Debug)]
pub enum LiquidType {
    #[default]
    Water,
    Lava,
}

#[cfg(test)]
mod tests {
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

    #[test]
    fn air_pressure() {
        let mut map = Map::<10, 10>::new_default();
        map.tiles[0][0] = Tile {
            tile_type: TileType::Ground(GroundTileData {
                air_pressure: 1.0,
                ..Default::default()
            }),
            ..Default::default()
        };

        for _ in 0..100000 {
            let diff = map.calculate_air_pressure_diff(0.01);
            map.apply_air_pressure_diff(diff);
        }

        map.print_air_pressure();
    }
}
