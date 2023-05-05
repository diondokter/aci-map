use crate::{air::AirData, liquids::LiquidData};

#[derive(Clone, Copy, Debug)]
pub struct Tile {
    pub ground_level: f32,
    pub tile_type: TileType,
}

impl Tile {
    pub fn new(ground_level: f32, tile_type: TileType) -> Self {
        Self {
            ground_level,
            tile_type,
        }
    }

    pub const fn new_default() -> Self {
        Self {
            ground_level: 0.0,
            tile_type: TileType::new_default(),
        }
    }
}

impl Default for Tile {
    fn default() -> Self {
        Self::new_default()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TileType {
    Wall,
    Ground { air: AirData, liquids: LiquidData },
}

impl TileType {
    pub const fn new_default() -> Self {
        TileType::Ground {
            air: AirData::new_default(),
            liquids: LiquidData::new_default(),
        }
    }

    pub(crate) fn get_ground(&self) -> Option<(&AirData, &LiquidData)> {
        if let Self::Ground { air, liquids } = self {
            Some((air, liquids))
        } else {
            None
        }
    }

    pub(crate) fn get_ground_mut(&mut self) -> Option<(&mut AirData, &mut LiquidData)> {
        if let Self::Ground { air, liquids } = self {
            Some((air, liquids))
        } else {
            None
        }
    }

    pub(crate) fn get_air(&self) -> Option<&AirData> {
        if let Self::Ground { air, .. } = self {
            Some(air)
        } else {
            None
        }
    }

    pub(crate) fn get_air_mut(&mut self) -> Option<&mut AirData> {
        if let Self::Ground { air, .. } = self {
            Some(air)
        } else {
            None
        }
    }

    pub(crate) fn get_liquids(&self) -> Option<&LiquidData> {
        if let Self::Ground { liquids, .. } = self {
            Some(liquids)
        } else {
            None
        }
    }

    pub(crate) fn get_liquids_mut(&mut self) -> Option<&mut LiquidData> {
        if let Self::Ground { liquids, .. } = self {
            Some(liquids)
        } else {
            None
        }
    }
}

impl Default for TileType {
    fn default() -> Self {
        Self::new_default()
    }
}
