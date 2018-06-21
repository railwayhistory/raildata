//! A set with an optimazation for holding only a single element.

use std::mem;
use std::hash::Hash;
use std::collections::hash_set;
use std::collections::HashSet;
use ::load::yaml::{FromYaml, Value};
use ::load::report::{Failed, PathReporter};
use super::marked::Location;


//------------ Set -----------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Set<T: Hash + Eq> {
    inner: Inner<T, HashSet<T>>,
    location: Location
}

#[derive(Clone, Debug)]
enum Inner<O, M> {
    Empty,
    One(O),
    Many(M),
}

impl<T: Hash + Eq> Set<T> {
    pub fn new() -> Self {
        Set { inner: Inner::Empty, location: Location::default() }
    }

    pub fn empty(location: Location) -> Self {
        Set { inner: Inner::Empty, location }
    }

    pub fn one(value: T, location: Location) -> Self {
        Set { inner: Inner::One(value), location }
    }

    pub fn many(set: HashSet<T>, location: Location) -> Self {
        Set { inner: Inner::Many(set), location }
    }

    pub fn location(&self) -> Location {
        self.location
    }

    pub fn insert(&mut self, value: T) -> bool {
        if let Inner::Many(ref mut set) = self.inner {
            set.insert(value)
        }
        else if let Inner::Empty = self.inner {
            self.inner = Inner::One(value);
            true
        }
        else {
            let mut set = HashSet::new();
            set.insert(match mem::replace(&mut self.inner, Inner::Empty) {
                Inner::One(first) => first,
                _ => unreachable!()
            });
            self.inner = Inner::Many(set);
            true
        }
    }

    pub fn contains(&self, value: &T) -> bool {
        match self.inner {
            Inner::Empty => false,
            Inner::One(ref some) => value == some,
            Inner::Many(ref set) => set.contains(value)
        }
    }

    pub fn iter(&self) -> Iter<T> {
        Iter::new(self)
    }

    pub fn len(&self) -> usize {
        match self.inner {
            Inner::Empty => 0,
            Inner::One(_) => 1,
            Inner::Many(ref set) => set.len()
        }
    }

    pub fn is_empty(&self) -> bool {
        match self.inner {
            Inner::Empty => true,
            Inner::One(_) => false,
            Inner::Many(ref set) => set.is_empty()
        }
    }
}

impl<T: Hash + Eq> Default for Set<T> {
    fn default() -> Self {
        Set::new()
    }
}

impl<C, T: FromYaml<C> + Hash + Eq> FromYaml<C> for Set<T> {
    fn from_yaml(
        value: Value,
        context: &mut C,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        match value.try_into_sequence() {
            Ok(mut seq) => {
                if seq.is_empty() {
                    Ok(Set::empty(seq.location()))
                }
                else if seq.len() == 1 {
                    T::from_yaml(seq.pop().unwrap(), context, report)
                             .map(|value| Set::one(value, seq.location()))
                }
                else {
                    let mut res = HashSet::with_capacity(seq.len());
                    let mut err = false;
                    let location = seq.location();
                    for item in seq {
                        if let Ok(item ) = T::from_yaml(item, context, report) {
                            res.insert(item);
                        }
                        else {
                            err = true
                        }
                    }
                    if err {
                        return Err(Failed)
                    }
                    Ok(Set::many(res, location))
                }
            }
            Err(value) => {
                let location = value.location();
                T::from_yaml(value, context, report).map(|value| {
                    Set::one(value, location)
                })
            }
        }
    }
}


//------------ Iter ----------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Iter<'a, T: 'a>(Inner<Option<&'a T>, hash_set::Iter<'a, T>>);

impl<'a, T: Hash + Eq> Iter<'a, T> {
    fn new(set: &'a Set<T>) -> Self {
        match set.inner {
            Inner::Empty => Iter(Inner::Empty),
            Inner::One(ref item) => Iter(Inner::One(Some(item))),
            Inner::Many(ref set) => Iter(Inner::Many(set.iter())),
        }
    }
}

impl<'a, T: 'a> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        match self.0 {
            Inner::Empty => None,
            Inner::One(ref mut item) => item.take(),
            Inner::Many(ref mut set) => set.next(),
        }
    }
}

