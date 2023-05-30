use glam::Vec2;
use ordered_float::OrderedFloat;

use super::{building::Building, ObjectId, ObjectProperties};
use crate::{
    air::OxygenUser,
    liquids::{AnyLiquid, Lava},
    Map,
};

/// Walk speed in meters per second
const CHARACTER_WALK_SPEED: f32 = 1.2;

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
    const LIQUID_DROWN_HEIGHT: f32 = 2.0;

    pub(crate) fn calculate_ai_changes(&self) -> Vec<AiChange> {
        let mut ai_changes = Vec::new();

        'character_loop: for character in self.objects().get_objects::<Character>() {
            'survive_loop: for possible_survive_goal in SURVIVE_GOAL_ORDER.iter() {
                if character.current_goal == CharacterGoal::Survive(*possible_survive_goal) {
                    // We already work on a goal of this importance
                    continue 'character_loop;
                }

                match possible_survive_goal {
                    SurviveGoal::RunFromDanger => {
                        let danger_detected = false;
                        if !danger_detected {
                            continue 'survive_loop;
                        }
                        todo!()
                    }
                    SurviveGoal::PreventStarvation => {
                        let is_starving = false;
                        if !is_starving {
                            continue 'survive_loop;
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
                            .objects()
                            // Get all buildings
                            .get_objects::<Building>()
                            // Only keep the ventilators
                            .filter(|building| building.building_type.is_ventilator())
                            // Get the open workspots of the ventilator and its index and the building id
                            .flat_map(|building| {
                                building
                                    .workspots()
                                    .into_iter()
                                    .enumerate()
                                    .filter(|(_, workspot)| workspot.occupation.is_open())
                                    .map(move |(workspot_index, workspot)| {
                                        (workspot_index, workspot, building.id())
                                    })
                            })
                            // Calculate the path to the workspot and only keep the workspots that have a valid path
                            .filter_map(|workspot| {
                                self.find_path(character.location, workspot.1.location, true, true)
                                    .map(|path| (workspot.0, workspot.2, path))
                            })
                            // Take the workspot with the shortest path
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
        let objects = self.objects();

        for ai_change in ai_changes {
            // We need to make some changes to the environment like workspot claims
            match &ai_change.new_task {
                CharacterTask::PanicRun { .. } => todo!(),
                CharacterTask::WorkAtSpot {
                    building,
                    workspot_index,
                } => {
                    let Some(mut target_building) = objects.get_object_mut(*building) else {
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

            let Some(mut character) = objects.get_object_mut(ai_change.character_id) else {
                log::warn!("Could not get character {:?}", ai_change.character_id);
                continue;
            };

            // We need to book off anything the character will stop doing like old workspots

            match character.current_task.clone() {
                CharacterTask::PanicRun { .. } => todo!(),
                CharacterTask::WorkAtSpot {
                    building,
                    workspot_index,
                } => {
                    if let Some(mut target_building) = objects.get_object_mut(building) {
                        target_building.release_workspot(workspot_index);
                    } else {
                        log::warn!("Could not get building {:?}", building);
                    }
                }
                CharacterTask::Idle => {}
            }

            character.current_goal = ai_change.new_goal;
            character.current_task = ai_change.new_task;
            character.current_path = ai_change.new_path;
        }
    }

    pub(crate) fn perform_ai_tick(&mut self, delta_time: f32) {
        let objects = self.objects.read().unwrap();

        for mut character in objects.get_objects_mut::<Character>() {
            let arrived_at_destination = if let Some(mut path) = character.current_path.take() {
                let mut distance_to_go = CHARACTER_WALK_SPEED * delta_time;

                while distance_to_go.min(path.total_length()) > f32::EPSILON {
                    let walk_vector = path.points[1] - path.points[0];
                    let walk_distance = walk_vector.length();
                    let walk_direction = walk_vector / walk_distance;

                    let distance_walked = walk_distance.min(distance_to_go);
                    character.location += walk_direction * distance_walked;
                    path.points[0] = character.location;

                    distance_to_go -= distance_walked;

                    if path.points[0].distance(path.points[1]) < f32::EPSILON {
                        path.points.remove(0);
                    }
                }

                if path.points.len() < 2 {
                    character.location = path.points[0];
                    true
                } else {
                    character.current_path = Some(path);
                    false
                }
            } else {
                false
            };

            if arrived_at_destination {
                match character.current_task {
                    CharacterTask::PanicRun { .. } => todo!(),
                    CharacterTask::WorkAtSpot {
                        building,
                        workspot_index,
                    } => {
                        let Some(mut target_building) = objects.get_object_mut(building) else {
                            character.current_goal = CharacterGoal::Idle;
                            character.current_task = CharacterTask::Idle;
                            log::warn!("Could not get building {building:?} to work at workspot {workspot_index:?}");
                            continue;
                        };

                        if target_building
                            .start_work_at_workspot(workspot_index, character.id())
                            .is_err()
                        {
                            character.current_goal = CharacterGoal::Idle;
                            character.current_task = CharacterTask::Idle;
                            log::warn!("Could not work at the designated spot at building {building:?} workspot {workspot_index:?}");
                        }
                    }
                    CharacterTask::Idle => todo!(),
                }
            }
        }
    }

    fn find_path(
        &self,
        from: Vec2,
        to: Vec2,
        avoid_lava: bool,
        avoid_drowning: bool,
    ) -> Option<Path> {
        // First make sure the from and to vectors are valid open positions
        self.position_penalty(from, avoid_lava, avoid_drowning)?;
        self.position_penalty(to, avoid_lava, avoid_drowning)?;

        // TODO: Actual pathfinding
        Some(Path {
            points: vec![from, to],
        })
    }

    /// - None if the position cannot be walked at all
    /// - Some with number if walkable. Lower numbers are preferential.
    fn position_penalty(
        &self,
        pos: Vec2,
        avoid_lava: bool,
        avoid_drowning: bool,
    ) -> Option<OrderedFloat<f32>> {
        let tile_coord = pos.as_uvec2();
        let tile = &self.tiles[tile_coord.x as usize][tile_coord.y as usize];

        // Tile must have a ground, may have a little bit of water and optionally a bit of lava (so we can pathfind to escape it)
        let liquids = tile.tile_type.get_liquids()?;

        let liquid_level = liquids.get_level::<AnyLiquid>();
        let will_drown = liquid_level > Self::LIQUID_DROWN_HEIGHT;
        let is_lava = liquids.get_level::<Lava>() > 0.001;

        if will_drown && avoid_drowning || is_lava && avoid_lava {
            return None;
        }

        Some(
            (liquid_level
                * if is_lava { 100000.0 } else { 1.0 }
                * if will_drown { 100000.0 } else { 1.0 })
            .into(),
        )
    }
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
