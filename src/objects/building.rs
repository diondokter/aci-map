use crate::{
    air::{AirLeveler, AirPusher, OxygenUser},
    liquids::LiquidLeveler,
    Facing,
};
use super::{ObjectProperties, ObjectId, characters::Character};

#[derive(Debug)]
pub struct Building {
    pub x: usize,
    pub y: usize,
    pub facing: Facing,
    pub building_type: BuildingType,
}

impl Building {
    pub fn workspots(&self) -> &[WorkSpot] {
        match &self.building_type {
            BuildingType::HandCrankedVentilator { workspots } => &workspots[..],
        }
    }
    
    pub fn workspots_mut(&mut self) -> &mut [WorkSpot] {
        match &mut self.building_type {
            BuildingType::HandCrankedVentilator { workspots } => &mut workspots[..],
        }
    }
}

impl ObjectProperties for Building {
    fn air_levelers(&self) -> Vec<AirLeveler<usize>> {
        self.building_type
            .air_levelers()
            .into_iter()
            .map(|val| val.to_absolute(self.x, self.y))
            .collect()
    }

    fn oxygen_users(&self) -> Vec<OxygenUser<usize>> {
        self.building_type
            .oxygen_users()
            .into_iter()
            .map(|val| val.to_absolute(self.x, self.y))
            .collect()
    }

    fn liquid_levelers(&self) -> Vec<LiquidLeveler<usize>> {
        self.building_type
            .liquid_levelers()
            .into_iter()
            .map(|val| val.to_absolute(self.x, self.y))
            .collect()
    }

    fn air_pushers(&self) -> Vec<AirPusher<usize>> {
        self.building_type
            .air_pushers()
            .into_iter()
            .map(|val| val.to_absolute(self.x, self.y, self.facing))
            .collect()
    }
}

#[derive(Debug)]
pub enum BuildingType {
    HandCrankedVentilator { workspots: [WorkSpot; 2] },
}

impl BuildingType {
    fn air_levelers(&self) -> Vec<AirLeveler<isize>> {
        Vec::new()
    }
    fn oxygen_users(&self) -> Vec<OxygenUser<isize>> {
        Vec::new()
    }
    fn liquid_levelers(&self) -> Vec<LiquidLeveler<isize>> {
        Vec::new()
    }
    fn air_pushers(&self) -> Vec<AirPusher<isize>> {
        Vec::new()
    }
}

#[derive(Debug, Clone)]
pub struct WorkSpot {
    pub x: f32,
    pub y: f32,
    pub occupation: WorkSpotOccupation,
}

#[derive(Debug, Clone)]
pub enum WorkSpotOccupation {
    /// No character is working this spot, nor is one coming to work it
    Open,
    /// No character is working this spot, but one is coming to work it
    Claimed(ObjectId<Character>),
    /// A character is working this spot
    Working(ObjectId<Character>),
}
