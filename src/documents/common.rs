use std::collections::HashMap;
use ::collection::CollectionBuilder;
use ::load::yaml::{FromYaml, ValueItem};
use super::source::SourceRef;


//------------ Progress ------------------------------------------------------

optional_enum! {
    pub enum Progress {
        (Stub => "stub"),
        (InProgress => "in-progress"),
        (Complete => "complete"),

        default InProgress
    }
}


//------------ LocalizedString -----------------------------------------------

pub struct LocalizedString(HashMap<String, String>);

impl FromYaml for LocalizedString {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        let (value, _) = item.into_mapping(builder)?.into_inner();
        let mut res = HashMap::new();
        let mut err = false;
        for (key, value) in value {
            let value = match value.into_string(builder) {
                Ok(value) => value,
                Err(()) => {
                    err = true;
                    continue
                }
            };
            res.insert(key, value);
        }
        if err { Err(()) }
        else { Ok(LocalizedString(res)) }
    }
}


impl<T> Default for ShortVec<T> {
    fn default() -> Self {
        ShortVec::Empty
    }
}


//------------ Sources -------------------------------------------------------

pub type Sources = ShortVec<SourceRef>;


//------------ ShortVec ------------------------------------------------------

pub enum ShortVec<T> {
    Empty,
    One(T),
    Many(Vec<T>),
}

impl<T> ShortVec<T> {
    pub fn iter(&self) -> ShortVecIter<T> {
        ShortVecIter::new(self)
    }

    pub fn len(&self) -> usize {
        match *self {
            ShortVec::Empty => 0,
            ShortVec::One(_) => 1,
            ShortVec::Many(ref vec) => vec.len()
        }
    }

    pub fn is_empty(&self) -> bool {
        match *self {
            ShortVec::Empty => true,
            ShortVec::One(_) => false,
            ShortVec::Many(ref vec) => vec.is_empty(),
        }
    }
}

impl<T: FromYaml> ShortVec<T> {
    pub fn from_opt_yaml(item: Option<ValueItem>, builder: &CollectionBuilder)
                         -> Result<Self, ()> {
        if let Some(item) = item {
            match item.try_into_sequence() {
                Ok(seq) => {
                    if seq.len() == 0 {
                        Ok(ShortVec::Empty)
                    }
                    else if seq.len() == 1 {
                        Self::from_opt_yaml(seq.into_iter().next(), builder)
                    }
                    else {
                        let mut vec = Vec::new();
                        let mut err = false;
                        for item in seq {
                            let item = match T::from_yaml(item, builder) {
                                Ok(item) => item,
                                Err(()) => {
                                    err = true;
                                    continue
                                }
                            };
                            if !err {
                                vec.push(item);
                            }
                        }
                        if err {
                            Err(())
                        }
                        else {
                            Ok(ShortVec::Many(vec))
                        }
                    }
                }
                Err(item) => {
                    T::from_yaml(item, builder).map(ShortVec::One)
                }
            }
        }
        else {
            Ok(ShortVec::Empty)
        }
    }
}

impl<T: FromYaml> FromYaml for ShortVec<T> {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        ShortVec::from_opt_yaml(Some(item), builder)
    }
}

impl<'a, T> IntoIterator for &'a ShortVec<T> {
    type Item = &'a T;
    type IntoIter = ShortVecIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        ShortVecIter::new(self)
    }
}


//------------ ShortVecIter --------------------------------------------------

pub enum ShortVecIter<'a, T: 'a> {
    Empty,
    One(&'a T),
    Many(&'a [T]),
}

impl<'a, T: 'a> ShortVecIter<'a, T> {
    fn new(vec: &'a ShortVec<T>) -> Self {
        match *vec {
            ShortVec::Empty => ShortVecIter::Empty,
            ShortVec::One(ref item) => ShortVecIter::One(item),
            ShortVec::Many(ref vec) => ShortVecIter::Many(vec)
        }
    }
}

impl<'a, T: 'a> Iterator for ShortVecIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        let res = match *self {
            ShortVecIter::Empty => return None,
            ShortVecIter::One(t) => t,
            ShortVecIter::Many(ref mut slice) => {
                let (first, rest) = match slice.split_first() {
                    None => return None,
                    Some(some) => some,
                };
                *slice = rest;
                return Some(first)
            }
        };
        *self = ShortVecIter::Empty;
        Some(res)
    }
}

