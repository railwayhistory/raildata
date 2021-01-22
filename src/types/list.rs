//! A list with an optimization for holding a single item.

use std::{cmp, fmt, mem, ops, slice};
use serde::{Deserialize, Serialize};
use crate::load::report::{Failed, PathReporter};
use crate::load::yaml::{FromYaml, Value};
use super::marked::IntoMarked;


//------------ List ----------------------------------------------------------

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct List<T> {
    inner: Inner<T, Vec<T>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
enum Inner<O, M> {
    Empty,
    One(O),
    Many(M),
}

impl<T> List<T> {
    pub fn new() -> Self {
        List { inner: Inner::Empty }
    }

    pub fn with_value(value: T) -> Self {
        List { inner: Inner::One(value) }
    }

    pub fn as_slice(&self) -> &[T] {
        match self.inner {
            Inner::Empty => &[],
            Inner::One(ref inner) => slice::from_ref(inner),
            Inner::Many(ref inner) => inner.as_ref(),
        }
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

    pub fn clear(&mut self) {
        self.inner = Inner::Empty
    }

    pub fn sort_by<F>(&mut self, op: F)
    where F: FnMut(&T, &T) -> cmp::Ordering {
        if let Inner::Many(ref mut inner) = self.inner {
            inner.sort_unstable_by(op)
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

    pub fn first(&self) -> Option<&T> {
        match self.inner {
            Inner::Empty => None,
            Inner::One(ref item) => Some(item),
            Inner::Many(ref vec) => vec.first(),
        }
    }

    pub fn last(&self) -> Option<&T> {
        match self.inner {
            Inner::Empty => None,
            Inner::One(ref item) => Some(item),
            Inner::Many(ref vec) => vec.last(),
        }
    }

    pub fn last_mut(&mut self) -> Option<&mut T> {
        match self.inner {
            Inner::Empty => None,
            Inner::One(ref mut item) => Some(item),
            Inner::Many(ref mut vec) => vec.last_mut(),
        }
    }
}

impl<T> Default for List<T> {
    fn default() -> Self {
        List { inner: Inner::Empty }
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

impl<T> From<Vec<T>> for List<T> {
    fn from(src: Vec<T>) -> Self {
        if src.is_empty() {
            List { inner: Inner::Empty }
        }
        else if src.len() == 1 {
            let mut iter = src.into_iter();
            List { inner: Inner::One(iter.next().unwrap()) }
        }
        else {
            List { inner: Inner::Many(src) }
        }
    }
}

impl<C, T: FromYaml<C>> FromYaml<C> for List<T> {
    fn from_yaml(
        value: Value,
        context: &C,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let location = value.location();
        let inner = match value.try_into_sequence() {
            Ok(mut seq) => {
                if seq.is_empty() {
                    report.error(ListError::Empty.marked(location));
                    return Err(Failed)
                }
                else if seq.len() == 1 {
                    T::from_yaml(seq.pop().unwrap(), context, report)
                        .map(Inner::One)?
                }
                else {
                    let mut res = Vec::with_capacity(seq.len());
                    let mut err = false;
                    for item in seq {
                        if let Ok(item) = T::from_yaml(item, context, report) {
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
            Err(value) => T::from_yaml(value, context, report).map(Inner::One)?
        };
        Ok(List { inner })
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

impl<T: PartialEq> PartialEq for List<T> {
    fn eq(&self, other: &Self) -> bool {
        use self::Inner::*;

        match (&self.inner, &other.inner) {
            (&Empty, &Empty) => true,
            (&One(ref left), &One(ref right)) => left.eq(right),
            (&Many(ref left), &Many(ref right)) => left.eq(right),
            _ => false,
        }
    }
}

impl<T: Eq> Eq for List<T> { }

impl<T> ops::Index<usize> for List<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.as_slice().index(index)
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

impl<'a, T: 'a> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<&'a T> {
        match self.0 {
            Inner::Empty => None,
            Inner::One(ref mut item) => item.take(),
            Inner::Many(ref mut iter) => iter.next_back(),
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

