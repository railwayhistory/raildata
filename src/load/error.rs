
use std::fmt;
use std::sync::{Arc, Mutex};
use ::types::{Location, Marked};
use super::path::Path;


//------------ Error ---------------------------------------------------------

pub struct Error(Marked<Box<fmt::Display + Send>>);

impl Error {
    pub fn new<E>(err: E, loc: Location) -> Self 
               where E: fmt::Display + 'static + Send {
        Error(Marked::new(Box::new(err), loc))
    }

    pub fn location(&self) -> Location {
        self.0.location()
    }

    pub fn inner(&self) -> &fmt::Display {
        self.0.as_value().as_ref()
    }
}

impl<E: fmt::Display + 'static + Send> From<Marked<E>> for Error {
    fn from(err: Marked<E>) -> Self {
        err.unwrap().into()
    }
}

impl<E: fmt::Display + Send + 'static> From<(E, Location)> for Error {
    fn from((err, loc): (E, Location)) -> Error {
        Error::new(err, loc)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error('{}')", self.0)
    }
}


//------------ ErrorStore ----------------------------------------------------

#[derive(Default, Debug)]
pub struct ErrorStore {
    errors: Vec<(Option<Path>, Error)>,
}

impl ErrorStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn push<E: Into<Error>>(&mut self, path: Option<&Path>, err: E) {
        self.errors.push((path.map(Clone::clone), err.into()))
    }

    pub fn merge(&mut self, path: Option<&Path>, errors: Vec<Error>) {
        let path = path.map(Clone::clone);
        for item in errors {
            self.errors.push((path.clone(), item))
        }
    }

    pub fn sort(&mut self) {
        self.errors.sort_unstable_by(|l, r| {
            if l.0 == r.0 {
                l.1.location().cmp(&r.1.location())
            }
            else {
                l.0.cmp(&r.0)
            }
        })
    }

    pub fn len(&self) -> usize {
        self.errors.len()
    }
}

impl fmt::Display for ErrorStore {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for err in &self.errors {
            if let Some(ref path) = err.0 {
                write!(f, "{}:{}:", path, err.1.location())?;
            }
            writeln!(f, "{}", err.1.inner())?;
        }
        Ok(())
    }
}


//------------ SharedErrorStore ----------------------------------------------

#[derive(Clone, Debug)]
pub struct SharedErrorStore(Arc<Mutex<ErrorStore>>);

impl SharedErrorStore {
    pub fn new() -> Self {
        SharedErrorStore::from_store(ErrorStore::new())
    }

    pub fn from_store(store: ErrorStore) -> Self {
        SharedErrorStore(Arc::new(Mutex::new(store)))
    }

    pub fn push<E: Into<Error>>(&self, path: Option<&Path>, err: E) {
        self.0.lock().unwrap().push(path, err)
    }

    pub fn try_unwrap(self) -> Result<ErrorStore, Self> {
        match Arc::try_unwrap(self.0) {
            Ok(store) => Ok(store.into_inner().unwrap()),
            Err(err) => Err(SharedErrorStore(err))
        }
    }
}

