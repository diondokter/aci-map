use crate::{object_id::ObjectProperties, AirLeveler, LiquidLeveler, OxygenUser};

#[derive(Debug)]
pub enum EnvironmentObject {
    AirLeveler(AirLeveler),
    OxygenUser(OxygenUser),
    LiquidLeveler(LiquidLeveler),
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
    fn air_levelers(&self) -> Option<Box<dyn Iterator<Item = &AirLeveler> + '_>> {
        match self {
            EnvironmentObject::AirLeveler(al) => Some(Box::new(std::iter::once(al))),
            _ => None,
        }
    }

    fn oxygen_users(&self) -> Option<Box<dyn Iterator<Item = &OxygenUser> + '_>> {
        match self {
            EnvironmentObject::OxygenUser(ou) => Some(Box::new(std::iter::once(ou))),
            _ => None,
        }
    }

    fn liquid_levelers(&self) -> Option<Box<dyn Iterator<Item = &LiquidLeveler> + '_>> {
        match self {
            EnvironmentObject::LiquidLeveler(ll) => Some(Box::new(std::iter::once(ll))),
            _ => None,
        }
    }
}
