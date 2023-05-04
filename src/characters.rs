use crate::{object_id::ObjectProperties, OxygenUser};

#[derive(Debug, Clone)]
pub struct Character {
    pub x: f32,
    pub y: f32,
    pub health: f32,
}

impl ObjectProperties for Character {
    fn oxygen_users(&self) -> Vec<crate::OxygenUser<usize>> {
        vec![OxygenUser {
            x: self.x.floor() as usize,
            y: self.y.floor() as usize,
            change_per_sec: 0.00001,
        }]
    }
}
