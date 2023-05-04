#![feature(const_type_id)]

use building::Building;
use characters::Character;
use environment_object::EnvironmentObject;
use object_id::{Object, ObjectId, ObjectProperties};
use std::{
    any::{type_name, TypeId},
    ops::{Add, Deref},
};

pub mod building;
pub mod characters;
pub mod environment_object;
pub mod object_id;

#[derive(Debug)]
pub struct Map<const WIDTH: usize, const HEIGHT: usize> {
    pub tiles: [[Tile; HEIGHT]; WIDTH],
    next_object_id: usize,
    current_time: f64,
    environment_objects: Vec<Object<EnvironmentObject>>,
    buildings: Vec<Object<Building>>,
    characters: Vec<Object<Character>>,
}

/// Get an `Iterator<item = &dyn ObjectProperties>` containing all map objects.
/// This is a macro because a function would borrow the whole map object instead of just the object fields
macro_rules! all_map_objects {
    ($map:ident) => {{
        let eo = $map
            .environment_objects
            .iter()
            .map(|val| val.deref() as &dyn ObjectProperties);
        let b = $map
            .buildings
            .iter()
            .map(|val| val.deref() as &dyn ObjectProperties);
        let c = $map
            .characters
            .iter()
            .map(|val| val.deref() as &dyn ObjectProperties);

        eo.chain(b).chain(c)
    }};
}

const ENVIRONMENT_OBJECT: TypeId = TypeId::of::<EnvironmentObject>();
const BUILDING_OBJECT: TypeId = TypeId::of::<Building>();
const CHARACTER_OBJECT: TypeId = TypeId::of::<Character>();

impl<const WIDTH: usize, const HEIGHT: usize> Map<WIDTH, HEIGHT> {
    pub const fn new_default() -> Self {
        Self {
            tiles: [[Tile::new_default(); HEIGHT]; WIDTH],
            next_object_id: 0,
            current_time: 0.0,
            environment_objects: Vec::new(),
            buildings: Vec::new(),
            characters: Vec::new(),
        }
    }

    pub fn push_object<T: ObjectProperties>(&mut self, object: impl Into<T>) -> ObjectId<T> {
        let object = object.into();

        let new_object_id = self.next_object_id;
        self.next_object_id += 1;

        let object = Object {
            id: new_object_id,
            object,
        };
        let object_id = object.id();
        self.get_vec_of_type_mut().push(object);

        object_id
    }

    pub fn remove_object<T: ObjectProperties>(&mut self, id: ObjectId<T>) {
        let object_vec = self.get_vec_of_type_mut::<T>();
        let index = object_vec
            .iter()
            .enumerate()
            .find_map(|(index, object)| (object.id() == id).then_some(index))
            .unwrap();

        object_vec.remove(index);
    }

    pub fn get_object<T: ObjectProperties>(&mut self, id: ObjectId<T>) -> Option<&Object<T>> {
        self.get_vec_of_type::<T>()
            .iter()
            .find(|obj| obj.id() == id)
    }

    pub fn get_object_mut<T: ObjectProperties>(
        &mut self,
        id: ObjectId<T>,
    ) -> Option<&mut Object<T>> {
        self.get_vec_of_type_mut::<T>()
            .iter_mut()
            .find(|obj| obj.id() == id)
    }

    fn get_vec_of_type<T: ObjectProperties>(&self) -> &Vec<Object<T>> {
        match TypeId::of::<T>() {
            ENVIRONMENT_OBJECT => unsafe { std::mem::transmute(&self.environment_objects) },
            BUILDING_OBJECT => unsafe { std::mem::transmute(&self.buildings) },
            CHARACTER_OBJECT => unsafe { std::mem::transmute(&self.characters) },
            _ => unreachable!(),
        }
    }

    fn get_vec_of_type_mut<T: ObjectProperties>(&mut self) -> &mut Vec<Object<T>> {
        match TypeId::of::<T>() {
            ENVIRONMENT_OBJECT => unsafe { std::mem::transmute(&mut self.environment_objects) },
            BUILDING_OBJECT => unsafe { std::mem::transmute(&mut self.buildings) },
            CHARACTER_OBJECT => unsafe { std::mem::transmute(&mut self.characters) },
            _ => unreachable!("{} is not covered", type_name::<T>()),
        }
    }

    pub fn all_tile_coords() -> impl Iterator<Item = (usize, usize)> {
        (0..WIDTH)
            .map(|x| (0..HEIGHT).map(move |y| (x, y)))
            .flatten()
    }

    #[cfg(test)]
    pub fn collect_air_pressure_map(&self) -> [[f32; HEIGHT]; WIDTH] {
        let mut result = [[0.0; HEIGHT]; WIDTH];

        for (x, y) in Self::all_tile_coords() {
            result[x][y] = self.tiles[x][y]
                .tile_type
                .get_ground()
                .map(|(air, liquids)| air.air_pressure(liquids.get_level::<AnyLiquid>()))
                .unwrap_or(f32::NAN);
        }

        result
    }

    #[cfg(test)]
    pub fn collect_oxygen_map(&self) -> [[f32; HEIGHT]; WIDTH] {
        let mut result = [[0.0; HEIGHT]; WIDTH];

        for (x, y) in Self::all_tile_coords() {
            result[x][y] = self.tiles[x][y]
                .tile_type
                .get_air()
                .map(|air| air.oxygen_fraction())
                .unwrap_or(f32::NAN);
        }

        result
    }

    #[cfg(test)]
    pub fn collect_fumes_map(&self) -> [[f32; HEIGHT]; WIDTH] {
        let mut result = [[0.0; HEIGHT]; WIDTH];

        for (x, y) in Self::all_tile_coords() {
            result[x][y] = self.tiles[x][y]
                .tile_type
                .get_air()
                .map(|air| air.fumes_fraction())
                .unwrap_or(f32::NAN);
        }

        result
    }

    #[cfg(test)]
    pub fn collect_liquids_map<L: Liquid>(&self) -> [[f32; HEIGHT]; WIDTH] {
        let mut result = [[0.0; HEIGHT]; WIDTH];

        for (x, y) in Self::all_tile_coords() {
            result[x][y] = self.tiles[x][y]
                .tile_type
                .get_liquids()
                .map(|liquids| liquids.get_level::<L>())
                .unwrap_or(f32::NAN);
        }

        result
    }

    #[cfg(test)]
    pub fn collect_surface_level_map(&self) -> [[f32; HEIGHT]; WIDTH] {
        let mut result = [[0.0; HEIGHT]; WIDTH];

        for (x, y) in Self::all_tile_coords() {
            result[x][y] = self.tiles[x][y]
                .tile_type
                .get_liquids()
                .map(|liquids| self.tiles[x][y].ground_level + liquids.get_level::<AnyLiquid>())
                .unwrap_or(self.tiles[x][y].ground_level);
        }

        result
    }

    #[cfg(test)]
    pub fn collect_ground_level_map(&self) -> [[f32; HEIGHT]; WIDTH] {
        let mut result = [[0.0; HEIGHT]; WIDTH];

        for (x, y) in Self::all_tile_coords() {
            result[x][y] = self.tiles[x][y].ground_level;
        }

        result
    }

    fn neighbour_tile_coords(
        target_tile_x: usize,
        target_tile_y: usize,
    ) -> impl Iterator<Item = (usize, usize)> + Clone {
        let has_neg_x_neighbour = target_tile_x > 0;
        let has_neg_y_neighbour = target_tile_y > 0;
        let has_pos_x_neighbour = target_tile_x < WIDTH - 1;
        let has_pos_y_neighbour = target_tile_y < HEIGHT - 1;

        [
            (has_neg_x_neighbour && has_neg_y_neighbour)
                .then(|| (target_tile_x - 1, target_tile_y - 1)),
            (has_neg_x_neighbour).then(|| (target_tile_x - 1, target_tile_y)),
            (has_neg_x_neighbour && has_pos_y_neighbour)
                .then(|| (target_tile_x - 1, target_tile_y + 1)),
            (has_neg_y_neighbour).then(|| (target_tile_x, target_tile_y - 1)),
            (has_pos_y_neighbour).then(|| (target_tile_x, target_tile_y + 1)),
            (has_pos_x_neighbour && has_neg_y_neighbour)
                .then(|| (target_tile_x + 1, target_tile_y - 1)),
            (has_pos_x_neighbour).then(|| (target_tile_x + 1, target_tile_y)),
            (has_pos_x_neighbour && has_pos_y_neighbour)
                .then(|| (target_tile_x + 1, target_tile_y + 1)),
        ]
        .into_iter()
        .filter_map(|t| t)
    }

    fn neighbour_tiles(
        &self,
        target_tile_x: usize,
        target_tile_y: usize,
    ) -> impl Iterator<Item = (usize, usize, &Tile)> + Clone {
        Self::neighbour_tile_coords(target_tile_x, target_tile_y)
            .map(|(x, y)| (x, y, &self.tiles[x][y]))
    }

    pub fn simulate(&mut self, delta_time: f32) {
        let mut air_diff = [[AirDiff::default(); HEIGHT]; WIDTH];
        let mut water_diff = [[0.0; HEIGHT]; WIDTH];
        let mut lava_diff = [[0.0; HEIGHT]; WIDTH];

        rayon::scope(|s| {
            s.spawn(|_| air_diff = self.calculate_air_diff(delta_time));
            s.spawn(|_| water_diff = self.calculate_liquid_diff::<Water>(delta_time));
            s.spawn(|_| lava_diff = self.calculate_liquid_diff::<Lava>(delta_time));
        });

        self.apply_air_diff(air_diff, delta_time);
        self.apply_liquid_diff(water_diff, lava_diff);

        self.current_time += delta_time as f64;
    }

    fn calculate_air_diff(&self, delta_time: f32) -> [[AirDiff; HEIGHT]; WIDTH] {
        let mut air_diff_result = [[AirDiff::default(); HEIGHT]; WIDTH];

        const PRESSURE_SPREAD_RATE: f32 = 0.01;
        const DIFFUSION_SPREAD_RATE: f32 = 0.05;

        // In this model we will 'give away' air pressure and oxygen.

        for (x, y) in Self::all_tile_coords() {
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

    fn apply_air_diff(&mut self, air_diff: [[AirDiff; HEIGHT]; WIDTH], delta_time: f32) {
        for (x, y) in Self::all_tile_coords() {
            let Some(air) = self.tiles[x][y].tile_type.get_air_mut() else {
                    continue;
                };

            air.nitrogen = air.nitrogen.add(air_diff[x][y].nitrogen).max(0.0);
            air.oxygen = air.oxygen.add(air_diff[x][y].oxygen).max(0.0);
            air.fumes = air.fumes.add(air_diff[x][y].fumes).max(0.0);
        }

        for map_object in all_map_objects!(self) {
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

    fn calculate_liquid_diff<L: Liquid>(&self, delta_time: f32) -> [[f32; HEIGHT]; WIDTH] {
        let mut liquid_diff_result = [[0.0; HEIGHT]; WIDTH];

        for (x, y) in Self::all_tile_coords() {
            let Some(liquids) = self.tiles[x][y].tile_type.get_liquids() else {
                continue;
            };
            let ground_level = self.tiles[x][y].ground_level;
            let liquid_level = liquids.get_level::<L>();
            let total_level = ground_level + liquid_level;

            if liquid_level < L::MINIMAL_HEIGHT_TO_SPREAD {
                continue;
            }

            let neighbour_liquids = self
                // Get all neighbours
                .neighbour_tiles(x, y)
                // Get only the ones that are ground
                .filter_map(|(x, y, tile)| {
                    tile.tile_type
                        .get_liquids()
                        .map(|liquids| (x, y, tile.ground_level, liquids.get_level::<L>()))
                });

            for (nx, ny, neighbour_ground_level, neighbour_liquid_level) in neighbour_liquids {
                let neighbour_total_level = neighbour_ground_level + neighbour_liquid_level;
                if neighbour_total_level >= total_level
                    || neighbour_liquid_level >= LiquidData::MAX_LEVEL
                {
                    continue;
                }

                let height_delta = total_level - neighbour_total_level;
                let applied_height_delta =
                    ((height_delta * L::SPREAD_RATE).sqrt() * delta_time).min(liquid_level / 0.8);

                liquid_diff_result[nx][ny] += applied_height_delta;
                liquid_diff_result[x][y] -= applied_height_delta;
            }
        }

        liquid_diff_result
    }

    fn apply_liquid_diff(
        &mut self,
        water_diff: [[f32; HEIGHT]; WIDTH],
        lava_diff: [[f32; HEIGHT]; WIDTH],
    ) {
        for (x, y) in Self::all_tile_coords() {
            let Some(liquids) = self.tiles[x][y].tile_type.get_liquids_mut() else {
                    continue;
                };

            let new_water_level = (liquids.get_level::<Water>() + water_diff[x][y]).max(0.0);
            let new_lava_level = (liquids.get_level::<Lava>() + lava_diff[x][y]).max(0.0);

            *liquids = if new_water_level == 0.0 && new_lava_level == 0.0 {
                LiquidData::None
            } else {
                let difference = new_water_level - new_lava_level;

                if new_water_level > 0.0 && new_lava_level > 0.0 {
                    self.tiles[x][y].ground_level += difference.abs();
                }

                if difference >= 0.0 {
                    LiquidData::Water { level: difference }
                } else {
                    LiquidData::Lava { level: -difference }
                }
            }
        }

        for liquid_leveler in all_map_objects!(self)
            .map(|object| object.liquid_levelers())
            .flatten()
        {
            let Some(liquids) = self.tiles[liquid_leveler.x][liquid_leveler.y].tile_type.get_liquids_mut() else {
                continue;
            };

            *liquids = liquid_leveler.target;
        }
    }
}

impl<const WIDTH: usize, const HEIGHT: usize> Default for Map<WIDTH, HEIGHT> {
    fn default() -> Self {
        Self::new_default()
    }
}

#[derive(Default, Clone, Copy, Debug)]
struct AirDiff {
    nitrogen: f32,
    oxygen: f32,
    fumes: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Tile {
    pub ground_level: f32,
    pub tile_type: TileType,
}

impl Tile {
    pub fn new(ground_level: f32, tile_type: TileType) -> Self {
        Self {
            ground_level,
            tile_type,
        }
    }

    pub const fn new_default() -> Self {
        Self {
            ground_level: 0.0,
            tile_type: TileType::new_default(),
        }
    }
}

impl Default for Tile {
    fn default() -> Self {
        Self::new_default()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TileType {
    Wall,
    Ground { air: AirData, liquids: LiquidData },
}

impl TileType {
    pub const fn new_default() -> Self {
        TileType::Ground {
            air: AirData::new_default(),
            liquids: LiquidData::new_default(),
        }
    }

    #[inline(always)]
    pub fn get_ground(&self) -> Option<(&AirData, &LiquidData)> {
        if let Self::Ground { air, liquids } = self {
            Some((air, liquids))
        } else {
            None
        }
    }
    #[inline(always)]
    pub fn get_ground_mut(&mut self) -> Option<(&mut AirData, &mut LiquidData)> {
        if let Self::Ground { air, liquids } = self {
            Some((air, liquids))
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn get_air(&self) -> Option<&AirData> {
        if let Self::Ground { air, .. } = self {
            Some(air)
        } else {
            None
        }
    }
    #[inline(always)]
    pub fn get_air_mut(&mut self) -> Option<&mut AirData> {
        if let Self::Ground { air, .. } = self {
            Some(air)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn get_liquids(&self) -> Option<&LiquidData> {
        if let Self::Ground { liquids, .. } = self {
            Some(liquids)
        } else {
            None
        }
    }
    #[inline(always)]
    pub fn get_liquids_mut(&mut self) -> Option<&mut LiquidData> {
        if let Self::Ground { liquids, .. } = self {
            Some(liquids)
        } else {
            None
        }
    }
}

impl Default for TileType {
    fn default() -> Self {
        Self::new_default()
    }
}
#[derive(Clone, Copy, Debug)]
pub struct AirData {
    nitrogen: f32,
    oxygen: f32,
    fumes: f32,
}

impl AirData {
    pub const fn new_default() -> Self {
        Self {
            nitrogen: 0.79,
            oxygen: 0.21,
            fumes: 0.0,
        }
    }

    #[inline(always)]
    pub fn nitrogen_fraction(&self) -> f32 {
        self.nitrogen / (self.nitrogen + self.oxygen + self.fumes)
    }

    #[inline(always)]
    pub fn oxygen_fraction(&self) -> f32 {
        self.oxygen / (self.nitrogen + self.oxygen + self.fumes)
    }

    #[inline(always)]
    pub fn fumes_fraction(&self) -> f32 {
        self.fumes / (self.nitrogen + self.oxygen + self.fumes)
    }

    #[inline(always)]
    pub fn air_pressure(&self, liquid_level: f32) -> f32 {
        (self.nitrogen + self.oxygen + self.fumes)
            / (1.0 - liquid_level / LiquidData::MAX_LEVEL).max(0.001)
    }
}

impl Default for AirData {
    fn default() -> Self {
        Self::new_default()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum LiquidData {
    None,
    Water { level: f32 },
    Lava { level: f32 },
}

impl LiquidData {
    const MAX_LEVEL: f32 = 3.0;

    pub const fn new_default() -> Self {
        Self::None
    }

    #[inline(always)]
    fn get_level<L: Liquid>(&self) -> f32 {
        self.get_level_optional::<L>().unwrap_or_default()
    }

    #[inline(always)]
    fn get_level_optional<L: Liquid>(&self) -> Option<f32> {
        L::get_level(self)
    }
}

impl Default for LiquidData {
    fn default() -> Self {
        Self::new_default()
    }
}

pub trait Liquid {
    const SPREAD_RATE: f32;
    const MINIMAL_HEIGHT_TO_SPREAD: f32;

    fn get_level(data: &LiquidData) -> Option<f32>;
}

struct AnyLiquid;
impl Liquid for AnyLiquid {
    const SPREAD_RATE: f32 = 0.0;
    const MINIMAL_HEIGHT_TO_SPREAD: f32 = 0.0;

    #[inline(always)]
    fn get_level(data: &LiquidData) -> Option<f32> {
        match data {
            LiquidData::None => None,
            LiquidData::Water { level } => Some(*level),
            LiquidData::Lava { level } => Some(*level),
        }
    }
}

struct Water;
impl Liquid for Water {
    const SPREAD_RATE: f32 = 0.01;
    const MINIMAL_HEIGHT_TO_SPREAD: f32 = 0.01;

    #[inline(always)]
    fn get_level(data: &LiquidData) -> Option<f32> {
        match data {
            LiquidData::Water { level } => Some(*level),
            _ => None,
        }
    }
}

struct Lava;
impl Liquid for Lava {
    const SPREAD_RATE: f32 = 0.001;
    const MINIMAL_HEIGHT_TO_SPREAD: f32 = 0.1;

    #[inline(always)]
    fn get_level(data: &LiquidData) -> Option<f32> {
        match data {
            LiquidData::Lava { level } => Some(*level),
            _ => None,
        }
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
    pub fn to_absolute(self, base_x: usize, base_y: usize) -> AirLeveler<usize> {
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
    pub fn to_absolute(self, base_x: usize, base_y: usize) -> OxygenUser<usize> {
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
    pub fn to_absolute(
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

#[derive(Debug, Clone, Copy)]
pub struct LiquidLeveler<COORD> {
    pub x: COORD,
    pub y: COORD,
    pub target: LiquidData,
}

impl LiquidLeveler<isize> {
    pub fn to_absolute(self, base_x: usize, base_y: usize) -> LiquidLeveler<usize> {
        LiquidLeveler {
            x: base_x.wrapping_add_signed(self.x),
            y: base_y.wrapping_add_signed(self.y),
            target: self.target,
        }
    }
}

/// A cardinal direction something can be facing to.
///
/// When used for rotation (for example in buildings), the North facing is the identity rotation.
///
/// The coord system has the 0,0 at the North-West.
/// So going north is -y, going east is +x, going south is +y, going west is -x.
#[derive(Debug, Clone, Copy, num_enum::UnsafeFromPrimitive, PartialEq, Eq)]
#[repr(u8)]
pub enum Facing {
    North,
    East,
    South,
    West,
}

impl Facing {
    pub fn move_coords_in_direction<const WIDTH: usize, const HEIGHT: usize>(
        &self,
        x: usize,
        y: usize,
    ) -> Option<(usize, usize)> {
        match self {
            Facing::North => (y > 0).then(|| (x, y - 1)),
            Facing::East => (x < WIDTH - 1).then(|| (x + 1, y)),
            Facing::South => (y < HEIGHT - 1).then(|| (x, y + 1)),
            Facing::West => (x > 0).then(|| (x - 1, y)),
        }
    }

    /// Rotates a facing. The default is North.
    ///
    /// So East rotate East = South.
    pub fn rotate(self, applied: Facing) -> Self {
        use num_enum::UnsafeFromPrimitive;
        let new_discriminant = (self as u8 + applied as u8) % 4;
        unsafe { Self::unchecked_transmute_from(new_discriminant) }
    }

    /// Rotate the given coords according to the facing.
    /// They will be rotated relative to 0,0
    pub fn rotate_isize_coords(&self, x: isize, y: isize) -> (isize, isize) {
        match self {
            Facing::North => (x, y),
            Facing::East => (-y, x),
            Facing::South => (-x, -y),
            Facing::West => (y, -x),
        }
    }

    /// Rotate the given coords according to the facing.
    /// They will be rotated relative to 0.5,0.5 (which is the middle of tile 0,0)
    pub fn rotate_f32_coords(&self, mut x: f32, mut y: f32) -> (f32, f32) {
        x -= 0.5;
        y -= 0.5;

        let (x, y) = match self {
            Facing::North => (x, y),
            Facing::East => (-y, x),
            Facing::South => (-x, -y),
            Facing::West => (y, -x),
        };

        (x + 0.5, y + 0.5)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, path::PathBuf};

    use approx::assert_relative_eq;

    use super::*;

    #[test]
    fn neighbours() {
        let neighbours = Map::<10, 10>::neighbour_tile_coords(0, 0).collect::<Vec<_>>();

        assert!(neighbours.contains(&(0, 1)));
        assert!(neighbours.contains(&(1, 1)));
        assert!(neighbours.contains(&(1, 0)));
        assert_eq!(neighbours.len(), 3);

        let neighbours = Map::<10, 10>::neighbour_tile_coords(9, 9).collect::<Vec<_>>();

        assert!(neighbours.contains(&(8, 9)));
        assert!(neighbours.contains(&(8, 8)));
        assert!(neighbours.contains(&(9, 8)));
        assert_eq!(neighbours.len(), 3);

        let neighbours = Map::<10, 10>::neighbour_tile_coords(5, 5).collect::<Vec<_>>();

        assert!(neighbours.contains(&(4, 4)));
        assert!(neighbours.contains(&(4, 5)));
        assert!(neighbours.contains(&(4, 6)));
        assert!(neighbours.contains(&(5, 4)));
        assert!(neighbours.contains(&(5, 6)));
        assert!(neighbours.contains(&(6, 4)));
        assert!(neighbours.contains(&(6, 5)));
        assert!(neighbours.contains(&(6, 6)));
        assert_eq!(neighbours.len(), 8);

        let neighbours = dbg!(Map::<10, 1>::neighbour_tile_coords(1, 0).collect::<Vec<_>>());

        assert!(neighbours.contains(&(0, 0)));
        assert!(neighbours.contains(&(2, 0)));
        assert_eq!(neighbours.len(), 2);
    }

    fn all_tile_coords_gif<const WIDTH: usize, const HEIGHT: usize>(
    ) -> impl Iterator<Item = (usize, usize)> {
        (0..HEIGHT)
            .map(|y| (0..WIDTH).map(move |x| (x, y)))
            .flatten()
    }

    struct GifSetup<const WIDTH: usize, const HEIGHT: usize> {
        path: PathBuf,
        max_value: f32,
        min_value: f32,
        gradient: colorgrad::Gradient,
        data_getter: fn(&Map<WIDTH, HEIGHT>) -> [[f32; HEIGHT]; WIDTH],
    }

    fn create_map_gif<const WIDTH: usize, const HEIGHT: usize>(
        map: &mut Map<WIDTH, HEIGHT>,
        iterations: usize,
        frame_every_nth: usize,
        delta_time: f32,
        gif_setups: &[GifSetup<WIDTH, HEIGHT>],
    ) {
        let mut encoders = gif_setups
            .iter()
            .map(|setup| {
                let image = File::create(&setup.path).unwrap();
                let mut encoder =
                    gif::Encoder::new(image, WIDTH as u16, HEIGHT as u16, &[]).unwrap();
                encoder.set_repeat(gif::Repeat::Infinite).unwrap();
                encoder
            })
            .collect::<Vec<_>>();

        for i in 0..iterations {
            if i % frame_every_nth == 0 {
                for (setup, encoder) in gif_setups.iter().zip(encoders.iter_mut()) {
                    let data = (setup.data_getter)(&map);

                    let mut pixels = vec![128; WIDTH * HEIGHT * 3];
                    for (i, (x, y)) in all_tile_coords_gif::<WIDTH, HEIGHT>().enumerate() {
                        if data[x][y].is_nan() {
                            continue;
                        }

                        if data[x][y] < setup.min_value {
                            pixels[i * 3 + 0] = 0;
                            pixels[i * 3 + 1] = 0;
                            pixels[i * 3 + 2] = 0;
                        } else if data[x][y] > setup.max_value {
                            pixels[i * 3 + 0] = 255;
                            pixels[i * 3 + 1] = 255;
                            pixels[i * 3 + 2] = 255;
                        } else {
                            let fraction = (data[x][y] - setup.min_value)
                                / (setup.max_value - setup.min_value);
                            let [r, g, b, _] = setup.gradient.at(fraction as f64).to_rgba8();

                            pixels[i * 3 + 0] = r;
                            pixels[i * 3 + 1] = g;
                            pixels[i * 3 + 2] = b;
                        }
                    }
                    encoder
                        .write_frame(&gif::Frame::from_rgb(
                            WIDTH as u16,
                            HEIGHT as u16,
                            &mut pixels,
                        ))
                        .unwrap();
                }
            }

            map.simulate(delta_time)
        }
    }

    #[test]
    fn facing_rotation() {
        assert_eq!(Facing::North.rotate(Facing::North), Facing::North);
        assert_eq!(Facing::North.rotate(Facing::East), Facing::East);
        assert_eq!(Facing::North.rotate(Facing::South), Facing::South);
        assert_eq!(Facing::North.rotate(Facing::West), Facing::West);

        assert_eq!(Facing::East.rotate(Facing::North), Facing::East);
        assert_eq!(Facing::East.rotate(Facing::East), Facing::South);
        assert_eq!(Facing::East.rotate(Facing::South), Facing::West);
        assert_eq!(Facing::East.rotate(Facing::West), Facing::North);

        assert_eq!(Facing::South.rotate(Facing::North), Facing::South);
        assert_eq!(Facing::South.rotate(Facing::East), Facing::West);
        assert_eq!(Facing::South.rotate(Facing::South), Facing::North);
        assert_eq!(Facing::South.rotate(Facing::West), Facing::East);

        assert_eq!(Facing::West.rotate(Facing::North), Facing::West);
        assert_eq!(Facing::West.rotate(Facing::East), Facing::North);
        assert_eq!(Facing::West.rotate(Facing::South), Facing::East);
        assert_eq!(Facing::West.rotate(Facing::West), Facing::South);
    }

    #[test]
    #[rustfmt::skip]
    fn facing_move_coords_in_direction() {
        assert_eq!(Facing::North.move_coords_in_direction::<5, 10>(2, 0), None);
        assert_eq!(Facing::North.move_coords_in_direction::<5, 10>(0, 1), Some((0, 0)));
        assert_eq!(Facing::North.move_coords_in_direction::<5, 10>(4, 9), Some((4, 8)));

        assert_eq!(Facing::East.move_coords_in_direction::<5, 10>(4, 2), None);
        assert_eq!(Facing::East.move_coords_in_direction::<5, 10>(3, 1), Some((4, 1)));
        assert_eq!(Facing::East.move_coords_in_direction::<5, 10>(0, 9), Some((1, 9)));

        assert_eq!(Facing::South.move_coords_in_direction::<5, 10>(4, 9), None);
        assert_eq!(Facing::South.move_coords_in_direction::<5, 10>(0, 8), Some((0, 9)));
        assert_eq!(Facing::South.move_coords_in_direction::<5, 10>(2, 0), Some((2, 1)));

        assert_eq!(Facing::West.move_coords_in_direction::<5, 10>(0, 6), None);
        assert_eq!(Facing::West.move_coords_in_direction::<5, 10>(1, 1), Some((0, 1)));
        assert_eq!(Facing::West.move_coords_in_direction::<5, 10>(4, 9), Some((3, 9)));
    }

    #[test]
    fn facing_rotate_isize() {
        assert_eq!(Facing::North.rotate_isize_coords(0, 0), (0, 0));
        assert_eq!(Facing::East.rotate_isize_coords(0, 0), (0, 0));
        assert_eq!(Facing::South.rotate_isize_coords(0, 0), (0, 0));
        assert_eq!(Facing::West.rotate_isize_coords(0, 0), (0, 0));

        assert_eq!(Facing::North.rotate_isize_coords(2, 1), (2, 1));
        assert_eq!(Facing::East.rotate_isize_coords(2, 1), (-1, 2));
        assert_eq!(Facing::South.rotate_isize_coords(2, 1), (-2, -1));
        assert_eq!(Facing::West.rotate_isize_coords(2, 1), (1, -2));
    }

    #[test]
    fn facing_rotate_f32() {
        assert_relative_eq!(Facing::North.rotate_f32_coords(0.7, 0.6).0, (0.7, 0.6).0);
        assert_relative_eq!(Facing::North.rotate_f32_coords(0.7, 0.6).1, (0.7, 0.6).1);
        assert_relative_eq!(Facing::East.rotate_f32_coords(0.7, 0.6).0, (0.4, 0.7).0);
        assert_relative_eq!(Facing::East.rotate_f32_coords(0.7, 0.6).1, (0.4, 0.7).1);
        assert_relative_eq!(Facing::South.rotate_f32_coords(0.7, 0.6).0, (0.3, 0.4).0);
        assert_relative_eq!(Facing::South.rotate_f32_coords(0.7, 0.6).1, (0.3, 0.4).1);
        assert_relative_eq!(Facing::West.rotate_f32_coords(0.7, 0.6).0, (0.6, 0.3).0);
        assert_relative_eq!(Facing::West.rotate_f32_coords(0.7, 0.6).1, (0.6, 0.3).1);

        assert_relative_eq!(Facing::North.rotate_f32_coords(2.5, 1.5).0, (2.5, 1.5).0);
        assert_relative_eq!(Facing::North.rotate_f32_coords(2.5, 1.5).1, (2.5, 1.5).1);
        assert_relative_eq!(Facing::East.rotate_f32_coords(2.5, 1.5).0, (-0.5, 2.5).0);
        assert_relative_eq!(Facing::East.rotate_f32_coords(2.5, 1.5).1, (-0.5, 2.5).1);
        assert_relative_eq!(Facing::South.rotate_f32_coords(2.5, 1.5).0, (-1.5, -0.5).0);
        assert_relative_eq!(Facing::South.rotate_f32_coords(2.5, 1.5).1, (-1.5, -0.5).1);
        assert_relative_eq!(Facing::West.rotate_f32_coords(2.5, 1.5).0, (1.5, -1.5).0);
        assert_relative_eq!(Facing::West.rotate_f32_coords(2.5, 1.5).1, (1.5, -1.5).1);
    }

    #[test]
    fn air_pressure() {
        std::thread::Builder::new()
            .name("TestThread".into())
            .stack_size(16 * 1024 * 1024)
            .spawn(|| {
                let mut map = Map::<20, 10>::new_default();
                map.push_object::<EnvironmentObject>(AirLeveler {
                    x: 0,
                    y: 9,
                    nitrogen: 0.79 / 2.0,
                    oxygen: 0.21 / 2.0,
                    fumes: 0.0,
                });
                map.push_object::<EnvironmentObject>(AirLeveler {
                    x: 9,
                    y: 0,
                    nitrogen: 0.79,
                    oxygen: 0.21,
                    fumes: 0.0,
                });
                map.push_object::<EnvironmentObject>(OxygenUser {
                    x: 5,
                    y: 5,
                    change_per_sec: 0.0001,
                });
                map.push_object::<EnvironmentObject>(OxygenUser {
                    x: 18,
                    y: 2,
                    change_per_sec: 0.0001,
                });

                map.push_object::<EnvironmentObject>(LiquidLeveler {
                    x: 19,
                    y: 0,
                    target: LiquidData::Water { level: 1.0 },
                });
                map.push_object::<EnvironmentObject>(LiquidLeveler {
                    x: 19,
                    y: 9,
                    target: LiquidData::Lava { level: 1.1 },
                });
                map.push_object::<EnvironmentObject>(AirPusher {
                    x: 18,
                    y: 4,
                    direction: Facing::South,
                    amount: 2.0,
                });
                map.push_object::<EnvironmentObject>(AirPusher {
                    x: 16,
                    y: 8,
                    direction: Facing::West,
                    amount: 2.0,
                });
                map.push_object::<EnvironmentObject>(AirPusher {
                    x: 10,
                    y: 8,
                    direction: Facing::West,
                    amount: 2.0,
                });

                for (x, y) in Map::<20, 10>::all_tile_coords().filter(|(x, _)| *x >= 10) {
                    map.tiles[x][y].ground_level = -1.1;
                }

                for i in 1..8 {
                    map.tiles[1][i] = Tile {
                        tile_type: TileType::Wall,
                        ..Default::default()
                    };
                }
                for i in 1..8 {
                    map.tiles[i][1] = Tile {
                        tile_type: TileType::Wall,
                        ..Default::default()
                    };
                }
                for i in 3..8 {
                    map.tiles[3][i] = Tile {
                        tile_type: TileType::Wall,
                        ..Default::default()
                    };
                }
                for i in 3..8 {
                    map.tiles[i][3] = Tile {
                        tile_type: TileType::Wall,
                        ..Default::default()
                    };
                }
                for i in 3..7 {
                    map.tiles[7][i] = Tile {
                        tile_type: TileType::Wall,
                        ..Default::default()
                    };
                }
                for i in 3..6 {
                    map.tiles[i][7] = Tile {
                        tile_type: TileType::Wall,
                        ..Default::default()
                    };
                }

                create_map_gif(
                    &mut map,
                    100000,
                    100,
                    0.05,
                    &[
                        GifSetup {
                            path: "target/total_air_pressure.gif".into(),
                            max_value: 1.02,
                            min_value: 0.00,
                            gradient: colorgrad::viridis(),
                            data_getter: |map| map.collect_air_pressure_map(),
                        },
                        GifSetup {
                            path: "target/oxygen.gif".into(),
                            max_value: 0.21,
                            min_value: 0.10,
                            gradient: colorgrad::viridis(),
                            data_getter: |map| map.collect_oxygen_map(),
                        },
                        GifSetup {
                            path: "target/fumes.gif".into(),
                            max_value: 0.005,
                            min_value: 0.00,
                            gradient: colorgrad::viridis(),
                            data_getter: |map| map.collect_fumes_map(),
                        },
                        GifSetup {
                            path: "target/water.gif".into(),
                            max_value: 3.00,
                            min_value: 0.00,
                            gradient: colorgrad::viridis(),
                            data_getter: |map| map.collect_liquids_map::<Water>(),
                        },
                        GifSetup {
                            path: "target/lava.gif".into(),
                            max_value: 3.00,
                            min_value: 0.00,
                            gradient: colorgrad::viridis(),
                            data_getter: |map| map.collect_liquids_map::<Lava>(),
                        },
                        GifSetup {
                            path: "target/surface.gif".into(),
                            max_value: 1.00,
                            min_value: -1.1,
                            gradient: colorgrad::viridis(),
                            data_getter: |map| map.collect_surface_level_map(),
                        },
                        GifSetup {
                            path: "target/ground_level.gif".into(),
                            max_value: 1.00,
                            min_value: -1.1,
                            gradient: colorgrad::viridis(),
                            data_getter: |map| map.collect_ground_level_map(),
                        },
                    ],
                );
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn all_tile_coords() {
        let coords = Map::<1, 2>::all_tile_coords().collect::<Vec<_>>();
        assert_eq!(coords, vec![(0, 0), (0, 1)])
    }
}
