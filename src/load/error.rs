
use std::fmt;
use ::documents::types::{Location, Marked};
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


/*
use std::fmt;
use yaml_rust::scanner::Marker;
use ::load::path::Path;


//------------ Error ---------------------------------------------------------

pub struct Error {
    source: Source,
    error: Box<fmt::Display>,
}

impl Error {
    pub fn new<E: fmt::Display + 'static>(source: Source, error: E) -> Self {
        Error {
            source: source,
            error: Box::new(error),
        }
    }

    pub fn from_str(source: Source, s: &str) -> Self {
        Error::new(source, String::from(s))
    }

    pub fn global<E: fmt::Display + 'static>(error: E) -> Self {
        Self::new(Source::Global, error)
    }

    pub fn file<E>(path: Path, error: E) -> Self
                    where E: fmt::Display + 'static {
        Self::new(Source::File{path: path}, error)
    }

    pub fn in_file<E>(path: Path, pos: Marker, error: E) -> Self
                       where E: fmt::Display + 'static {
        Self::new(Source::InFile{path: path, pos: pos}, error)
    }

    pub fn source(&self) -> &Source {
        &self.source
    }

    pub fn error(&self) -> &Box<fmt::Display> {
        &self.error
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err = format!("{}", self.error);
        f.debug_struct("Error")
         .field("source", &self.source)
         .field("error", &err)
         .finish()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.source {
            Source::Global => {
                write!(f, "{}", self.error)
            }
            Source::File{ref path} => {
                write!(f, "{}: {}", path.display(), self.error)
            }
            Source::InFile{ref path, ref pos} => {
                write!(f, "{}:{}:{}: {}", path.display(), pos.line(),
                                          pos.col(), self.error)
            }
        }
    }
}

impl<E: fmt::Display + 'static> From<(Source, E)> for Error {
    fn from(err: (Source, E)) -> Error {
        Error::new(err.0, err.1)
    }
}

impl<E: fmt::Display + 'static> From<(Path, E)> for Error {
    fn from(err: (Path, E)) -> Error {
        Error::file(err.0, err.1)
    }
}


//------------ Source --------------------------------------------------------

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Source {
    Global,
    File { path: Path },
    InFile { path: Path, pos: Marker },
}

impl Source {
    pub fn file(path: Path) -> Self {
        Source::File{path: path}
    }

    pub fn in_file(path: Path, pos: Marker) -> Self {
        Source::InFile {
            path: path,
            pos: pos,
        }
    }
}

impl From<Path> for Source {
    fn from(path: Path) -> Self {
        Source::File{path: path}
    }
}

impl<'a> From<&'a Path> for Source {
    fn from(path: &'a  Path) -> Self {
        Source::File{path: path.clone()}
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Source::Global => Ok(()),
            Source::File{ref path} => write!(f, "{}", path.display()),
            Source::InFile{ref path, ref pos} => {
                write!(f, "{}:{}:{}", path.display(), pos.line(),
                                      pos.col())
            }
        }
    }
}

*/
