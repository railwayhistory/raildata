//! Verifying line documents.

use ::load::report::{PathReporter, StageReporter};
use ::types::marked::IntoMarked;
use super::Line;


pub fn verify(line: &Line, report: &StageReporter) {
    let mut report = report.clone()
        .with_path(line.common.origin().path().clone());
    verify_sections(line, &mut report);
}

fn verify_sections(line: &Line, report: &mut PathReporter) {
    for event in line.events.iter() {
        for section in event.sections.iter() {
            if let Some(ref link) = section.start {
                if line.points.get_index(link.as_value()).is_none() {
                    report.error(IllegalSectionStart.marked(link.location()))
                }
            }
            if let Some(ref link) = section.end {
                if line.points.get_index(link.as_value()).is_none() {
                    report.error(IllegalSectionEnd.marked(link.location()))
                }
            }
        }
    }
}


//============ Errors ========================================================

#[derive(Clone, Debug, Fail)]
#[fail(display="section start not in points")]
pub struct IllegalSectionStart;

#[derive(Clone, Debug, Fail)]
#[fail(display="section end not in points")]
pub struct IllegalSectionEnd;

