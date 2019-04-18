//! Reporting during loading.

use std::{fmt, ops, path};
use std::fmt::Display;
use std::sync::{Arc, Mutex};
use crate::types::{IntoMarked, Location, Marked};


//------------ Severity ------------------------------------------------------

/// Severity of a notice.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Severity {
    /// The notice represents a fatal error.
    Fatal,

    /// The notice represents a regular error.
    Error,

    /// The notice represents a warning.
    Warning,

    /// The notice is only informational.
    Info,
}


//------------ Stage --------------------------------------------------------

/// The loading stage of the error.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Stage {
    /// Parsing of raw data.
    Parse = 0,

    /// Translation of parsed data into its structural components.
    Translate = 1,

    /// Cross-linking between documents.
    Crosslink = 2,

    /// Verify document sanity.
    Verify = 3,
}


//------------ Origin --------------------------------------------------------

/// The origin location of a notice.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Origin {
    path: Path,
    location: Location,
}

impl Origin {
    pub fn new(path: Path, location: Location) -> Self {
        Origin { path, location }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn location(&self) -> Location {
        self.location
    }
}

impl Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.path.display(), self.location)
    }
}


//------------ Path ----------------------------------------------------------

/// A file path.
///
/// This type wraps an owned path into an arc for cheaper copying.
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

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0.display(), f)
    }
}


//------------ Message -------------------------------------------------------

pub trait Message: Display + Send + 'static { }

impl<T: Display + Send + 'static> Message for T { }


//------------ Notice --------------------------------------------------------

/// A single element of a report.
pub struct Notice {
    /// Severity of the notice.
    severity: Severity,

    /// Stage of the notice.
    stage: Stage,

    /// The optional location of where the reportable event happened.
    origin: Option<Origin>,

    /// The message of the report.
    message: Box<Message>,
}

impl Notice {
    pub fn new<M: Message>(
        severity: Severity,
        stage: Stage,
        origin: Option<Origin>,
        message: M
    ) -> Self {
        Notice { severity, stage, origin, message: Box::new(message) }
    }

    pub fn severity(&self) -> Severity {
        self.severity
    }

    pub fn stage(&self) -> Stage {
        self.stage
    }

    pub fn origin(&self) -> Option<&Origin> {
        self.origin.as_ref()
    }

    pub fn message(&self) -> &Box<Message> {
        &self.message
    }
}

impl Display for Notice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref origin) = self.origin {
            write!(f, "{}: {}", origin, self.message)
        }
        else {
            self.message.fmt(f)
        }
    }
}


//------------ Report --------------------------------------------------------

/// A report is a collection of notices.
pub struct Report {
    notices: Vec<Notice>,
    stage_count: [usize; 4],
}

impl Report {
    pub fn new() -> Self {
        Report {
            notices: Vec::new(),
            stage_count: [0; 4],
        }
    }

    pub fn notice(&mut self, notice: Notice) {
        self.stage_count[notice.stage as usize] += 1;
        self.notices.push(notice)
    }

    pub fn sort(&mut self) {
        self.notices.sort_by(|l, r| l.origin.cmp(&r.origin))
    }

    pub fn has_stage(&self, stage: Stage) -> bool {
        self.stage_count[stage as usize] > 0
    }

    pub fn stage_count(&self, stage: Stage) -> usize {
        self.stage_count[stage as usize]
    }
}

impl ops::Deref for Report {
    type Target = [Notice];

    fn deref(&self) -> &Self::Target {
        self.notices.as_ref()
    }
}


//------------ Reporter ------------------------------------------------------

/// A type allowing access to a report.
///
/// This type doesn’t allow adding notices to the report just yet. You need
/// to convert it into a `StageReporter` first.
#[derive(Clone)]
pub struct Reporter {
    report: Arc<Mutex<Report>>,
}

impl Reporter {
    pub fn new() -> Self {
        Reporter {
            report: Arc::new(Mutex::new(Report::new()))
        }
    }

    /// Unwraps the report from the reporter.
    ///
    /// If there are currently other reporters for the same report, returns
    /// an error containing a reporter.
    ///
    /// # Panics
    ///
    /// The method panics if any of the reporters for the same report had
    /// previously paniced while accessing the report.
    pub fn try_unwrap(self) -> Result<Report, Self> {
        let report = match Arc::try_unwrap(self.report) {
            Ok(report) => report,
            Err(report) => return Err(Reporter { report }),
        };
        Ok(report.into_inner().unwrap())
    }

    pub fn unwrap(self) -> Report {
        match self.try_unwrap() {
            Ok(some) => some,
            Err(_) => panic!("cannot unwrap reporter")
        }
    }

    pub fn stage(self, stage: Stage) -> StageReporter {
        StageReporter::new(self, stage)
    }

    pub fn is_empty(&self) -> bool {
        self.report.lock().unwrap().is_empty()
    }

    fn notice(&mut self, notice: Notice) {
        self.report.lock().unwrap().notice(notice)
    }
}


//------------ StageReporter ------------------------------------------------

/// A reporter bound to a specific processing stage.
#[derive(Clone)]
pub struct StageReporter {
    reporter: Reporter,
    stage: Stage,
}

impl StageReporter {
    pub fn new(reporter: Reporter, stage: Stage) -> Self {
        StageReporter { reporter, stage }
    }

    pub fn unwrap(self) -> Reporter {
        self.reporter
    }

    pub fn notice<M: Message>(
        &mut self,
        severity: Severity,
        origin: Option<Origin>, 
        message: M,
    ) {
        self.reporter.notice(
            Notice::new(severity, self.stage, origin, message)
        )
    }

    pub fn with_path(self, path: Path) -> PathReporter {
        PathReporter::new(self, path)
    }

    pub fn fatal<M: Message>(&mut self, message: M) {
        self.notice(Severity::Fatal, None, message)
    }

    pub fn error<M: Message>(&mut self, message: M) {
        self.notice(Severity::Error, None, message)
    }

    pub fn warning<M: Message>(&mut self, message: M) {
        self.notice(Severity::Warning, None, message)
    }

    pub fn info<M: Message>(&mut self, message: M) {
        self.notice(Severity::Info, None, message)
    }

    pub fn fatal_at<M: Message>(&mut self, origin: Origin, message: M) {
        self.notice(Severity::Fatal, Some(origin), message)
    }

    pub fn error_at<M: Message>(&mut self, origin: Origin, message: M) {
        self.notice(Severity::Error, Some(origin), message)
    }

    pub fn warning_at<M: Message>(&mut self, origin: Origin, message: M) {
        self.notice(Severity::Warning, Some(origin), message)
    }

    pub fn info_at<M: Message>(&mut self, origin: Origin, message: M) {
        self.notice(Severity::Info, Some(origin), message)
    }
}


//------------ PathReporter --------------------------------------------------

/// A reporter that is bound to a stage and path.
pub struct PathReporter {
    reporter: StageReporter,
    path: Path,
}

impl PathReporter {
    pub fn new(reporter: StageReporter, path: Path) -> Self {
        PathReporter { reporter, path }
    }

    pub fn path(&self) -> Path {
        self.path.clone()
    }

    pub fn origin(&self, location: Location) -> Origin {
        Origin::new(self.path.clone(), location)
    }

    pub fn unwrap(self) -> StageReporter {
        self.reporter
    }

    pub fn restage(self, stage: Stage) -> Self {
        self.reporter.unwrap().stage(stage).with_path(self.path)
    }

    pub fn global(&mut self) -> &mut StageReporter {
        &mut self.reporter
    }

    pub fn notice<M: Message>(
        &mut self,
        severity: Severity,
        message: Marked<M>
    ) {
        self.reporter.notice(
            severity,
            Some(Origin::new(self.path.clone(), message.location())),
            message.into_value()
        )
    }

    pub fn fatal<M: Message>(&mut self, message: Marked<M>) {
        self.notice(Severity::Fatal, message)
    }

    pub fn error<M: Message>(&mut self, message: Marked<M>) {
        self.notice(Severity::Error, message);
    }

    pub fn unmarked_error<M: Message>(&mut self, message: M) {
        self.error(message.unmarked())
    }

    pub fn warning<M: Message>(&mut self, message: Marked<M>) {
        self.notice(Severity::Warning, message)
    }

    pub fn unmarked_warning<M: Message>(&mut self, message: M) {
        self.warning(message.unmarked())
    }

    pub fn info<M: Message>(&mut self, message: Marked<M>) {
        self.notice(Severity::Info, message)
    }
}


//------------ Failed --------------------------------------------------------

/// An operation has failed with a report.
///
/// When an operation has failed after a notice has been reported, it is often
/// unncessary to be more specific about the actual failure reason. In these
/// cases, you can simply use this type as the error type.
#[derive(Clone, Copy, Debug)]
pub struct Failed;


//------------ ResultExt -----------------------------------------------------

pub trait ResultExt<T>: Sized {
    fn notice(
        self,
        severity: Severity,
        report: &mut PathReporter
    ) -> Result<T, Failed>;

    fn or_fatal(self, report: &mut PathReporter) -> Result<T, Failed> {
        self.notice(Severity::Fatal, report)
    }

    fn or_error(self, report: &mut PathReporter) -> Result<T, Failed> {
        self.notice(Severity::Error, report)
    }

    fn or_warning(self, report: &mut PathReporter) -> Result<T, Failed> {
        self.notice(Severity::Warning, report)
    }

    fn or_info(self, report: &mut PathReporter) -> Result<T, Failed> {
        self.notice(Severity::Info, report)
    }
}

impl<T, E: Message> ResultExt<T> for Result<T, Marked<E>> {
    fn notice(
        self,
        severity: Severity,
        report: &mut PathReporter
    ) -> Result<T, Failed> {
        self.map_err(|err| {
            report.notice(severity, err);
            Failed
        })
    }
}

