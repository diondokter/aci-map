use glam::Vec2;
use ordered_float::OrderedFloat;

use super::{building::Building, ObjectId, ObjectProperties};
use crate::{air::OxygenUser, Map};

#[derive(Debug)]
pub struct Character {
    pub location: Vec2,
    pub health: f32,
    pub(crate) work_goals_order: Vec<WorkGoal>,
    pub(crate) current_goal: CharacterGoal,
    pub(crate) current_task: Option<CharacterTask>,
}

impl Character {
    pub fn new(location: Vec2, health: f32, work_goals_order: Vec<WorkGoal>) -> Self {
        Self {
            location,
            health,
            work_goals_order,
            current_goal: CharacterGoal::Idle,
            current_task: None,
        }
    }
}

impl ObjectProperties for Character {
    fn oxygen_users(&self) -> Vec<OxygenUser<usize>> {
        vec![OxygenUser {
            x: self.location.x.floor() as usize,
            y: self.location.y.floor() as usize,
            change_per_sec: 0.00001,
        }]
    }
}

const SURVIVE_GOAL_ORDER: [SurviveGoal; 2] =
    [SurviveGoal::RunFromDanger, SurviveGoal::PreventStarvation];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SurviveGoal {
    RunFromDanger,
    PreventStarvation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkGoal {
    WorkAtVentilation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CharacterGoal {
    Survive(SurviveGoal),
    Work(WorkGoal),
    Idle,
}

#[derive(Debug)]
pub(crate) enum CharacterTask {
    PanicRun { target_x: f32, target_y: f32 },
    WorkAtSpot { building: ObjectId<Building> },
    Idle,
}

pub(crate) struct AiChange {
    id: ObjectId<Character>,
    new_goal: CharacterGoal,
    new_task: CharacterTask,
}

impl<const WIDTH: usize, const HEIGHT: usize> Map<WIDTH, HEIGHT> {
    pub(crate) fn recalculate_ai(&self) -> Vec<AiChange> {
        let mut ai_changes = Vec::new();

        'character_loop: for character in self.characters.iter() {
            for possible_survive_goal in SURVIVE_GOAL_ORDER.iter() {
                if character.current_goal == CharacterGoal::Survive(*possible_survive_goal) {
                    // We already work on a goal of this importance
                    continue 'character_loop;
                }

                match possible_survive_goal {
                    SurviveGoal::RunFromDanger => {
                        let danger_detected = false;
                        if !danger_detected {
                            continue 'character_loop;
                        }
                        todo!()
                    }
                    SurviveGoal::PreventStarvation => {
                        let is_starving = false;
                        if !is_starving {
                            continue 'character_loop;
                        }
                        todo!()
                    }
                }
            }

            for possible_work_goal in character.work_goals_order.iter() {
                if character.current_goal == CharacterGoal::Work(*possible_work_goal) {
                    // We already work on a goal of this importance
                    continue 'character_loop;
                }

                match possible_work_goal {
                    WorkGoal::WorkAtVentilation => {
                        let mut open_reachable_ventilation_workspots = self
                            .buildings
                            .iter()
                            .filter(|building| building.building_type.is_ventilator())
                            .flat_map(|building| {
                                building
                                    .workspots()
                                    .into_iter()
                                    .filter(|workspot| workspot.occupation.is_open())
                            })
                            .filter_map(|workspot| {
                                find_path(character.location, workspot.location)
                                    .map(|path| (workspot, path))
                            })
                            .collect::<Vec<_>>();

                        let closest_workspot = open_reachable_ventilation_workspots.into_iter().min_by_key(|(_, path)| OrderedFloat(path.total_length()));
                    }
                }
            }
        }

        ai_changes
    }
}

fn find_path(from: Vec2, to: Vec2) -> Option<Path> {
    // TODO: Actual pathfinding
    Some(Path {
        points: vec![from, to],
    })
}

struct Path {
    points: Vec<Vec2>,
}

impl Path {
    fn total_length(&self) -> f32 {
        self.points
            .windows(2)
            .fold(0.0, |len, points| len + points[0].distance(points[1]))
    }
}
