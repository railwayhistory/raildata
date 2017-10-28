//! Links between documents.

use std::fmt;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, RwLock, Weak};
use ::document::{Line, Organization, Path, Point, Source, Structure};
use ::document::{Document, DocumentType, Variant, VariantError};
use ::load::construct::{Constructable, ConstructContext, Failed};
use ::load::yaml::Value;
use ::types::{Key, Location, Marked};


//------------ Link ----------------------------------------------------------

pub struct Link<V: Variant> {
    link: Weak<RwLock<Document>>,
    marker: PhantomData<V>,
}

pub type LineLink = Link<Line>;
pub type OrganizationLink = Link<Organization>;
pub type PathLink = Link<Path>;
pub type PointLink = Link<Point>;
pub type SourceLink = Link<Source>;
pub type StructureLink = Link<Structure>;

impl<V: Variant> Link<V> {
    fn new(link: Weak<RwLock<Document>>) -> Self {
        Link { link, marker: PhantomData }
    }

    pub fn convert<U: Variant>(self) -> Link<U> {
        Link::new(self.link)
    }
    
    pub fn check(&self) -> Result<(), LinkError> {
        self.with(|_| ())
    }

    pub fn with<F, T>(&self, op: F) -> Result<T, LinkError>
                where F: FnOnce(&V) -> T {
        let arc = self.link.upgrade().ok_or(LinkError::Gone)?;
        let guard = arc.read().map_err(|_| LinkError::Poisoned)?;
        let v = V::from_doc(guard.deref())?;
        Ok(op(v))
    }

    pub fn try_with<F, T, E>(&self, op: F) -> Result<T, E>
                    where F: FnOnce(&V) -> Result<T, E>,
                          E: From<LinkError> {
        let arc = self.link.upgrade().ok_or(LinkError::Gone)?;
        let guard = arc.read().map_err(|_| LinkError::Poisoned)?;
        let v = V::from_doc(guard.deref()).map_err(LinkError::Variant)?;
        op(v)
    }

    pub fn with_mut<F, T>(&self, op: F) -> Result<T, LinkError>
                    where F: FnOnce(&mut V) -> T {
        let arc = self.link.upgrade().ok_or(LinkError::Gone)?;
        let mut guard = arc.write().map_err(|_| LinkError::Poisoned)?;
        let v = V::from_doc_mut(guard.deref_mut())?;
        Ok(op(v))
    }

    pub fn try_with_mut<F, T, E>(&self, op: F) -> Result<T, E>
                        where F: FnOnce(&mut V) -> Result<T, E>,
                              E: From<LinkError> {
        let arc = self.link.upgrade().ok_or(LinkError::Gone)?;
        let mut guard = arc.write().map_err(|_| LinkError::Poisoned)?;
        let v = V::from_doc_mut(guard.deref_mut()).map_err(LinkError::Variant)?;
        op(v)
    }
}

impl<V: Variant> Clone for Link<V> {
    fn clone(&self) -> Self {
        Link::new(self.link.clone())
    }
}

impl<V: Variant> Constructable for Marked<Link<V>> {
    fn construct(value: Value, context: &mut ConstructContext)
                 -> Result<Self, Failed> {
        let location = value.location();
        let key = Key::construct(value, context)?;
        Link::from_key(key, location, context)
             .map(|link| Marked::new(link, location))
    }
}

impl<V: Variant> Constructable for Link<V> {
    fn construct(value: Value, context: &mut ConstructContext)
                 -> Result<Self, Failed> {
        Marked::construct(value, context).map(Marked::into_value)
    }
}


impl<V: Variant> Marked<Link<V>> {
    pub fn from_string(s: Marked<String>, context: &mut ConstructContext)
                       -> Result<Self, Failed> {
        let key = Marked::<Key>::from_string(s, context)?;
        Self::from_key(key, context)
    }

    pub fn from_key(key: Marked<Key>, context: &mut ConstructContext)
                    -> Result<Self, Failed> {
        let (key, location) = key.unwrap();
        Link::from_key(key, location, context)
             .map(|link| Marked::new(link, location))
    }
}

impl<V: Variant> Link<V> {
    pub fn from_string(s: String, context: &mut ConstructContext)
                       -> Result<Self, Failed> {
        let key = context.ok(Key::from_string(s)
                         .map_err(|err| (err, Location::NONE)))?;
        Self::from_key(key, Location::NONE, context)
    }

    pub fn from_key(key: Key, location: Location,
                    context: &mut ConstructContext) -> Result<Self, Failed> {
        let link = context.forge_link(key);
        let done = link.with(|doc| {
            if doc.is_nonexisting() {
                // We need to continue with a ref mut to add our link.
                Ok(false)
            }
            else {
                check_link_type(doc, V::DOC_TYPE, location, context)?;
                Ok(true)
            }
        });
        if !(done.unwrap()?) {
            // Picking up from the nonexisting case ...
            link.with_mut(|doc| {
                if let Some(doc) = doc.as_nonexisting_mut() {
                    doc.add_link(context.path().clone(), location,
                                 Some(V::DOC_TYPE));
                    return Ok(())
                }
                check_link_type(doc, V::DOC_TYPE, location, context)?;
                Ok(())
            }).unwrap()?;
        }
        Ok(link.convert())
    }
}

fn check_link_type(doc: &Document, doc_type: DocumentType, location: Location,
                   context: &mut ConstructContext) -> Result<(), Failed> {
    if let Some(broken) = doc.as_broken() {
        if let Some(broken_type) = broken.doc_type() {
            if broken_type != doc_type {
                context.push_error((
                    LinkError::from(
                        VariantError::new(doc_type, Some(broken_type))
                    ),
                    location
                ))
            }
        }
        Err(Failed)
    }
    else if doc.doc_type() != Some(doc_type) {
        context.push_error((
            LinkError::from(
                VariantError::new(doc_type, doc.doc_type())
            ),
            location
        ));
        Err(Failed)
    }
    else {
        Ok(())
    }
}

impl<V: Variant> fmt::Debug for Link<V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Link<_>()")
    }
}


//------------ DocumentLink --------------------------------------------------

pub struct DocumentLink {
    link: Weak<RwLock<Document>>,
}

impl DocumentLink {
    fn new(link: Weak<RwLock<Document>>) -> Self {
        DocumentLink { link }
    }

    pub fn convert<V: Variant>(self) -> Link<V> {
        Link::new(self.link)
    }

    pub fn with<F, T>(&self, op: F) -> Result<T, LinkError>
                where F: FnOnce(&Document) -> T {
        let arc = self.link.upgrade().ok_or(LinkError::Gone)?;
        let guard = arc.read().map_err(|_| LinkError::Poisoned)?;
        Ok(op(guard.deref()))
    }

    pub fn try_with<F, T, E>(&self, op: F) -> Result<T, E>
                    where F: FnOnce(&Document) -> Result<T, E>,
                          E: From<LinkError> {
        let arc = self.link.upgrade().ok_or(LinkError::Gone)?;
        let guard = arc.read().map_err(|_| LinkError::Poisoned)?;
        op(guard.deref())
    }

    pub fn with_mut<F, T>(&self, op: F) -> Result<T, LinkError>
                    where F: FnOnce(&mut Document) -> T {
        let arc = self.link.upgrade().ok_or(LinkError::Gone)?;
        let mut guard = arc.write().map_err(|_| LinkError::Poisoned)?;
        Ok(op(guard.deref_mut()))
    }

    pub fn try_with_mut<F, T, E>(&self, op: F) -> Result<T, E>
                        where F: FnOnce(&mut Document) -> Result<T, E>,
                              E: From<LinkError> {
        let arc = self.link.upgrade().ok_or(LinkError::Gone)?;
        let mut guard = arc.write().map_err(|_| LinkError::Poisoned)?;
        op(guard.deref_mut())
    }
}

impl Clone for DocumentLink {
    fn clone(&self) -> Self {
        DocumentLink::new(self.link.clone())
    }
}

impl Constructable for Marked<DocumentLink> {
    fn construct(value: Value, context: &mut ConstructContext)
                 -> Result<Self, Failed> {
        let key = Marked::<Key>::construct(value, context)?;
        Marked::<DocumentLink>::from_key(key, context)
    }
}

impl Marked<DocumentLink> {
    pub fn from_key(key: Marked<Key>, context: &mut ConstructContext)
                    -> Result<Self, Failed> {
        let (key, location) = key.unwrap();
        let link = context.forge_link(key);
        let new = link.with(|doc| doc.is_nonexisting()).unwrap();
        if new {
            link.with_mut(|doc| {
                if let Some(doc) = doc.as_nonexisting_mut() {
                    doc.add_link(context.path().clone(), location, None);
                }
            }).unwrap();
        }
        return Ok(Marked::new(link, location))
    }
}

impl fmt::Debug for DocumentLink {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("DocumentLink()")
    }
}


//------------ Permalink -----------------------------------------------------

pub struct Permalink {
    link: Arc<RwLock<Document>>,
}

impl Permalink {
    pub fn from_document(document: Document) -> Self {
        Permalink { link: Arc::new(RwLock::new(document)) }
    }

    pub fn nonexisting(key: Key) -> Self {
        Self::from_document(Document::nonexisting(key))
    }

    pub fn link(&self) -> DocumentLink {
        DocumentLink::new(Arc::downgrade(&self.link))
    }
}

impl Clone for Permalink {
    fn clone(&self) -> Self {
        Permalink { link: self.link.clone() }
    }
}    

impl Deref for Permalink {
    type Target = RwLock<Document>;

    fn deref(&self) -> &Self::Target {
        self.link.deref()
    }
}

impl fmt::Debug for Permalink {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Permalink()")
    }
}


//------------ LinkError -----------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LinkError {
    Gone,
    Poisoned,
    Variant(VariantError)
}

impl From<VariantError> for LinkError {
    fn from(err: VariantError) -> Self {
        LinkError::Variant(err)
    }
}

impl fmt::Display for LinkError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LinkError::Gone => write!(f, "internal error (link gone)"),
            LinkError::Poisoned => write!(f, "internal error (link poisoned"),
            LinkError::Variant(ref err) => err.fmt(f)
        }
    }
}

