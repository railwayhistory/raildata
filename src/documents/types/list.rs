use std::{fmt, mem, slice};
use ::load::yaml::Value;
use ::load::construct::{Constructable, Context, Failed};
use super::marked::Location;


//------------ List ----------------------------------------------------------

#[derive(Clone, Debug)]
pub struct List<T> {
    inner: Inner<T, Vec<T>>,
    location: Location,
}

#[derive(Clone, Debug)]
enum Inner<O, M> {
    Empty,
    One(O),
    Many(M),
}

impl<T> List<T> {
    pub fn new() -> Self {
        List { inner: Inner::Empty, location: Location::default() }
    }

    pub fn location(&self) -> Location {
        self.location
    }

    pub fn push(&mut self, item: T) {
        if let Inner::Many(ref mut vec) = self.inner {
            vec.push(item)
        }
        else {
            self.inner = match mem::replace(&mut self.inner, Inner::Empty) {
                Inner::Empty => Inner::One(item),
                Inner::One(first) => Inner::Many(vec![first, item]),
                _ => unreachable!()
            };
        }
    }

    pub fn iter(&self) -> Iter<T> {
        Iter::new(self)
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut::new(self)
    }

    pub fn len(&self) -> usize {
        match self.inner {
            Inner::Empty => 0,
            Inner::One(_) => 1,
            Inner::Many(ref vec) => vec.len()
        }
    }

    pub fn is_empty(&self) -> bool {
        match self.inner {
            Inner::Empty => true,
            Inner::One(_) => false,
            Inner::Many(ref vec) => vec.is_empty(),
        }
    }
}

impl<T> Default for List<T> {
    fn default() -> Self {
        List { inner: Inner::Empty, location: Location::default() }
    }
}

impl<T> From<Option<List<T>>> for List<T> {
    fn from(list: Option<List<T>>) -> Self {
        match list {
            Some(list) => list,
            None => List::default()
        }
    }
}

impl<T: Constructable> Constructable for List<T> {
    fn construct<C>(value: Value, context: &mut C) -> Result<Self, Failed>
                 where C: Context {
        let location = value.location();
        let inner = match value.try_into_sequence() {
            Ok(mut seq) => {
                if seq.is_empty() {
                    context.push_error((ListError::Empty, location));
                    return Err(Failed)
                }
                else if seq.len() == 1 {
                    T::construct(seq.pop().unwrap(), context).map(Inner::One)?
                }
                else {
                    let mut res = Vec::with_capacity(seq.len());
                    let mut err = false;
                    for item in seq.into_value() {
                        if let Ok(item) = T::construct(item, context) {
                            res.push(item)
                        }
                        else {
                            err = true
                        }
                    }
                    if err { 
                        return Err(Failed)
                    }
                    Inner::Many(res)
                }
            }
            Err(value) => T::construct(value, context).map(Inner::One)?
        };
        Ok(List { inner, location })
    }
}


impl<'a, T> IntoIterator for &'a List<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}

impl<'a, T> IntoIterator for &'a mut List<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        IterMut::new(self)
    }
}


//------------ Iter ----------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Iter<'a, T: 'a>(Inner<Option<&'a T>, slice::Iter<'a, T>>);

impl<'a, T: 'a> Iter<'a, T> {
    fn new(list: &'a List<T>) -> Self {
        match list.inner {
            Inner::Empty => Iter(Inner::Empty),
            Inner::One(ref item) => Iter(Inner::One(Some(item))),
            Inner::Many(ref vec) => Iter(Inner::Many(vec.iter()))
        }
    }
}

impl<'a, T: 'a> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        match self.0 {
            Inner::Empty => None,
            Inner::One(ref mut item) => item.take(),
            Inner::Many(ref mut iter) => iter.next(),
        }
    }
}


//------------ IterMut -------------------------------------------------------

#[derive(Debug)]
pub struct IterMut<'a, T: 'a>(Inner<Option<&'a mut T>, slice::IterMut<'a, T>>);

impl<'a, T: 'a> IterMut<'a, T> {
    fn new(list: &'a mut List<T>) -> Self {
        match list.inner {
            Inner::Empty => IterMut(Inner::Empty),
            Inner::One(ref mut item) => IterMut(Inner::One(Some(item))),
            Inner::Many(ref mut vec) => IterMut(Inner::Many(vec.iter_mut()))
        }
    }
}

impl<'a, T: 'a> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            Inner::Empty => None,
            Inner::One(ref mut item) => item.take(),
            Inner::Many(ref mut iter) => iter.next(),
        }
    }
}


//------------ ListError -----------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub enum ListError {
    Empty,
}

impl fmt::Display for ListError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ListError::Empty => f.write_str("empty list not allowed"),
        }
    }
}

