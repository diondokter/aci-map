use self::{building::Building, characters::Character, environment_object::EnvironmentObject};
use crate::{
    air::{AirLeveler, AirPusher, OxygenUser},
    liquids::LiquidLeveler,
};
use std::{
    any::{type_name, TypeId},
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicU32, Ordering},
};

pub mod building;
pub mod characters;
pub mod environment_object;
mod object_id;

pub use object_id::ObjectId;

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

        self.object_sync.push_object(object_id.cast());

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

        self.object_sync.remove_object(id.cast());
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
            o if o == TypeId::of::<EnvironmentObject>() => unsafe {
                std::mem::transmute(&self.environment_objects)
            },
            o if o == TypeId::of::<Building>() => unsafe { std::mem::transmute(&self.buildings) },
            o if o == TypeId::of::<Character>() => unsafe { std::mem::transmute(&self.characters) },
            _ => unreachable!(),
        }
    }

    fn get_vec_of_type_mut<T: ObjectProperties>(&mut self) -> &mut Vec<Object<T>> {
        match TypeId::of::<T>() {
            o if o == TypeId::of::<EnvironmentObject>() => unsafe {
                std::mem::transmute(&mut self.environment_objects)
            },
            o if o == TypeId::of::<Building>() => unsafe {
                std::mem::transmute(&mut self.buildings)
            },
            o if o == TypeId::of::<Character>() => unsafe {
                std::mem::transmute(&mut self.characters)
            },
            _ => unreachable!("{} is not covered", type_name::<T>()),
        }
    }
}

#[derive(Debug)]
pub(crate) struct ObjectSync {
    states: Vec<(ObjectId<()>, SyncState)>,
}

impl ObjectSync {
    pub const fn new() -> Self {
        Self { states: Vec::new() }
    }

    fn find_index(&self, object_id: ObjectId<()>) -> Result<usize, usize> {
        self.states.binary_search_by_key(&object_id, |(id, _)| *id)
    }

    pub fn push_object(&mut self, object_id: ObjectId<()>) {
        let insert_index = self.find_index(object_id).unwrap_err();
        self.states
            .insert(insert_index, (object_id, SyncState::new()));
    }

    pub fn remove_object(&mut self, object_id: ObjectId<()>) {
        let remove_index = self.find_index(object_id).unwrap();
        self.states.remove(remove_index);
    }

    pub fn take_read_access(&self, object_id: ObjectId<()>) {
        let index = self.find_index(object_id).unwrap();
        self.states[index].1.spin_take_read();
    }

    // Safety: Must have taken first
    pub unsafe fn free_read_access(&self, object_id: ObjectId<()>) {
        let index = self.find_index(object_id).unwrap();
        self.states[index].1.release_read();
    }

    pub fn take_write_access(&self, object_id: ObjectId<()>) {
        let index = self.find_index(object_id).unwrap();
        self.states[index].1.spin_take_write();
    }

    // Safety: Must have taken first
    pub unsafe fn free_write_access(&self, object_id: ObjectId<()>) {
        let index = self.find_index(object_id).unwrap();
        self.states[index].1.release_write();
    }
}

#[derive(Debug)]
struct SyncState(AtomicU32);

impl SyncState {
    pub const fn new() -> Self {
        Self(AtomicU32::new(0))
    }

    const WRITER: u32 = 1;
    const READER: u32 = 2;

    pub fn spin_take_read(&self) {
        loop {
            let previous_value = self.0.fetch_add(Self::READER, Ordering::AcqRel);
            if previous_value & Self::WRITER > 0 {
                // A writer is active. We need to release again
                self.0.fetch_sub(Self::READER, Ordering::AcqRel);
                std::hint::spin_loop();
            } else {
                break;
            }
        }
    }

    pub fn release_read(&self) {
        self.0.fetch_sub(Self::READER, Ordering::AcqRel);
    }

    pub fn spin_take_write(&self) {
        loop {
            if self
                .0
                .compare_exchange_weak(0, Self::WRITER, Ordering::Acquire, Ordering::Relaxed)
                .is_err()
            {
                std::hint::spin_loop();
            } else {
                break;
            }
        }
    }

    pub fn release_write(&self) {
        self.0.fetch_sub(Self::WRITER, Ordering::AcqRel);
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
        object_sync.take_read_access(object.id().cast());
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
        object_sync.take_read_access(object.id().cast());
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
        self.object
    }
}

impl<'o, T: ObjectProperties + ?Sized> Drop for LockedObject<'o, T> {
    fn drop(&mut self) {
        unsafe {
            self.object_sync.free_read_access(self.id);
        }
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
        object_sync.take_write_access(object.id().cast());
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
        object_sync.take_write_access(object.id().cast());
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
        self.object
    }
}

impl<'o, T: ObjectProperties + ?Sized> DerefMut for LockedObjectMut<'o, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.object
    }
}

impl<'o, T: ObjectProperties + ?Sized> Drop for LockedObjectMut<'o, T> {
    fn drop(&mut self) {
        unsafe {
            self.object_sync.free_write_access(self.id);
        }
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

#[cfg(test)]
mod tests {
    use std::{sync::Mutex, time::Duration};

    use super::*;

    #[test]
    fn spinlock() {
        let state = SyncState::new();

        enum Event {
            WriteStart,
            WriteEnd,
            ReadStart,
            ReadEnd,
        }

        let events = Mutex::new(Vec::new());

        std::thread::scope(|s| {
            for i in 0..500 {
                if i % 10 == 0 {
                    s.spawn(|| {
                        state.spin_take_write();
                        events.lock().unwrap().push(Event::WriteStart);
                        std::thread::sleep(Duration::from_micros(100));
                        events.lock().unwrap().push(Event::WriteEnd);
                        state.release_write();
                    });
                } else {
                    s.spawn(|| {
                        state.spin_take_read();
                        events.lock().unwrap().push(Event::ReadStart);
                        std::thread::sleep(Duration::from_micros(50));
                        events.lock().unwrap().push(Event::ReadEnd);
                        state.release_read();
                    });
                }
            }
        });

        let mut reads_active = 0;
        let mut writes_active = 0;

        for event in events.into_inner().unwrap().into_iter() {
            match event {
                Event::WriteStart => writes_active += 1,
                Event::WriteEnd => writes_active -= 1,
                Event::ReadStart => reads_active += 1,
                Event::ReadEnd => reads_active -= 1,
            }

            println!("{reads_active}, {writes_active}");

            assert!(
                reads_active >= 0 && writes_active == 0 || reads_active == 0 && writes_active == 1
            );
        }
    }
}
