use std::{
    any::{type_name, TypeId},
    ops::{Deref, DerefMut},
};

use crate::{
    air::{AirLeveler, AirPusher, OxygenUser},
    liquids::LiquidLeveler,
    Map,
};

use self::{building::Building, characters::Character, environment_object::EnvironmentObject};

pub mod building;
pub mod characters;
pub mod environment_object;
mod object_id;

pub use object_id::ObjectId;

const ENVIRONMENT_OBJECT: TypeId = TypeId::of::<EnvironmentObject>();
const BUILDING_OBJECT: TypeId = TypeId::of::<Building>();
const CHARACTER_OBJECT: TypeId = TypeId::of::<Character>();

/// Get an `Iterator<item = &dyn ObjectProperties>` containing all map objects.
/// This is a macro because a function would borrow the whole map object instead of just the object fields
#[macro_export]
macro_rules! all_map_objects {
    ($map:ident) => {{
        use crate::ObjectProperties;
        use std::ops::Deref;

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

impl<const WIDTH: usize, const HEIGHT: usize> Map<WIDTH, HEIGHT> {
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
        let vec = self.get_vec_of_type::<T>();
        let object_index = vec.binary_search_by_key(&id, |obj| obj.id()).ok()?;
        Some(&vec[object_index])
    }

    pub fn get_object_mut<T: ObjectProperties>(
        &mut self,
        id: ObjectId<T>,
    ) -> Option<&mut Object<T>> {
        let vec = self.get_vec_of_type_mut::<T>();
        let object_index = vec.binary_search_by_key(&id, |obj| obj.id()).ok()?;
        Some(&mut vec[object_index])
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
}

#[derive(Debug)]
pub struct Object<T: ObjectProperties> {
    pub id: usize,
    pub object: T,
}

impl<T: ObjectProperties> Object<T> {
    pub fn id(&self) -> ObjectId<T> {
        ObjectId::new(self.id)
    }
}

impl<T: ObjectProperties> Deref for Object<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl<T: ObjectProperties> DerefMut for Object<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.object
    }
}

pub trait ObjectProperties: 'static {
    fn air_levelers(&self) -> Vec<AirLeveler<usize>> {
        Vec::new()
    }
    fn oxygen_users(&self) -> Vec<OxygenUser<usize>> {
        Vec::new()
    }
    fn liquid_levelers(&self) -> Vec<LiquidLeveler<usize>> {
        Vec::new()
    }
    fn air_pushers(&self) -> Vec<AirPusher<usize>> {
        Vec::new()
    }
}
