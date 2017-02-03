//! An owned path that can be cloned cheaply.
//!
//! We really only have this to (a) have a shorter name and (b) so we can
//! change it later more easily.

use std::{ops, path};
use std::sync::Arc;


#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Path(Arc<path::PathBuf>);

impl Path {
    pub fn new<P: AsRef<path::Path>>(path: P) -> Self {
        Path(Arc::new(path.as_ref().to_path_buf()))
    }

    pub fn from_owned(path: path::PathBuf) -> Self {
        Path(Arc::new(path))
    }

    pub fn join<P: AsRef<path::Path>>(&self,  path: P) -> Self {
        Path(Arc::new(self.0.join(path)))
    }
}

impl Clone for Path {
    fn clone(&self) -> Self {
        Path(self.0.clone())
    }
}

impl<'a> From<&'a path::Path> for Path {
    fn from(path: &'a path::Path) -> Self {
        Path(Arc::new(path.into()))
    }
}

impl From<path::PathBuf> for Path {
    fn from(path: path::PathBuf) -> Self {
        Path(Arc::new(path))
    }
}

impl AsRef<path::Path> for Path {
    fn as_ref(&self) -> &path::Path {
        &self.0
    }
}

impl ops::Deref for Path {
    type Target = path::Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

