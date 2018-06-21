use std::any::TypeId;
use std::collections::hash_map::{Entry, HashMap};
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};
use ::load::report::{Failed, Origin, PathReporter, StageReporter};
use ::types::{IntoMarked, Location, Key, Marked};


//------------ StoreBuilder --------------------------------------------------

#[derive(Clone, Debug)]
pub struct StoreBuilder<F>(Arc<RwLock<BuilderInner<F>>>);

impl<F> StoreBuilder<F> {
    pub fn new() -> Self
    where F: Default {
        StoreBuilder(Arc::new(RwLock::new(BuilderInner::new())))
    }

    pub fn insert<T: FindShelf<FillStore=F>>(
        &self, 
        key: Key,
        item: T,
        location: Location,
        report: &mut PathReporter
    ) -> Result<(), ExistingItem> {
        self.0.write().unwrap().insert(key, item, location, report)
    }

    pub fn insert_broken<T: 'static>(
        &self,
        key: Key,
        location: Location,
        report: &mut PathReporter
    ) -> Result<(), ExistingItem> {
        self.0.write().unwrap().insert_broken::<T>(key, location, report)
    }

    pub fn forge_link<T: FindShelf<FillStore=F>>(
        &self,
        key: Marked<Key>,
        report: &mut PathReporter
    ) -> Result<Marked<Link<T>>, Failed> {
        let location = key.location();
        if let Some(res) = self.0.read().unwrap().try_forge_link(&key)
            .map_err(|err| {report.error(err.marked(location)); Failed })?
        {
            return Ok(res.marked(location))
        }
        match self.0.write().unwrap().forge_link(key, report) {
            Ok(link) => Ok(link.marked(location)),
            Err(err) => {
                report.error(err.marked(location));
                Err(Failed)
            }
        }
    }

    pub fn forge_generic(
        &self,
        key: Marked<Key>,
        report: &mut PathReporter
    ) -> GenericLink {
        if let Some(res) = self.0.read().unwrap().try_forge_generic(&key) {
            return res
        }
        self.0.write().unwrap().forge_generic(key, report)
    }

    pub fn finish(
        self,
        report: &mut StageReporter
    ) -> Result<(F, Vec<GenericLinkTarget>), Failed> {
        let mut inner = match Arc::try_unwrap(self.0) {
            Ok(lock) => lock.into_inner().unwrap(),
            Err(_) => panic!("cannot unwrap store builder"),
        };
        let mut generics = vec![None; inner.generics_len];
        let mut failed = false;
        for (key, info) in inner.keys.drain() {
            match info.state {
                ItemState::Stored { tid, index, .. } => {
                    if let Some(gen_index) = info.generic {
                        generics[gen_index] = Some(GenericLinkTarget {
                            tid, pos: index
                        })
                    }
                }
                ItemState::Nonexisting { links, .. } => {
                    if !links.is_empty() {
                        for (_, origin) in links {
                            report.error_at(
                                origin.clone(),
                                MissingDocument(key.clone())
                            );
                        }
                        failed = true;
                    }
                }
                ItemState::Broken { .. } => {
                    failed = true;
                }
            }
        }
        let mut res_generics = Vec::with_capacity(generics.len());
        for item in generics {
            if let Some(item) = item {
                res_generics.push(item)
            }
            else {
                failed = true;
                break
            }
        }
        if failed {
            Err(Failed)
        }
        else {
            Ok((inner.store, res_generics))
        }
    }
}


//------------ BuilderInner --------------------------------------------------

#[derive(Clone, Debug)]
struct BuilderInner<F> { 
    store: F,
    keys: HashMap<Key, ItemInfo>,
    generics_len: usize,
}


impl<F> BuilderInner<F> {
    pub fn new() -> Self
    where F: Default {
        BuilderInner {
            store: F::default(),
            keys: HashMap::new(),
            generics_len: 0,
        }
    }

    pub fn insert<T: FindShelf<FillStore=F>>(
        &mut self, 
        key: Key,
        item: T,
        location: Location,
        report: &mut PathReporter
    ) -> Result<(), ExistingItem> {
        let info = if let Some(info) = self.keys.get_mut(&key) {
            if let Some(index) = info.state.try_upgrade::<T>(location, report)? {
                T::fill_shelf(&mut self.store).update(index, item);
                return Ok(())
            }
            else {
                // Since HashMap::insert doesn’t care if there’s a value
                // already, we can fall through here and insert a new value
                // below.
                let index = T::fill_shelf(&mut self.store).insert(Some(item));
                let mut new_info = ItemInfo::new(
                    ItemState::Stored {
                        tid: TypeId::of::<T>(),
                        index,
                        origin: Origin::new(report.path(), location),
                    }
                );
                new_info.generic = info.generic;
                new_info
            }
        }
        else {
            let index = T::fill_shelf(&mut self.store).insert(Some(item));
            ItemInfo::new(
                ItemState::Stored {
                    tid: TypeId::of::<T>(),
                    index,
                    origin: Origin::new(report.path(), location),
                }
            )
        };
        self.keys.insert(key, info);
        Ok(())
    }
 
    pub fn insert_broken<T: 'static>(
        &mut self,
        key: Key,
        location: Location,
        report: &mut PathReporter
    ) -> Result<(), ExistingItem> {
        if let Some(info) = self.keys.get_mut(&key) {
            info.state.try_upgrade::<T>(location, report)?;
        }
        self.keys.insert(
            key,
            ItemInfo::new(ItemState::Broken {
                tid: TypeId::of::<T>(),
                origin: Origin::new(report.path(), location)
            })
        );
        Ok(())
    }

    pub fn try_forge_link<T: FindShelf<FillStore=F>>(
        &self,
        key: &Marked<Key>
    ) -> Result<Option<Link<T>>, TypeMismatch> {
        match self.keys.get(key).map(|item| &item.state) {
            Some(&ItemState::Stored { tid, index, .. }) => {
                if tid != TypeId::of::<T>() {
                    Err(TypeMismatch {
                        expected: TypeId::of::<T>(),
                        target: tid
                    })
                }
                else {
                    Ok(Some(Link::new(index)))
                }
            }
            Some(&ItemState::Broken { tid, .. }) => {
                if tid != TypeId::of::<T>() {
                    Err(TypeMismatch {
                        expected: TypeId::of::<T>(),
                        target: tid
                    })
                }
                else {
                    // We return a link pointing to the first document
                    // of the right type as a stand-in for the (never
                    // added) broken document. This is okay since
                    // conversion to a regular store will fail later if
                    // there are broken items.
                    Ok(Some(Link::new(0)))
                }
            }
            _ => Ok(None)
        }
    }

    pub fn forge_link<T: FindShelf<FillStore=F>>(
        &mut self,
        key: Marked<Key>,
        report: &mut PathReporter,
    ) -> Result<Link<T>, TypeMismatch> {
        // Someone may have added the item by now ...
        if let Some(link) = self.try_forge_link(&key)? {
            return Ok(link)
        }

        let tid = TypeId::of::<T>();
        let origin = Origin::new(report.path(), key.location());
        match self.keys.get_mut(&key).map(|item| &mut item.state) {
            Some(&mut ItemState::Nonexisting { ref mut index, ref mut links })
            => {
                links.push((Some(tid), origin));
                if let Some(&(_, res)) = index.iter().find(|x| x.0 == tid) {
                    return Ok(Link::new(res))
                }
                else {
                    let res = T::fill_shelf(&mut self.store).insert(None);
                    index.push((tid, res));
                    return Ok(Link::new(res))
                }
            }
            None => { }
            // Everything else has been taken care of by try_forge_link.
            _ => unreachable!()
        }
        let res = T::fill_shelf(&mut self.store).insert(None);
        self.keys.insert(
            key.into_value(),
            ItemInfo::new(ItemState::Nonexisting {
                index: vec![(tid, res)],
                links: vec![(Some(tid), origin)],
            })
        );
        Ok(Link::new(res))
    }

    fn try_forge_generic(&self, key: &Key) -> Option<GenericLink> {
        if let Some(index) = self.keys.get(key).and_then(|item| item.generic) {
            Some(GenericLink::new(index))
        }
        else {
            None
        }
    }

    fn forge_generic(
        &mut self,
        key: Marked<Key>,
        report: &mut PathReporter
    ) -> GenericLink {
        let origin = Origin::new(report.path(), key.location());
        let index = match self.keys.entry(key.into_value()) {
            Entry::Occupied(mut entry) => {
                if let ItemState::Nonexisting { ref mut links, .. }
                    = entry.get_mut().state
                {
                    links.push((None, origin))
                }
                if let Some(index) = entry.get().generic {
                    index
                }
                else {
                    let res = self.generics_len;
                    self.generics_len += 1;
                    entry.get_mut().generic = Some(res);
                    res
                }
            }
            Entry::Vacant(mut entry) => {
                let res = self.generics_len;
                self.generics_len += 1;
                entry.insert(
                    ItemInfo {
                        state: ItemState::Nonexisting {
                            index: Vec::new(),
                            links: Vec::new(),
                        },
                        generic: Some(res)
                    }
                );
                res
            }
        };
        GenericLink::new(index)
    }
}


//------------ ItemInfo ------------------------------------------------------

#[derive(Clone, Debug)]
struct ItemInfo {
    state: ItemState,
    generic: Option<usize>,
}

impl ItemInfo {
    fn new(state: ItemState) -> Self {
        ItemInfo {
            state,
            generic: None
        }
    }
}


//------------ ItemState -----------------------------------------------------

#[derive(Clone, Debug)]
enum ItemState {
    Stored {
        tid: TypeId,
        index: usize,
        origin: Origin,
    },
    Nonexisting {
        index: Vec<(TypeId, usize)>,
        links: Vec<(Option<TypeId>, Origin)>,
    },
    Broken {
        tid: TypeId,
        origin: Origin,
    }
}

impl ItemState {
    fn try_upgrade<T: 'static>(
        &mut self,
        location: Location,
        report: &mut PathReporter
    ) -> Result<Option<usize>, ExistingItem> {
        let (res, next) = match *self {
            ItemState::Stored { ref origin, .. } => {
                return Err(ExistingItem(origin.clone()))
            }
            ItemState::Broken { ref origin, .. } => {
                return Err(ExistingItem(origin.clone()))
            }
            ItemState::Nonexisting { ref index, ref links } => {
                let tid = TypeId::of::<T>();
                
                for (link_tid, origin) in links {
                    if let Some(link_tid) = *link_tid {
                        if link_tid != tid {
                            report.global().error_at(
                                origin.clone(),
                                TypeMismatch {
                                    expected: link_tid,
                                    target: tid,
                                }
                            )
                        }
                    }
                }

                if let Some(&(_, index)) = index.iter().find(|&x| x.0 == tid) {
                    (
                        index,
                        ItemState::Stored {
                            tid,
                            index,
                            origin: Origin::new(report.path(), location)
                        }
                    )
                }
                else {
                    // We can leave *self as it is, the caller will replace
                    // it after having gotten an index.
                    return Ok(None)
                }
            }
        };
        *self = next;
        Ok(Some(res))
    }
}


//------------ Shelf ---------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Shelf<T> {
    items: Vec<T>
}

impl<T> Shelf<T> {
    pub fn get(&self, index: usize) -> &T {
        &self.items[index]
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}


//------------ FillShelf -----------------------------------------------------

#[derive(Clone, Debug)]
pub struct FillShelf<T> {
    items: Vec<Option<T>>
}

impl<T> FillShelf<T> {
    pub fn new() -> Self {
        FillShelf {
            items: Vec::new()
        }
    }

    pub fn insert(&mut self, item: Option<T>) -> usize {
        let res = self.items.len();
        self.items.push(item);
        res
    }

    pub fn update(&mut self, index: usize, item: T) {
        self.items[index] = Some(item)
    }

    pub fn into_shelf(self) -> Option<Shelf<T>> {
        let mut res = Shelf { items: Vec::with_capacity(self.items.len()) };
        for item in self.items {
            match item {
                Some(item) => res.items.push(item),
                None => return None
            }
        }
        Some(res)
    }
}

impl<T> Default for FillShelf<T> {
    fn default() -> Self {
        Self::new()
    }
}


//------------ Link ----------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct Link<T> {
    /// The position of the linked item in its shelf.
    pos: usize,
    marker: PhantomData<T>,
}

impl<T> Link<T> {
    fn new(pos: usize) -> Self {
        Link {
            pos,
            marker: PhantomData
        }
    }

    pub fn resolve<'a>(&self, store: &'a <T as FindShelf>::Store) -> &'a T
    where T: FindShelf {
        T::shelf(store).items.get(self.pos).unwrap()
    }
}


//------------ GenericLink ---------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct GenericLink {
    pos: usize,
}

impl GenericLink {
    fn new(pos: usize) -> Self {
        GenericLink { pos }
    }
}


//------------ GenericLinkTarget ---------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct GenericLinkTarget {
    pub tid: TypeId,
    pub pos: usize,
}


//------------ FindShelf -----------------------------------------------------

pub trait FindShelf: Sized + 'static {
    type Store;
    type FillStore;

    fn shelf(store: &Self::Store) -> &Shelf<Self>;
    fn fill_shelf(store: &mut Self::FillStore) -> &mut FillShelf<Self>;
}


//============ Errors ========================================================

#[derive(Clone, Debug, Fail)]
#[fail(display="document already exists, first defined at {}", _0)]
pub struct ExistingItem(Origin);

#[derive(Clone, Copy, Debug, Fail)]
#[fail(display="linked document is a {:?}, expected {:?}", target, expected)]
pub struct TypeMismatch {
    expected: TypeId,
    target: TypeId,
}

#[derive(Clone, Debug, Fail)]
#[fail(display="link to missing document '{}'", _0)]
pub struct MissingDocument(Key);

