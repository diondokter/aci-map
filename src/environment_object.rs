use crate::{object_id::ObjectProperties, AirLeveler, LiquidLeveler, OxygenUser, AirPusher};

#[derive(Debug)]
pub enum EnvironmentObject {
    AirLeveler(AirLeveler),
    OxygenUser(OxygenUser),
    AirPusher(AirPusher),
    LiquidLeveler(LiquidLeveler),
}

impl From<AirPusher> for EnvironmentObject {
    fn from(v: AirPusher) -> Self {
        Self::AirPusher(v)
    }
}

impl From<LiquidLeveler> for EnvironmentObject {
    fn from(v: LiquidLeveler) -> Self {
        Self::LiquidLeveler(v)
    }
}

impl From<OxygenUser> for EnvironmentObject {
    fn from(v: OxygenUser) -> Self {
        Self::OxygenUser(v)
    }
}

impl From<AirLeveler> for EnvironmentObject {
    fn from(v: AirLeveler) -> Self {
        Self::AirLeveler(v)
    }
}

impl ObjectProperties for EnvironmentObject {
    fn air_levelers(&self) -> Vec<&AirLeveler> {
        match self {
            EnvironmentObject::AirLeveler(al) => vec![al],
            _ => vec![],
        }
    }

    fn oxygen_users(&self) -> Vec<&OxygenUser> {
        match self {
            EnvironmentObject::OxygenUser(ou) => vec![ou],
            _ => vec![],
        }
    }

    fn liquid_levelers(&self) -> Vec<&LiquidLeveler> {
        match self {
            EnvironmentObject::LiquidLeveler(ll) => vec![ll],
            _ => vec![],
        }
    }

    fn air_pushers(&self) -> Vec<&AirPusher> {
        match self {
            EnvironmentObject::AirPusher(ap) => vec![ap],
            _ => vec![],
        }
    }
}
