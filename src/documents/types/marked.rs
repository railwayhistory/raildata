//! Marking values with their source location.

use std::{borrow, cmp, fmt, hash, ops};
use std::cmp::min;
use yaml_rust::scanner::Marker;


//------------ Marked --------------------------------------------------------

/// A value that optionally is marked by its source location.
#[derive(Clone, Copy, Default)]
pub struct Marked<T> {
    value: T,
    location: Location,
}

impl<T> Marked<T> {
    pub fn new(value: T, location: Location) -> Self {
        Marked { value, location }
    }

    pub fn from_value(value: T) -> Self {
        Marked { value, location: Location::default() }
    }

    pub fn location(&self) -> Location {
        self.location
    }

    pub fn as_value(&self) -> &T {
        &self.value
    }

    pub fn as_value_mut(&mut self) -> &mut T {
        &mut self.value
    }

    pub fn into_value(self) -> T {
        self.value
    }

    pub fn unwrap(self) -> (T, Location) {
        (self.value, self.location)
    }

    pub fn map<F, U>(self, f: F) -> Marked<U>
               where F: FnOnce(T) -> U {
        Marked { value: f(self.value), location: self.location }
    }

    pub fn try_map<F, U, E>(self, f: F) -> Result<Marked<U>, Marked<E>>
                   where F: FnOnce(T) -> Result<U, E> {
        match f(self.value) {
            Ok(value) => Ok(Marked { value, location: self.location }),
            Err(value) => Err(Marked { value, location: self.location }),
        }
    }
}

impl<T: Copy> Marked<T> {
    pub fn to(&self) -> T {
        self.value
    }
}


//--- Deref and DerefMut

impl<T> ops::Deref for Marked<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T> ops::DerefMut for Marked<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}


//--- Borrow, BorrowMut, AsRef, and AsMut

impl<T> borrow::Borrow<T> for Marked<T> {
    fn borrow(&self) -> &T {
        &self.value
    }
}

impl<T> borrow::BorrowMut<T> for Marked<T> {
    fn borrow_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T> AsRef<T> for Marked<T> {
    fn as_ref(&self) -> &T {
        &self.value
    }
}

impl<T> AsMut<T> for Marked<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.value
    }
}


//--- PartialEq and Eq
//
// We can only implement PartialEq for other marked values, because of an
// issue with conflicting implementations if we did both PartialEq<U> and
// PartialEq<Marked<U>>. If we only did PartialEq<U> we couldn’t do Eq.

impl<T: PartialEq<U>, U> PartialEq<Marked<U>> for Marked<T> {
    fn eq(&self, other: &Marked<U>) -> bool {
        self.value.eq(other)
    }
}

impl<T: Eq> Eq for Marked<T> { }


//--- PartialOrd and Ord
//
// Ditto.

impl<T: PartialOrd<U>, U> PartialOrd<Marked<U>> for Marked<T> {
    fn partial_cmp(&self, other: &Marked<U>) -> Option<cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl<T: Ord> Ord for Marked<T> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.value.cmp(&other.value)
    }
}


//--- Hash

impl<T: hash::Hash> hash::Hash for Marked<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state)
    }
}


//--- Display and Debug

impl<T: fmt::Display> fmt::Display for Marked<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<T: fmt::Debug> fmt::Debug for Marked<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Marked")
            .field("value", &self.value)
            .field("location", &self.location)
            .finish()
    }
}


//------------ Location ------------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Location(u32);

impl Location {
    pub const NONE: Self = Location(0xFFFF_FFFF);

    pub fn new(line: usize, col: usize) -> Self {
        Location((min(line, 0xFFFF) as u32) << 16 | (min(col, 0xFFFF) as u32))
    }

    pub fn line(&self) -> Option<u16> {
        let res = (self.0 >> 16) as u16;
        if res == 0xFFFF {
            None
        }
        else {
            Some(res)
        }
    }

    pub fn col(&self) -> Option<u16> {
        let res = (self.0 & 0xFFFF) as u16;
        if res == 0xFFFF {
            None
        }
        else {
            Some(res)
        }
    }
}

impl Default for Location {
    fn default() -> Self {
        Location(0xFFFF_FFFF)
    }
}

impl From<Marker> for Location {
    fn from(mark: Marker) -> Self {
        Self::new(mark.line(), mark.col())
    }
}

impl From<Option<Marker>> for Location {
    fn from(mark: Option<Marker>) -> Self {
        match mark {
            Some(mark) => mark.into(),
            None => Location(0xFFFF_FFFF)
        }
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(line) = self.line() {
            if let Some(col) = self.col() {
                write!(f, ":{}:{}", line, col)
            }
            else {
                write!(f, ":{}", line)
            }
        }
        else {
            Ok(())
        }
    }
}
