use std::{marker::PhantomData, ops::{Deref, DerefMut}};

#[derive(Debug)]
pub struct ObjectId<T> {
    id: usize,
    _phantom: PhantomData<T>,
}

impl<T> PartialEq for ObjectId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for ObjectId<T> {}

#[derive(Debug)]
pub struct Object<T: 'static> {
    pub(crate) id: usize,
    pub(crate) object: T,
}

impl<T: 'static> Object<T> {
    pub fn id(&self) -> ObjectId<T> {
        ObjectId { id: self.id, _phantom: PhantomData::default() }
    }
}

impl<T: 'static> Deref for Object<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl<T: 'static> DerefMut for Object<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.object
    }
}
