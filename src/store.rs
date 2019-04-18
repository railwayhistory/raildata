//! A document store.

use std::{cmp, fmt, hash, mem, ops};
use std::sync::{Mutex, MutexGuard, TryLockError};
use std::marker::PhantomData;
use crossbeam::sync::MsQueue;
use rayon::prelude::*;


//------------ Store ---------------------------------------------------------

/// An imutable place to store imutable items.
#[derive(Debug)]
pub struct Store<S> {
    items: Vec<S>,
}

impl<S> Store<S> {
    /// Creates a new store from an iterator over its future items.
    pub fn from_iter<I: Iterator<Item=S>>(iter: I) -> Self {
        Store { items: iter.collect() }
    }

    /// Resolves a link into a reference to an item.
    ///
    /// # Panic
    ///
    /// This methods panics if `link` doesn’t link to an existing item.
    pub fn resolve(&self, link: Link<S>) -> &S {
        &self.items[link.index]
    }
}

impl<S> From<StoreMut<S>> for Store<S> {
    fn from(store: StoreMut<S>) -> Self {
        Self::from_iter(
            store.items.into_iter().map(|item| {
                item.item.into_inner().unwrap()
            })
        )
    }
}

impl<S> From<StoreBuilder<S>> for Store<S> {
    fn from(store: StoreBuilder<S>) -> Self {
        Self::from_iter(
            store.items.into_inner().unwrap().into_iter().map(Option::unwrap),
        )
    }
}


//------------ StoreMut ------------------------------------------------------

/// An imutable place to store mutable items.
pub struct StoreMut<S> {
    items: Vec<ItemMut<S>>,
}

struct ItemMut<S> {
    item: Mutex<S>,
    queue: MsQueue<Box<dyn Fn(&mut S) + Send>>,
}

impl<S> StoreMut<S> {
    /// Creates a new store from an iterator over items.
    pub fn from_iter<I: Iterator<Item=S>>(iter: I) -> Self {
        StoreMut {
            items: iter.map(|item| ItemMut {
                item: Mutex::new(item),
                queue: MsQueue::new()
            }).collect()
        }
    }

    /// Resolves a link.
    ///
    /// Blocks the current thread until the link can be resolved. This may
    /// lead to deadlocks if you aren’t careful.
    ///
    /// For simplicity, this panics if the item has been poisoned. This also
    /// panics if the link isn’t pointing to an item.
    pub fn resolve_mut(&self, link: Link<S>) -> ItemGuard<S> {
        Self::_resolve_mut(&self.items[link.index])
    }

    fn _resolve_mut(item: &ItemMut<S>) -> ItemGuard<S> {
        ItemGuard {
            guard: item.item.lock().unwrap(),
            queue: &item.queue
        }
    }

    /// Attempts to resolve a link.
    ///
    /// If resolving would block the thread, returns `None`. The same caveats
    /// as for `resolve_mut` apply.
    fn try_resolve_mut(&self, link: Link<S>) -> Option<ItemGuard<S>> {
        Self::_try_resolve_mut(&self.items[link.index])
    }

    fn _try_resolve_mut(item: &ItemMut<S>) -> Option<ItemGuard<S>> {
        Some(ItemGuard {
            guard: match item.item.try_lock() {
                Ok(guard) => guard,
                Err(TryLockError::Poisoned(_)) => panic!("poisoned mutex"),
                Err(TryLockError::WouldBlock) => return None,
            },
            queue: &item.queue
        })
    }

    pub fn update<F: Fn(&mut S) + 'static + Send>(&self, link: Link<S>, op: F) {
        let item = &self.items[link.index];
        if let Some(mut guard) = Self::_try_resolve_mut(item) {
            op(&mut guard)
        }
        else {
            item.queue.push(Box::new(op))
        }
    }

    pub fn par_iter(&self) -> impl ParallelIterator<Item=Link<S>> {
        (0..self.items.len()).into_par_iter().map(Link::new)
    }
}

impl<T> From<StoreBuilder<T>> for StoreMut<T> {
    fn from(store: StoreBuilder<T>) -> StoreMut<T> {
        Self::from_iter(
            store.items.into_inner().unwrap().into_iter().map(Option::unwrap)
        )
    }
}

impl<T> fmt::Debug for StoreMut<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "StoreMut {{ ... }}")
    }
}


//------------ ItemGuard ----------------------------------------------------

pub struct ItemGuard<'a, T> {
    guard: MutexGuard<'a, T>,
    queue: &'a MsQueue<Box<dyn Fn(&mut T) + Send>>
}

impl<'a, T> ops::Deref for ItemGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.guard.deref()
    }
}

impl<'a, T> ops::DerefMut for ItemGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.guard.deref_mut()
    }
}

impl<'a, T> Drop for ItemGuard<'a, T> {
    fn drop(&mut self) {
        while let Some(ref op) = self.queue.try_pop() {
            op(&mut self.guard)
        }
    }
}


//------------ StoreBuilder --------------------------------------------------

/// A mutable place to store imutable items.
#[derive(Debug)]
pub struct StoreBuilder<T> {
    items: Mutex<Vec<Option<T>>>,
}

impl<T> StoreBuilder<T> {
    /// Creates a new, empty store.
    pub fn new() -> Self {
        StoreBuilder { items: Mutex::new(Vec::new()) }
    }

    /// Creates a store from an iterator over items.
    pub fn from_iter<I: Iterator<Item=T>>(iter: I) -> Self {
        StoreBuilder {
            items: Mutex::new(iter.map(Some).collect()),
        }
    }

    /// Appends a new item to the store, returning a link to it.
    pub fn push(&self, item: Option<T>) -> Link<T> {
        let index = {
            let mut items = self.items.lock().unwrap();
            let res = items.len();
            items.push(item);
            res
        };
        Link::new(index)
    }

    /// Checkes whether a linked item already exists.
    pub fn exists(&self, link: Link<T>) -> bool {
        self.items.lock().unwrap()[link.index].is_some()
    }

    /// Updates an item.
    ///
    /// Returns the previous item.
    pub fn update(&self, link: Link<T>, item: T) -> Option<T> {
        mem::replace(&mut self.items.lock().unwrap()[link.index], Some(item))
    }
}


impl<T> Default for StoreBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}


//------------ Link ----------------------------------------------------------

/// A link to an item in a store.
#[derive(Debug)]
pub struct Link<T> {
    index: usize,
    marker: PhantomData<T>,
}

impl<T> Link<T> {
    /// Creates a new link from an index.
    fn new(index: usize) -> Self {
        Link { index, marker: PhantomData }
    }

    pub fn follow(self, store: &Store<T>) -> &T {
        store.resolve(self)
    }

    pub fn follow_mut(self, store: &StoreMut<T>) -> ItemGuard<T> {
        store.resolve_mut(self)
    }

    pub fn try_follow_mut(self, store: &StoreMut<T>) -> Option<ItemGuard<T>> {
        store.try_resolve_mut(self)
    }
}

unsafe impl<T> Send for Link<T> { }

impl<T> Clone for Link<T> {
    fn clone(&self) -> Self {
        Link {
            index: self.index,
            marker: PhantomData
        }
    }
}

impl<T> Copy for Link<T> { }

impl<T> PartialEq for Link<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<T> Eq for Link<T> { }

impl<T> PartialOrd for Link<T> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.index.partial_cmp(&other.index)
    }
}

impl<T> Ord for Link<T> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.index.cmp(&other.index)
    }
}

impl<T> hash::Hash for Link<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state)
    }
}

