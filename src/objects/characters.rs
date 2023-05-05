use super::{building::Building, ObjectId, ObjectProperties};
use crate::air::OxygenUser;

#[derive(Debug)]
pub(crate) struct Character {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) health: f32,
    pub(crate) goals: [CharacterGoal; 2],
    pub(crate) current_goal: CharacterGoal,
    pub(crate) current_task: Option<CharacterTask>,
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
pub(crate) enum CharacterGoal {
    WorkAtVentilation,
    Idle,
}

#[derive(Debug)]
pub(crate) enum CharacterTask {
    WorkAtSpot { building: ObjectId<Building> },
    Idle,
}
