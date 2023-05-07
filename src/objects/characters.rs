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
    pub(crate) current_task: CharacterTask,
    pub(crate) current_path: Option<Path>,
}

impl Character {
    pub fn new(location: Vec2, health: f32, work_goals_order: Vec<WorkGoal>) -> Self {
        Self {
            location,
            health,
            work_goals_order,
            current_goal: CharacterGoal::Idle,
            current_task: CharacterTask::Idle,
            current_path: None,
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

#[derive(Debug, Clone)]
pub(crate) enum CharacterTask {
    PanicRun {
        target_x: f32,
        target_y: f32,
    },
    WorkAtSpot {
        building: ObjectId<Building>,
        workspot_index: usize,
    },
    Idle,
}

#[derive(Debug)]
pub(crate) struct AiChange {
    character_id: ObjectId<Character>,
    new_goal: CharacterGoal,
    new_task: CharacterTask,
    new_path: Option<Path>,
}

impl<const WIDTH: usize, const HEIGHT: usize> Map<WIDTH, HEIGHT> {
    pub(crate) fn calculate_ai_changes(&self) -> Vec<AiChange> {
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
                        let closest_workspot = self
                            .buildings
                            .iter()
                            .filter(|building| building.building_type.is_ventilator())
                            .flat_map(|building| {
                                building
                                    .workspots()
                                    .into_iter()
                                    .enumerate()
                                    .filter(|(_, workspot)| workspot.occupation.is_open())
                                    .map(|(workspot_index, workspot)| {
                                        (workspot_index, workspot, building.id())
                                    })
                            })
                            .filter_map(|workspot| {
                                find_path(character.location, workspot.1.location)
                                    .map(|path| (workspot.0, workspot.2, path))
                            })
                            .min_by_key(|(_, _, path)| OrderedFloat(path.total_length()));

                        if let Some((closest_workspot_index, building_id, path)) = closest_workspot
                        {
                            ai_changes.push(AiChange {
                                character_id: character.id(),
                                new_goal: CharacterGoal::Work(WorkGoal::WorkAtVentilation),
                                new_task: CharacterTask::WorkAtSpot {
                                    building: building_id,
                                    workspot_index: closest_workspot_index,
                                },
                                new_path: Some(path),
                            })
                        }
                    }
                }
            }
        }

        ai_changes
    }

    pub(crate) fn apply_ai_changes(&mut self, ai_changes: impl Iterator<Item = AiChange>) {
        for ai_change in ai_changes {
            // We need to make some changes to the environment like workspot claims
            match &ai_change.new_task {
                CharacterTask::PanicRun { target_x, target_y } => todo!(),
                CharacterTask::WorkAtSpot {
                    building,
                    workspot_index,
                } => {
                    let Some(target_building) = self.get_object_mut(*building) else {
                        log::warn!("Could not get building {:?}", building);
                        continue;
                    };

                    if target_building
                        .claim_workspot(*workspot_index, ai_change.character_id)
                        .is_err()
                    {
                        // Could not claim the workspot, likely that another character has just taken this
                        continue;
                    }
                }
                CharacterTask::Idle => todo!(),
            }

            let Some(character) = self.get_object_mut(ai_change.character_id) else {
                log::warn!("Could not get character {:?}", ai_change.character_id);
                continue;
            };

            // We need to book off anything the character will stop doing like old workspots

            match character.current_task.clone() {
                CharacterTask::PanicRun { target_x, target_y } => todo!(),
                CharacterTask::WorkAtSpot {
                    building,
                    workspot_index,
                } => {
                    if let Some(target_building) = self.get_object_mut(building) {
                        target_building.release_workspot(workspot_index);
                    } else {
                        log::warn!("Could not get building {:?}", building);
                    }
                }
                CharacterTask::Idle => {}
            }

            // TODO: We need search for the character again because of lifetimes. I should find a solution for this
            let Some(character) = self.get_object_mut(ai_change.character_id) else {
                log::warn!("Could not get character {:?}", ai_change.character_id);
                continue;
            };

            character.current_goal = ai_change.new_goal;
            character.current_task = ai_change.new_task;
            character.current_path = ai_change.new_path;
        }
    }
}

fn find_path(from: Vec2, to: Vec2) -> Option<Path> {
    // TODO: Actual pathfinding
    Some(Path {
        points: vec![from, to],
    })
}

#[derive(Debug)]
pub(crate) struct Path {
    points: Vec<Vec2>,
}

impl Path {
    pub(crate) fn total_length(&self) -> f32 {
        self.points
            .windows(2)
            .fold(0.0, |len, points| len + points[0].distance(points[1]))
    }
}
