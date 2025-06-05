use std::ops::Deref;

#[repr(transparent)]
pub struct Synced<T>(T);

impl<T> Synced<T> {
    pub fn new(obj: T) -> Self {
        Self(obj)
    }
}

impl<T> Deref for Synced<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

unsafe impl<T> Send for Synced<T> {}
unsafe impl<T> Sync for Synced<T> {}
