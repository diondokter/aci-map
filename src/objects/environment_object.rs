use crate::{
    air::{AirLeveler, AirPusher, OxygenUser},
    liquids::LiquidLeveler,
    objects::ObjectProperties,
};

#[derive(Debug)]
pub enum EnvironmentObject {
    AirLeveler(AirLeveler<usize>),
    OxygenUser(OxygenUser<usize>),
    AirPusher(AirPusher<usize>),
    LiquidLeveler(LiquidLeveler<usize>),
}

impl From<AirPusher<usize>> for EnvironmentObject {
    fn from(v: AirPusher<usize>) -> Self {
        Self::AirPusher(v)
    }
}

impl From<LiquidLeveler<usize>> for EnvironmentObject {
    fn from(v: LiquidLeveler<usize>) -> Self {
        Self::LiquidLeveler(v)
    }
}

impl From<OxygenUser<usize>> for EnvironmentObject {
    fn from(v: OxygenUser<usize>) -> Self {
        Self::OxygenUser(v)
    }
}

impl From<AirLeveler<usize>> for EnvironmentObject {
    fn from(v: AirLeveler<usize>) -> Self {
        Self::AirLeveler(v)
    }
}

impl ObjectProperties for EnvironmentObject {
    fn air_levelers(&self) -> Vec<AirLeveler<usize>> {
        match self {
            EnvironmentObject::AirLeveler(al) => vec![*al],
            _ => vec![],
        }
    }

    fn oxygen_users(&self) -> Vec<OxygenUser<usize>> {
        match self {
            EnvironmentObject::OxygenUser(ou) => vec![*ou],
            _ => vec![],
        }
    }

    fn liquid_levelers(&self) -> Vec<LiquidLeveler<usize>> {
        match self {
            EnvironmentObject::LiquidLeveler(ll) => vec![*ll],
            _ => vec![],
        }
    }

    fn air_pushers(&self) -> Vec<AirPusher<usize>> {
        match self {
            EnvironmentObject::AirPusher(ap) => vec![*ap],
            _ => vec![],
        }
    }
}
