use crate::{all_map_objects, Map};

impl<const WIDTH: usize, const HEIGHT: usize> Map<WIDTH, HEIGHT> {
    pub(crate) fn calculate_liquid_diff<L: Liquid>(
        &self,
        delta_time: f32,
    ) -> [[f32; HEIGHT]; WIDTH] {
        let mut liquid_diff_result = [[0.0; HEIGHT]; WIDTH];

        for (x, y) in self.all_tile_coords() {
            let Some(liquids) = self.tiles[x][y].tile_type.get_liquids() else {
                continue;
            };
            let ground_level = self.tiles[x][y].ground_level;
            let liquid_level = liquids.get_level::<L>();
            let total_level = ground_level + liquid_level;

            if liquid_level < L::MINIMAL_HEIGHT_TO_SPREAD {
                continue;
            }

            let neighbour_liquids = self
                // Get all neighbours
                .neighbour_tiles(x, y)
                // Get only the ones that are ground
                .filter_map(|(x, y, tile)| {
                    tile.tile_type
                        .get_liquids()
                        .map(|liquids| (x, y, tile.ground_level, liquids.get_level::<L>()))
                });

            for (nx, ny, neighbour_ground_level, neighbour_liquid_level) in neighbour_liquids {
                let neighbour_total_level = neighbour_ground_level + neighbour_liquid_level;
                if neighbour_total_level >= total_level
                    || neighbour_liquid_level >= LiquidData::MAX_LEVEL
                {
                    continue;
                }

                let height_delta = total_level - neighbour_total_level;
                let applied_height_delta =
                    ((height_delta * L::SPREAD_RATE).sqrt() * delta_time).min(liquid_level / 0.8);

                liquid_diff_result[nx][ny] += applied_height_delta;
                liquid_diff_result[x][y] -= applied_height_delta;
            }
        }

        liquid_diff_result
    }

    pub(crate) fn apply_liquid_diff(
        &mut self,
        water_diff: [[f32; HEIGHT]; WIDTH],
        lava_diff: [[f32; HEIGHT]; WIDTH],
    ) {
        for (x, y) in self.all_tile_coords() {
            let Some(liquids) = self.tiles[x][y].tile_type.get_liquids_mut() else {
                    continue;
                };

            let new_water_level = (liquids.get_level::<Water>() + water_diff[x][y]).max(0.0);
            let new_lava_level = (liquids.get_level::<Lava>() + lava_diff[x][y]).max(0.0);

            *liquids = if new_water_level == 0.0 && new_lava_level == 0.0 {
                LiquidData::None
            } else {
                let difference = new_water_level - new_lava_level;

                if new_water_level > 0.0 && new_lava_level > 0.0 {
                    self.tiles[x][y].ground_level += difference.abs();
                }

                if difference >= 0.0 {
                    LiquidData::Water { level: difference }
                } else {
                    LiquidData::Lava { level: -difference }
                }
            }
        }

        for liquid_leveler in all_map_objects!(self)
            .map(|object| object.liquid_levelers())
            .flatten()
        {
            let Some(liquids) = self.tiles[liquid_leveler.x][liquid_leveler.y].tile_type.get_liquids_mut() else {
                continue;
            };

            *liquids = liquid_leveler.target;
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum LiquidData {
    None,
    Water { level: f32 },
    Lava { level: f32 },
}

impl LiquidData {
    pub(crate) const MAX_LEVEL: f32 = 3.0;

    pub const fn new_default() -> Self {
        Self::None
    }

    pub(crate) fn get_level<L: Liquid>(&self) -> f32 {
        self.get_level_optional::<L>().unwrap_or_default()
    }

    pub(crate) fn get_level_optional<L: Liquid>(&self) -> Option<f32> {
        L::get_level(self)
    }
}

impl Default for LiquidData {
    fn default() -> Self {
        Self::new_default()
    }
}

pub(crate) trait Liquid {
    const SPREAD_RATE: f32;
    const MINIMAL_HEIGHT_TO_SPREAD: f32;

    fn get_level(data: &LiquidData) -> Option<f32>;
}

pub(crate) struct AnyLiquid;
impl Liquid for AnyLiquid {
    const SPREAD_RATE: f32 = 0.0;
    const MINIMAL_HEIGHT_TO_SPREAD: f32 = 0.0;

    fn get_level(data: &LiquidData) -> Option<f32> {
        match data {
            LiquidData::None => None,
            LiquidData::Water { level } => Some(*level),
            LiquidData::Lava { level } => Some(*level),
        }
    }
}

pub(crate) struct Water;
impl Liquid for Water {
    const SPREAD_RATE: f32 = 0.01;
    const MINIMAL_HEIGHT_TO_SPREAD: f32 = 0.01;

    fn get_level(data: &LiquidData) -> Option<f32> {
        match data {
            LiquidData::Water { level } => Some(*level),
            _ => None,
        }
    }
}

pub(crate) struct Lava;
impl Liquid for Lava {
    const SPREAD_RATE: f32 = 0.001;
    const MINIMAL_HEIGHT_TO_SPREAD: f32 = 0.1;

    fn get_level(data: &LiquidData) -> Option<f32> {
        match data {
            LiquidData::Lava { level } => Some(*level),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LiquidLeveler<COORD> {
    pub x: COORD,
    pub y: COORD,
    pub target: LiquidData,
}

impl LiquidLeveler<isize> {
    pub(crate) fn to_absolute(self, base_x: usize, base_y: usize) -> LiquidLeveler<usize> {
        LiquidLeveler {
            x: base_x.wrapping_add_signed(self.x),
            y: base_y.wrapping_add_signed(self.y),
            target: self.target,
        }
    }
}
