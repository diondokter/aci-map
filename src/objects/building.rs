use glam::{Vec2, UVec2};

use super::{characters::Character, ObjectId, ObjectProperties};
use crate::{
    air::{AirLeveler, AirPusher, OxygenUser},
    liquids::LiquidLeveler,
    Facing,
};

#[derive(Debug)]
pub struct Building {
    pub location: UVec2,
    pub facing: Facing,
    pub building_type: BuildingType,
}

impl Building {
    pub(crate) fn workspots(&self) -> Vec<WorkSpot> {
        self.building_type
            .relative_workspots()
            .iter()
            .cloned()
            .map(|mut workspot| {
                let absolute_location = self.facing.rotate_f32_coords(workspot.location);
                workspot.location = absolute_location;
                workspot
            })
            .collect()
    }
}

impl ObjectProperties for Building {
    fn air_levelers(&self) -> Vec<AirLeveler<usize>> {
        self.building_type
            .air_levelers()
            .into_iter()
            .map(|val| val.to_absolute(self.location.x as usize, self.location.y as usize))
            .collect()
    }

    fn oxygen_users(&self) -> Vec<OxygenUser<usize>> {
        self.building_type
            .oxygen_users()
            .into_iter()
            .map(|val| val.to_absolute(self.location.x as usize, self.location.y as usize))
            .collect()
    }

    fn liquid_levelers(&self) -> Vec<LiquidLeveler<usize>> {
        self.building_type
            .liquid_levelers()
            .into_iter()
            .map(|val| val.to_absolute(self.location.x as usize, self.location.y as usize))
            .collect()
    }

    fn air_pushers(&self) -> Vec<AirPusher<usize>> {
        self.building_type
            .air_pushers()
            .into_iter()
            .map(|val| val.to_absolute(self.location.x as usize, self.location.y as usize, self.facing))
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

    pub(crate) fn is_ventilator(&self) -> bool {
        matches!(self, Self::HandCrankedVentilator { .. })
    }

    fn relative_workspots(&self) -> &[WorkSpot] {
        match self {
            BuildingType::HandCrankedVentilator { workspots } => workspots,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WorkSpot {
    pub location: Vec2,
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

impl WorkSpotOccupation {
    /// Returns `true` if the work spot occupation is [`Open`].
    ///
    /// [`Open`]: WorkSpotOccupation::Open
    #[must_use]
    pub(crate) fn is_open(&self) -> bool {
        matches!(self, Self::Open)
    }
}
