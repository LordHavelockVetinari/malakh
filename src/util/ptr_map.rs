use std::collections::hash_map::Entry;
use std::collections::{self, HashMap};
use std::hash::{self, Hash};
use std::ops::Index;
use std::rc::Rc;

#[derive(Debug)]
pub struct RcHashWrap<T: ?Sized>(Rc<T>);

impl<T: ?Sized> Clone for RcHashWrap<T> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<T: ?Sized> PartialEq for RcHashWrap<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<T: ?Sized> Eq for RcHashWrap<T> {}

impl<T: ?Sized> Hash for RcHashWrap<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.0).hash(state);
    }
}

#[derive(Debug)]
pub struct PtrMap<K: ?Sized, V>(pub HashMap<RcHashWrap<K>, V>);

impl<K: ?Sized, V> PtrMap<K, V> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn insert(&mut self, key: Rc<K>, value: V) -> Option<V> {
        self.0.insert(RcHashWrap(key), value)
    }

    pub fn insert_new(&mut self, key: Rc<K>, value: V) {
        match self.0.entry(RcHashWrap(key)) {
            Entry::Occupied(_) => panic!("PtrMap key already present"),
            Entry::Vacant(entry) => {
                entry.insert(value);
            }
        }
    }

    pub fn get(&self, key: Rc<K>) -> Option<&V> {
        self.0.get(&RcHashWrap(key))
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        IntoIterator::into_iter(self)
    }
}

impl<K: ?Sized, V: Clone> Clone for PtrMap<K, V> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<'a, K: ?Sized, V> IntoIterator for &'a PtrMap<K, V> {
    type Item = (&'a Rc<K>, &'a V);

    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        Iter(self.0.iter())
    }
}

pub struct Iter<'a, K: ?Sized, V>(pub collections::hash_map::Iter<'a, RcHashWrap<K>, V>);

impl<'a, K: ?Sized, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a Rc<K>, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(RcHashWrap(k), v)| (k, v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<K: ?Sized, V> IntoIterator for PtrMap<K, V> {
    type Item = (Rc<K>, V);

    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.0.into_iter())
    }
}

pub struct IntoIter<K: ?Sized, V>(pub collections::hash_map::IntoIter<RcHashWrap<K>, V>);

impl<K: ?Sized, V> Iterator for IntoIter<K, V> {
    type Item = (Rc<K>, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(RcHashWrap(k), v)| (k, v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<K: ?Sized, V> Index<&Rc<K>> for PtrMap<K, V> {
    type Output = V;

    fn index(&self, index: &Rc<K>) -> &Self::Output {
        self.get(Rc::clone(index)).expect("key not found in PtrMap")
    }
}
