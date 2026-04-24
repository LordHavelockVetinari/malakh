use std::rc::Rc;

use crate::util::ptr_map::PtrMap;

#[derive(Debug)]
pub struct PtrSet<T: ?Sized>(pub PtrMap<T, ()>);

impl<T: ?Sized> PtrSet<T> {
    pub fn new() -> Self {
        Self(PtrMap::new())
    }

    pub fn insert(&mut self, x: Rc<T>) -> bool {
        self.0.insert(x, ()).is_none()
    }

    pub fn contains(&self, x: Rc<T>) -> bool {
        self.0.get(x).is_some()
    }
}

impl<T: ?Sized> Default for PtrSet<T> {
    fn default() -> Self {
        Self::new()
    }
}
