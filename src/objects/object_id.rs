use super::ObjectProperties;
use std::{any::type_name, marker::PhantomData};

pub struct ObjectId<T> {
    id: usize,
    _phantom: PhantomData<T>,
}

impl<T> Ord for ObjectId<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl<T> PartialOrd for ObjectId<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl<T> Copy for ObjectId<T> {}

impl<T> Clone for ObjectId<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            _phantom: self._phantom.clone(),
        }
    }
}

impl<T> std::fmt::Debug for ObjectId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ObjectId")
            .field("type", &type_name::<T>())
            .field("id", &self.id)
            .finish()
    }
}

impl<T> PartialEq for ObjectId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for ObjectId<T> {}

impl<T> ObjectId<T> {
    pub(crate) fn new(id: usize) -> Self {
        Self {
            id,
            _phantom: PhantomData,
        }
    }
}

impl<T: ObjectProperties> ObjectId<T> {
    pub(crate) fn cast(self) -> ObjectId<()> {
        ObjectId {
            id: self.id,
            _phantom: PhantomData,
        }
    }
}

impl ObjectId<()> {
    pub(crate) fn cast<T: ObjectProperties>(self) -> ObjectId<T> {
        ObjectId {
            id: self.id,
            _phantom: PhantomData,
        }
    }
}
