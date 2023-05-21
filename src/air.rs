use std::ops::Add;

use crate::{
    liquids::{AnyLiquid, LiquidData},
    Facing, Map,
};

impl<const WIDTH: usize, const HEIGHT: usize> Map<WIDTH, HEIGHT> {
    pub(crate) fn calculate_air_diff(&self, delta_time: f32) -> [[AirDiff; HEIGHT]; WIDTH] {
        let mut air_diff_result = [[AirDiff::default(); HEIGHT]; WIDTH];

        const PRESSURE_SPREAD_RATE: f32 = 0.01;
        const DIFFUSION_SPREAD_RATE: f32 = 0.05;

        // In this model we will 'give away' air pressure and oxygen.

        for (x, y) in self.all_tile_coords() {
            let Some((air, liquids)) = self.tiles[x][y].tile_type.get_ground() else {
                    continue;
                };

            let air_pressure = air.air_pressure(liquids.get_level::<AnyLiquid>());

            let neighbour_airs = self
                // Get all neighbours
                .neighbour_tiles(x, y)
                // Get only the ones that are ground
                .filter_map(|(x, y, tile)| {
                    tile.tile_type
                        .get_ground()
                        .map(|(air, liquids)| (x, y, air, liquids))
                });

            let nitrogen_fraction = air.nitrogen_fraction();
            let oxygen_fraction = air.oxygen_fraction();
            let fumes_fraction = air.fumes_fraction();

            for (nx, ny, neighbour_air, neighbour_liquids) in neighbour_airs {
                let neighbour_air_pressure =
                    neighbour_air.air_pressure(neighbour_liquids.get_level::<AnyLiquid>());

                // Move air due to diffusion. We trade air equally. We give some, we take some
                let nitrogen_needed_for_equal = nitrogen_fraction * neighbour_air_pressure;
                let oxygen_needed_for_equal = oxygen_fraction * neighbour_air_pressure;
                let fumes_needed_for_equal = fumes_fraction * neighbour_air_pressure;

                let nitrogen_traded = nitrogen_needed_for_equal
                    .clamp(-neighbour_air.nitrogen, air.nitrogen / 8.0)
                    * DIFFUSION_SPREAD_RATE
                    * delta_time;
                let oxygen_traded = oxygen_needed_for_equal
                    .clamp(-neighbour_air.oxygen, air.oxygen / 8.0)
                    * DIFFUSION_SPREAD_RATE
                    * delta_time;
                let fumes_traded = fumes_needed_for_equal
                    .clamp(-neighbour_air.fumes, air.fumes / 8.0)
                    * DIFFUSION_SPREAD_RATE
                    * delta_time;

                air_diff_result[nx][ny].nitrogen += nitrogen_traded;
                air_diff_result[nx][ny].oxygen += oxygen_traded;
                air_diff_result[nx][ny].fumes += fumes_traded;

                air_diff_result[x][y].nitrogen -= nitrogen_traded;
                air_diff_result[x][y].oxygen -= oxygen_traded;
                air_diff_result[x][y].fumes -= fumes_traded;

                // Move air due to pressure difference
                if neighbour_air_pressure < air_pressure {
                    // It moves due to the total pressure difference, not the difference between each element separately
                    let pressure_delta = air_pressure - neighbour_air_pressure;
                    let applied_pressure_delta = ((pressure_delta * PRESSURE_SPREAD_RATE).sqrt()
                        * delta_time)
                        .min(air_pressure / 8.0);

                    let nitrogen_delta = applied_pressure_delta * nitrogen_fraction;
                    let oxygen_delta = applied_pressure_delta * oxygen_fraction;
                    let fumes_delta = applied_pressure_delta * fumes_fraction;

                    air_diff_result[nx][ny].nitrogen += nitrogen_delta;
                    air_diff_result[nx][ny].oxygen += oxygen_delta;
                    air_diff_result[nx][ny].fumes += fumes_delta;

                    air_diff_result[x][y].nitrogen -= nitrogen_delta;
                    air_diff_result[x][y].oxygen -= oxygen_delta;
                    air_diff_result[x][y].fumes -= fumes_delta;
                }
            }
        }

        air_diff_result
    }

    pub(crate) fn apply_air_diff(&mut self, air_diff: [[AirDiff; HEIGHT]; WIDTH], delta_time: f32) {
        for (x, y) in self.all_tile_coords() {
            let Some(air) = self.tiles[x][y].tile_type.get_air_mut() else {
                    continue;
                };

            air.nitrogen = air.nitrogen.add(air_diff[x][y].nitrogen).max(0.0);
            air.oxygen = air.oxygen.add(air_diff[x][y].oxygen).max(0.0);
            air.fumes = air.fumes.add(air_diff[x][y].fumes).max(0.0);
        }

        for map_object in self.objects.read().unwrap().get_all_objects_mut() {
            for air_leveler in map_object.air_levelers() {
                let Some(air) = self.tiles[air_leveler.x][air_leveler.y].tile_type.get_air_mut() else {
                    continue;
                };

                air.nitrogen = air_leveler.nitrogen;
                air.oxygen = air_leveler.oxygen;
                air.fumes = air_leveler.fumes;
            }

            for oxygen_user in map_object.oxygen_users() {
                let Some(air) = self.tiles[oxygen_user.x][oxygen_user.y].tile_type.get_air_mut() else {
                    continue;
                };

                if air.oxygen < oxygen_user.change_per_sec * delta_time {
                    continue;
                }

                air.oxygen -= oxygen_user.change_per_sec * delta_time;
                air.fumes += oxygen_user.change_per_sec * delta_time;
            }

            for air_pusher in map_object.air_pushers() {
                let Some((push_x, push_y)) = air_pusher.direction
                    .move_coords_in_direction::<WIDTH, HEIGHT>(air_pusher.x, air_pusher.y) else {
                        continue;
                    };

                let Some(source_air) = self.tiles[air_pusher.x][air_pusher.y].tile_type.get_air() else {
                    continue;
                };

                let nitrogen_taken = source_air.nitrogen * air_pusher.amount * delta_time;
                let oxygen_taken = source_air.oxygen * air_pusher.amount * delta_time;
                let fumes_taken = source_air.fumes * air_pusher.amount * delta_time;

                let Some(target_air) = self.tiles[push_x][push_y].tile_type.get_air_mut() else {
                    continue;
                };

                target_air.nitrogen += nitrogen_taken;
                target_air.oxygen += oxygen_taken;
                target_air.fumes += fumes_taken;

                let source_air = self.tiles[air_pusher.x][air_pusher.y]
                    .tile_type
                    .get_air_mut()
                    .unwrap();

                source_air.nitrogen -= nitrogen_taken;
                source_air.oxygen -= oxygen_taken;
                source_air.fumes -= fumes_taken;
            }
        }
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub(crate) struct AirDiff {
    nitrogen: f32,
    oxygen: f32,
    fumes: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct AirData {
    pub nitrogen: f32,
    pub oxygen: f32,
    pub fumes: f32,
}

impl AirData {
    pub const fn new_default() -> Self {
        Self {
            nitrogen: 0.79,
            oxygen: 0.21,
            fumes: 0.0,
        }
    }

    pub(crate) fn nitrogen_fraction(&self) -> f32 {
        self.nitrogen / (self.nitrogen + self.oxygen + self.fumes)
    }

    pub(crate) fn oxygen_fraction(&self) -> f32 {
        self.oxygen / (self.nitrogen + self.oxygen + self.fumes)
    }

    pub(crate) fn fumes_fraction(&self) -> f32 {
        self.fumes / (self.nitrogen + self.oxygen + self.fumes)
    }

    pub(crate) fn air_pressure(&self, liquid_level: f32) -> f32 {
        (self.nitrogen + self.oxygen + self.fumes)
            / (1.0 - liquid_level / LiquidData::MAX_LEVEL).max(0.001)
    }
}

impl Default for AirData {
    fn default() -> Self {
        Self::new_default()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AirLeveler<COORD> {
    pub x: COORD,
    pub y: COORD,
    pub nitrogen: f32,
    pub oxygen: f32,
    pub fumes: f32,
}

impl AirLeveler<isize> {
    pub(crate) fn to_absolute(self, base_x: usize, base_y: usize) -> AirLeveler<usize> {
        AirLeveler {
            x: base_x.wrapping_add_signed(self.x),
            y: base_y.wrapping_add_signed(self.y),
            nitrogen: self.nitrogen,
            oxygen: self.oxygen,
            fumes: self.fumes,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OxygenUser<COORD> {
    pub x: COORD,
    pub y: COORD,
    pub change_per_sec: f32,
}

impl OxygenUser<isize> {
    pub(crate) fn to_absolute(self, base_x: usize, base_y: usize) -> OxygenUser<usize> {
        OxygenUser {
            x: base_x.wrapping_add_signed(self.x),
            y: base_y.wrapping_add_signed(self.y),
            change_per_sec: self.change_per_sec,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AirPusher<COORD> {
    pub x: COORD,
    pub y: COORD,
    pub direction: Facing,
    /// Fraction of the air in the pusher location that is push into the given direction per second
    pub amount: f32,
}

impl AirPusher<isize> {
    pub(crate) fn to_absolute(
        self,
        base_x: usize,
        base_y: usize,
        base_direction: Facing,
    ) -> AirPusher<usize> {
        AirPusher {
            x: base_x.wrapping_add_signed(self.x),
            y: base_y.wrapping_add_signed(self.y),
            direction: base_direction.rotate(self.direction),
            amount: self.amount,
        }
    }
}
