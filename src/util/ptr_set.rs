use std::rc::Rc;

use crate::util::ptr_map::{self, PtrMap};

#[derive(Debug)]
pub struct PtrSet<T: ?Sized>(pub PtrMap<T, ()>);

pub struct Iter<'a, T: ?Sized>(ptr_map::Iter<'a, T, ()>);

pub struct Drain<'a, T: ?Sized>(ptr_map::Drain<'a, T, ()>);

impl<T: ?Sized> PtrSet<T> {
    pub fn new() -> Self {
        Self(PtrMap::new())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn insert(&mut self, x: Rc<T>) -> bool {
        self.0.insert(x, ()).is_none()
    }

    pub fn remove(&mut self, x: &Rc<T>) -> bool {
        self.0.remove(x).is_some()
    }

    pub fn contains(&self, x: Rc<T>) -> bool {
        self.0.get(x).is_some()
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter(self.0.iter())
    }

    pub fn drain(&mut self) -> Drain<'_, T> {
        Drain(self.0.drain())
    }
}

impl<T: ?Sized> Default for PtrSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T: ?Sized> IntoIterator for &'a PtrSet<T> {
    type IntoIter = Iter<'a, T>;
    type Item = &'a Rc<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T: ?Sized> Iterator for Iter<'a, T> {
    type Item = &'a Rc<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(x, ())| x)
    }
}

impl<'a, T: ?Sized> Iterator for Drain<'a, T> {
    type Item = Rc<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(x, ())| x)
    }
}
