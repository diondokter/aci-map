use self::{building::Building, characters::Character, environment_object::EnvironmentObject};
use crate::{
    air::{AirLeveler, AirPusher, OxygenUser},
    liquids::LiquidLeveler,
};
use std::{
    any::{type_name, TypeId},
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};

pub mod building;
pub mod characters;
pub mod environment_object;
mod object_id;

pub use object_id::ObjectId;

const ENVIRONMENT_OBJECT: TypeId = TypeId::of::<EnvironmentObject>();
const BUILDING_OBJECT: TypeId = TypeId::of::<Building>();
const CHARACTER_OBJECT: TypeId = TypeId::of::<Character>();

#[derive(Debug)]
pub struct Objects {
    next_object_id: u32,
    object_sync: ObjectSync,

    // These object arrays must be in order of object ID
    environment_objects: Vec<Object<EnvironmentObject>>,
    buildings: Vec<Object<Building>>,
    characters: Vec<Object<Character>>,
}

impl Objects {
    pub const fn new() -> Self {
        Self {
            next_object_id: 0,
            object_sync: ObjectSync::new(),
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
            object: UnsafeCell::new(object),
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

    pub fn get_object<T: ObjectProperties>(&self, id: ObjectId<T>) -> Option<LockedObject<'_, T>> {
        let vec = self.get_vec_of_type::<T>();
        let object_index = vec.binary_search_by_key(&id, |obj| obj.id()).ok()?;
        Some(LockedObject::new(&vec[object_index], &self.object_sync))
    }

    pub fn get_object_mut<T: ObjectProperties>(
        &self,
        id: ObjectId<T>,
    ) -> Option<LockedObjectMut<T>> {
        let vec = self.get_vec_of_type::<T>();
        let object_index = vec.binary_search_by_key(&id, |obj| obj.id()).ok()?;
        Some(LockedObjectMut::new(&vec[object_index], &self.object_sync))
    }

    pub fn get_all_objects(&self) -> impl Iterator<Item = LockedObject<'_, dyn ObjectProperties>> {
        let eo = self
            .environment_objects
            .iter()
            .map(|val| LockedObject::new_dyn(val, &self.object_sync));
        let b = self
            .buildings
            .iter()
            .map(|val| LockedObject::new_dyn(val, &self.object_sync));
        let c = self
            .characters
            .iter()
            .map(|val| LockedObject::new_dyn(val, &self.object_sync));

        eo.chain(b).chain(c)
    }

    pub fn get_all_objects_mut(
        &self,
    ) -> impl Iterator<Item = LockedObjectMut<'_, dyn ObjectProperties>> {
        let eo = self
            .environment_objects
            .iter()
            .map(|val| LockedObjectMut::new_dyn(val, &self.object_sync));
        let b = self
            .buildings
            .iter()
            .map(|val| LockedObjectMut::new_dyn(val, &self.object_sync));
        let c = self
            .characters
            .iter()
            .map(|val| LockedObjectMut::new_dyn(val, &self.object_sync));

        eo.chain(b).chain(c)
    }

    pub fn get_objects<T: ObjectProperties>(&self) -> impl Iterator<Item = LockedObject<'_, T>> {
        self.get_vec_of_type()
            .iter()
            .map(|obj| LockedObject::new(obj, &self.object_sync))
    }

    pub fn get_objects_mut<T: ObjectProperties>(
        &self,
    ) -> impl Iterator<Item = LockedObjectMut<'_, T>> {
        self.get_vec_of_type()
            .iter()
            .map(|obj| LockedObjectMut::new(obj, &self.object_sync))
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
pub(crate) struct ObjectSync {}

impl ObjectSync {
    pub const fn new() -> Self {
        Self {}
    }

    fn take_read_access<T>(&self, object_id: ObjectId<T>) {
        todo!()
    }

    fn free_read_access<T>(&self, object_id: ObjectId<T>) {
        todo!()
    }

    fn take_write_access<T>(&self, object_id: ObjectId<T>) {
        todo!()
    }

    fn free_write_access<T>(&self, object_id: ObjectId<T>) {
        todo!()
    }
}

#[derive(Debug)]
pub struct Object<T: ObjectProperties> {
    id: u32,
    object: UnsafeCell<T>,
}

impl<T: ObjectProperties> Object<T> {
    pub fn id(&self) -> ObjectId<T> {
        ObjectId::new(self.id)
    }
}

unsafe impl<T: ObjectProperties + Sync> Sync for Object<T> {}
unsafe impl<T: ObjectProperties + Send> Send for Object<T> {}

#[derive(Debug)]
pub struct LockedObject<'o, T: ObjectProperties + ?Sized> {
    id: ObjectId<()>,
    object: &'o T,
    object_sync: &'o ObjectSync,
}

impl<'o, T: ObjectProperties> LockedObject<'o, T> {
    pub(crate) fn new(object: &'o Object<T>, object_sync: &'o ObjectSync) -> Self {
        object_sync.take_read_access(object.id());
        Self {
            id: object.id().cast(),
            object: unsafe { &*object.object.get() },
            object_sync,
        }
    }

    pub fn id(&self) -> ObjectId<T> {
        self.id.cast()
    }
}

impl<'o> LockedObject<'o, dyn ObjectProperties> {
    pub(crate) fn new_dyn<T: ObjectProperties>(
        object: &'o Object<T>,
        object_sync: &'o ObjectSync,
    ) -> Self {
        object_sync.take_read_access(object.id());
        Self {
            id: object.id().cast(),
            object: unsafe { &*object.object.get() },
            object_sync,
        }
    }
}

impl<'o, T: ObjectProperties + ?Sized> Deref for LockedObject<'o, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl<'o, T: ObjectProperties + ?Sized> Drop for LockedObject<'o, T> {
    fn drop(&mut self) {
        self.object_sync.free_read_access(self.id);
    }
}

#[derive(Debug)]
pub struct LockedObjectMut<'o, T: ObjectProperties + ?Sized> {
    id: ObjectId<()>,
    object: &'o mut T,
    object_sync: &'o ObjectSync,
}

impl<'o, T: ObjectProperties> LockedObjectMut<'o, T> {
    pub(crate) fn new(object: &'o Object<T>, object_sync: &'o ObjectSync) -> Self {
        object_sync.take_write_access(object.id());
        Self {
            id: object.id().cast(),
            object: unsafe { &mut *object.object.get() },
            object_sync,
        }
    }

    pub fn id(&self) -> ObjectId<T> {
        self.id.cast()
    }
}

impl<'o> LockedObjectMut<'o, dyn ObjectProperties> {
    pub(crate) fn new_dyn<T: ObjectProperties>(
        object: &'o Object<T>,
        object_sync: &'o ObjectSync,
    ) -> Self {
        object_sync.take_write_access(object.id());
        Self {
            id: object.id().cast(),
            object: unsafe { &mut *object.object.get() },
            object_sync,
        }
    }
}

impl<'o, T: ObjectProperties + ?Sized> Deref for LockedObjectMut<'o, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl<'o, T: ObjectProperties + ?Sized> DerefMut for LockedObjectMut<'o, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.object
    }
}

impl<'o, T: ObjectProperties + ?Sized> Drop for LockedObjectMut<'o, T> {
    fn drop(&mut self) {
        self.object_sync.free_write_access(self.id);
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
