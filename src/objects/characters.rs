use super::{building::Building, ObjectId, ObjectProperties};
use crate::air::OxygenUser;

#[derive(Debug)]
pub struct Character {
    pub x: f32,
    pub y: f32,
    pub health: f32,
    pub goals: [CharacterGoal; 2],
    pub current_goal: CharacterGoal,
    pub current_task: Option<CharacterTask>,
}

impl ObjectProperties for Character {
    fn oxygen_users(&self) -> Vec<OxygenUser<usize>> {
        vec![OxygenUser {
            x: self.x.floor() as usize,
            y: self.y.floor() as usize,
            change_per_sec: 0.00001,
        }]
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CharacterGoal {
    WorkAtVentilation,
    Idle,
}

#[derive(Debug)]
pub enum CharacterTask {
    WorkAtSpot { building: ObjectId<Building> },
    Idle,
}
