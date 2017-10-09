//! The store for the documents plus links between them.

use std::fmt;
use std::ops::{Deref, DerefMut};
use std::marker::PhantomData;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak};


//------------ Variant -------------------------------------------------------

/// A trait for a type that is part of an enum.
pub trait Variant: Sized {
    /// The type of the enum.
    type Item;

    /// The error for trying to convert from the enum to the variant.
    type Err: fmt::Debug;

    fn from_doc(doc: &Self::Item) -> Result<&Self, Self::Err>;
    fn from_doc_mut(doc: &mut Self::Item) -> Result<&mut Self, Self::Err>;
}


//------------ Link ----------------------------------------------------------

/// A link to a document.
pub struct Link<T: Variant> {
    link: Weak<RwLock<T::Item>>,
    marker: PhantomData<T>,
}

impl<T: Variant> Link<T> {
    fn new(link: Weak<RwLock<T::Item>>) -> Self {
        Link { link, marker: PhantomData }
    }

    pub fn convert<U: Variant<Item=T::Item>>(self) -> Link<U> {
        Link::new(self.link)
    }

    pub fn upgrade(&self) -> Result<Permalink<T>, LinkError<T::Err>> {
        let arc = self.link.upgrade().ok_or(LinkError::Gone)?;
        Ok(Permalink::new(arc))
    }

    pub fn check(&self) -> Result<(), LinkError<T::Err>> {
        self.with(|_| ())
    }

    pub fn with<F, U>(&self, f: F) -> Result<U, LinkError<T::Err>>
                where F: FnOnce(&T) -> U {
        let arc = self.link.upgrade().ok_or(LinkError::Gone)?;
        let guard = arc.read().map_err(|_| LinkError::Poisoned)?;
        let t = T::from_doc(guard.deref())?;
        Ok(f(t))
    }

    pub fn try_with<F, U, E>(&self, f: F) -> Result<U, E>
                    where F: FnOnce(&T) -> Result<U, E>,
                          E: From<LinkError<T::Err>> {
        let arc = self.link.upgrade().ok_or(LinkError::Gone.into())?;
        let guard = arc.read().map_err(|_| LinkError::Poisoned.into())?;
        let t = T::from_doc(guard.deref()).map_err(LinkError::Variant)?;
        f(t)
    }

    pub fn try_mut<F, U>(&self, f: F) -> Result<U, LinkError<T::Err>>
                   where F: FnOnce(&mut T) -> U {
        let arc = self.link.upgrade().ok_or(LinkError::Gone)?;
        let mut guard = arc.write().map_err(|_| LinkError::Poisoned)?;
        let t = T::from_doc_mut(guard.deref_mut())?;
        Ok(f(t))
    }

    pub fn try_with_mut<F, U, E>(&self, f: F) -> Result<U, E>
                    where F: FnOnce(&mut T) -> Result<U, E>,
                          E: From<LinkError<T::Err>> {
        let arc = self.link.upgrade().ok_or(LinkError::Gone.into())?;
        let mut guard = arc.write().map_err(|_| LinkError::Poisoned.into())?;
        let t = T::from_doc_mut(guard.deref_mut()).map_err(LinkError::Variant)?;
        f(t)
    }
}

impl<T: Variant> Clone for Link<T> {
    fn clone(&self) -> Self {
        Link::new(self.link.clone())
    }
}

impl<T: Variant> fmt::Debug for Link<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Link<_>()")
    }
}


//------------ Permalink -----------------------------------------------------

pub struct Permalink<T: Variant> {
    link: Arc<RwLock<T::Item>>,
    marker: PhantomData<T>,
}

impl<T: Variant> Permalink<T> {
    fn new(link: Arc<RwLock<T::Item>>) -> Self {
        Permalink { link, marker: PhantomData }
    }

    pub fn downgrade(&self) -> Link<T> {
        Link::new(Arc::downgrade(&self.link))
    }

    pub fn get(&self) -> Result<PermalinkRef<T>, PermalinkError<T::Err>> {
        PermalinkRef::new(&self.link)
    }

    pub fn get_mut(&self)
                   -> Result<PermalinkMut<T>, PermalinkError<T::Err>> {
        PermalinkMut::new(&self.link)
    }
}

impl<T: Variant> Clone for Permalink<T> {
    fn clone(&self) -> Self {
        Permalink::new(self.link.clone())
    }
}


//------------ PermalinkRef --------------------------------------------------

pub struct PermalinkRef<'a, T: Variant + 'a> {
    guard: RwLockReadGuard<'a, T::Item>,
    marker: PhantomData<T>,
}

impl<'a, T: Variant + 'a> PermalinkRef<'a, T> {
    fn new(arc: &'a Arc<RwLock<T::Item>>)
           -> Result<Self, PermalinkError<T::Err>> {
        let guard = arc.read().map_err(|_| PermalinkError::Poisoned)?;
        let _ = T::from_doc(guard.deref())?;
        Ok(PermalinkRef { guard, marker: PhantomData })
    }
}

impl<'a, T: Variant + 'a> Deref for PermalinkRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        T::from_doc(self.guard.deref()).unwrap()
    }
}


//------------ PermalinkMut --------------------------------------------------

pub struct PermalinkMut<'a, T: Variant + 'a> {
    guard: RwLockWriteGuard<'a, T::Item>,
    marker: PhantomData<T>,
}

impl<'a, T: Variant + 'a> PermalinkMut<'a, T> {
    fn new(arc: &'a Arc<RwLock<T::Item>>)
           -> Result<Self, PermalinkError<T::Err>> {
        let mut guard = arc.write().map_err(|_| PermalinkError::Poisoned)?;
        let _ = T::from_doc_mut(guard.deref_mut())?;
        Ok(PermalinkMut { guard, marker: PhantomData })
    }
}

impl<'a, T: Variant + 'a> Deref for PermalinkMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        T::from_doc(self.guard.deref()).unwrap()
    }
}

impl<'a, T: Variant + 'a> DerefMut for PermalinkMut<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        T::from_doc_mut(self.guard.deref_mut()).unwrap()
    }
}


//------------ ItemPermalink -------------------------------------------------

pub struct ItemPermalink<T> {
    link: Arc<RwLock<T>>
}

impl<T> ItemPermalink<T> {
    fn new(item: Arc<RwLock<T>>) -> Self {
        ItemPermalink { link: item }
    }

    pub fn convert<U>(&self) -> Result<Permalink<U>, PermalinkError<U::Err>>
               where U: Variant<Item=T> {
        let res = Permalink::new(self.link.clone());
        let _ = res.get()?;
        Ok(res)
    }

    pub fn force_convert<U>(&self) -> Permalink<U>
                         where U: Variant<Item=T> {
        Permalink::new(self.link.clone())
    }

    pub fn get(&self) -> Result<ItemRef<T>, ItemError> {
        ItemRef::new(&self.link)
    }

    pub fn get_mut(&mut self) -> Result<ItemMut<T>, ItemError> {
        ItemMut::new(&self.link)
    }
}

impl<T> Clone for ItemPermalink<T> {
    fn clone(&self) -> Self {
        ItemPermalink::new(self.link.clone())
    }
}


//------------ ItemRef -------------------------------------------------------

pub struct ItemRef<'a, T: 'a> {
    guard: RwLockReadGuard<'a, T>
}

impl<'a, T: 'a> ItemRef<'a, T> {
    fn new(arc: &'a Arc<RwLock<T>>) -> Result<Self, ItemError> {
        arc.read().map(|guard| ItemRef { guard }).map_err(|_| ItemError)
    }
}

impl<'a, T: 'a> Deref for ItemRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.guard.deref()
    }
}

impl<'a, T: 'a> AsRef<T> for ItemRef<'a, T> {
    fn as_ref(&self) -> &T {
        self.deref()
    }
}


//------------ ItemMut -------------------------------------------------------

pub struct ItemMut<'a, T: 'a> {
    guard: RwLockWriteGuard<'a, T>
}

impl<'a, T: 'a> ItemMut<'a, T> {
    fn new(arc: &'a Arc<RwLock<T>>) -> Result<Self, ItemError> {
        arc.write().map(|guard| ItemMut { guard }).map_err(|_| ItemError)
    }
}

impl<'a, T: 'a> Deref for ItemMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.guard.deref()
    }
}

impl<'a, T: 'a> DerefMut for ItemMut<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.guard.deref_mut()
    }
}

impl<'a, T: 'a> AsRef<T> for ItemMut<'a, T> {
    fn as_ref(&self) -> & T {
        self.deref()
    }
}

impl<'a, T: 'a> AsMut<T> for ItemMut<'a, T> {
    fn as_mut(&mut self) -> &mut T {
        self.deref_mut()
    }
}


//------------ Store ---------------------------------------------------------

pub struct Store<T>(Vec<Arc<RwLock<T>>>);

impl<T> Store<T> {
    pub fn new() -> Self {
        Store(Vec::new())
    }

    pub fn merge(&mut self, mut other: Self) {
        self.0.append(&mut other.0 )
    }

    pub fn insert(&mut self, item: T) -> ItemPermalink<T> {
        let item = Arc::new(RwLock::new(item));
        self.0.push(item.clone());
        ItemPermalink::new(item)
    }

    pub fn iter<V>(&self) -> StoreIter<V>
                where V: Variant<Item = T> {
        StoreIter(self.0.as_ref())
    }
}


//----------- StoreIter ------------------------------------------------------

pub struct StoreIter<'a, T: Variant + 'a>(&'a [Arc<RwLock<T::Item>>]);

impl<'a, T: Variant + 'a> Iterator for StoreIter<'a, T> {
    type Item = Permalink<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.split_first() {
            Some((head, tail)) => {
                self.0 = tail;
                Some(Permalink::new(head.clone()))
            }
            None => None
        }
    }
}


//------------ ItemError -----------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ItemError;


//------------ LinkError -----------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LinkError<T> {
    Gone,
    Poisoned,
    Variant(T)
}

impl<T> From<T> for LinkError<T> {
    fn from(err: T) -> Self {
        LinkError::Variant(err)
    }
}

//------------ PermalinkError ------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PermalinkError<T> {
    Poisoned,
    Variant(T)
}

impl<T> From<T> for PermalinkError<T> {
    fn from(err: T) -> Self {
        PermalinkError::Variant(err)
    }
}

