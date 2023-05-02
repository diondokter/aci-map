use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};
use crate::{AirLeveler, LiquidLeveler, OxygenUser, AirPusher};

#[derive(Debug, Clone, Copy)]
pub struct ObjectId<T> {
    id: usize,
    _phantom: PhantomData<T>,
}

impl<T> ObjectId<T> {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            _phantom: PhantomData,
        }
    }
}

impl<T: ObjectProperties> ObjectId<T> {
    pub fn cast(self) -> ObjectId<()> {
        ObjectId {
            id: self.id,
            _phantom: PhantomData,
        }
    }
}

impl ObjectId<()> {
    pub fn cast<T>(self) -> ObjectId<T> {
        ObjectId {
            id: self.id,
            _phantom: PhantomData,
        }
    }
}

impl<T> PartialEq for ObjectId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for ObjectId<T> {}

#[derive(Debug)]
pub struct Object<T: ObjectProperties> {
    pub(crate) id: usize,
    pub(crate) object: T,
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
    fn oxygen_users(&self) -> Vec<OxygenUser<usize>>{
        Vec::new()
    }
    fn liquid_levelers(&self) -> Vec<LiquidLeveler<usize>> {
        Vec::new()
    }
    fn air_pushers(&self) -> Vec<AirPusher<usize>> {
        Vec::new()
    }
}
