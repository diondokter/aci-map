use crate::{
    object_id::ObjectProperties, AirLeveler, AirPusher, Facing, LiquidLeveler, OxygenUser,
};

#[derive(Debug)]
pub struct Building {
    pub x: usize,
    pub y: usize,
    pub facing: Facing,
    pub building_type: BuildingType,
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
    Fan {},
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
