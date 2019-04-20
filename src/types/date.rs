//! The date type.

use std::{cmp, fmt, ops, str};
use std::str::FromStr;
use crate::load::yaml::{FromYaml, Value};
use crate::load::report::{Failed, PathReporter};
use super::list::List;
use super::marked::Marked;


//------------ Temporary: Stand-in Dates -------------------------------------

fn stand_in_dates(key: &str) -> Option<Date> {
    match key {
        "de.Bft"
            => Some(Date::new(1990, None, None, Precision::Circa, false)),
        "dd.rkl.65"
            => Some(Date::new(1965, None, None, Precision::Circa, false)),
        "de.lknr.30"
            => Some(Date::new(1930, None, None, Precision::Circa, false)),
        "de.lknr.kb"
            => Some(Date::new(1935, None, None, Precision::Circa, false)),
        "de.vzg.dr"
            => Some(Date::new(1990, None, None, Precision::Circa, false)),
        "de.ds100.dr"
            => Some(Date::new(1992, Some(1), Some(1),
                              Precision::Exact, false)),
        "org.de.DB.start"
            => Some(Date::new(1949, Some(7), Some(1),
                              Precision::Exact, false)),
        "org.dd.DR.start"
            => Some(Date::new(1949, None, None, Precision::Exact, false)),
        _ => None,
    }
}


//------------ Precision -----------------------------------------------------

/// The precision of a date.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Precision {
    /// Exactly the given date.
    ///
    /// Since month and day of month can be `None`, this doesn’t necessarily
    /// mean an exact day.
    Exact,

    /// Sometime before or after but relatively close to the given date.
    Circa,

    /// Sometime before the given date.
    Before,

    /// Sometime after the given date.
    After
}

impl Ord for Precision {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        use self::Precision::*;

        match (*self, *other) {
            (Before, Before) | (Exact, Exact) | (Circa, Circa) |
                (After, After) => cmp::Ordering::Equal,
            (Before, _) => cmp::Ordering::Less,
            (Exact, Before) => cmp::Ordering::Greater,
            (Exact, _) => cmp::Ordering::Less,
            (Circa, Before) | (Circa, Exact) => cmp::Ordering::Greater,
            (Circa, _) => cmp::Ordering::Less,
            (_, _) => cmp::Ordering::Greater,
        }
    }
}

impl PartialOrd for Precision {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}


//------------ Date ----------------------------------------------------------

/// A date.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Date {
    year: i16,
    month: Option<u8>,
    day: Option<u8>,
    precision: Precision,
    doubt: bool,
}

impl Date {
    pub fn new(year: i16, month: Option<u8>, day: Option<u8>,
               precision: Precision, doubt: bool) -> Self {
        Date { year, month, day, precision, doubt }
    }

    pub fn from_year(year: i16) -> Self {
        Date {
            year, month: None, day: None,
            precision: Precision::Exact,
            doubt: false
        }
    }

    pub fn year(&self) -> i16 { self.year }
    pub fn month(&self) -> Option<u8> { self.month }
    pub fn day(&self) -> Option<u8> { self.day }
    pub fn precision(&self) -> Precision { self.precision }
    pub fn doubt(&self) -> bool { self.doubt }

    pub fn is_valid(&self) -> bool {
        if let Some(month) = self.month {
            if month < 1 || month > 12 { return false }
            if let Some(day) = self.day {
                if day < 1 { return false }
                match month {
                    1 | 3 | 5 | 7 | 8 | 10 | 12 => {
                        if day > 31 { return false }
                    }
                    2 => {
                        if self.is_leap() {
                            if day > 29 { return false }
                        }
                        else {
                            if day > 28 { return false }
                        }
                    }
                    _ => {
                        if day > 30 { return false }
                    }
                }
            }
            true
        }
        else {
            // If we don’t have a month we can’t have a day either.
            self.day.is_none()
        }
    }

    pub fn is_leap(&self) -> bool {
        (self.year % 4 == 0 && self.year % 100 != 0) || self.year % 400 == 0
    }
}

impl<C> FromYaml<C> for Marked<Date> {
    fn from_yaml(
        value: Value,
        _: &C,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let value = match value.try_into_integer() {
            Ok(year) => {
                return year.try_map(|year| {
                    if year <= ::std::u16::MAX as i64
                    && year >= ::std::u16::MIN as i64 {
                        Ok(Date::from_year(year as i16))
                    }
                    else { Err(FromStrError) }
                }).map_err(|err| { report.error(err); Failed });
            }
            Err(value) => value
        };

        let value = value.into_string(report)?;
        value.try_map(|plain| {
            if let Some(date) = stand_in_dates(&plain) {
                Ok(date)
            }
            else {
                Date::from_str(&plain)
            }
        }).map_err(|err| { report.error(err); Failed })
    }
}


impl Ord for Date {
    /// Returns the ordering between `self` and `other`.
    ///
    /// Because of all the fuzziness allowed in dates, ordering is rather
    /// arbitrary. Here are the rules:
    ///
    /// * Dates are ordered by year first.
    /// * If the years are equal, they are ordered by the value of the
    ///   month with a missing month being greater than any given month.
    /// * If the years and months are equal, they are ordered by the
    ///   value of the day with a missing day being greater than any given
    ///   day.
    /// * If years, months, and days are equal, they are ordered by precision
    ///   with `Before < Exact < Circa < After`.
    /// * If years, months, days, and precision are equal, they are ordered
    ///   by doubt where dates with doubt are greater than those without.
    /// * If years, months, days, precision, and doubt are equal, the dates
    ///   are equal.
    ///  
    fn cmp(&self, other: &Date) -> cmp::Ordering {
        if self.year != other.year {
            self.year.cmp(&other.year)
        }
        else if self.month != other.month {
            match (self.month, other.month) {
                (None, Some(_)) => cmp::Ordering::Greater,
                (Some(_), None) => cmp::Ordering::Less,
                (None, None) => cmp::Ordering::Equal,
                (Some(s), Some(o)) => s.cmp(&o)
            }
        }
        else if self.day != other.day {
            match (self.day, other.day) {
                (None, Some(_)) => cmp::Ordering::Greater,
                (Some(_), None) => cmp::Ordering::Less,
                (None, None) => cmp::Ordering::Equal,
                (Some(s), Some(o)) => s.cmp(&o)
            }
        }
        else if self.precision != other.precision {
            self.precision.cmp(&other.precision)
        }
        else if self.doubt != other.doubt {
            if self.doubt { cmp::Ordering::Greater }
            else { cmp::Ordering::Less }
        }
        else {
            cmp::Ordering::Equal
        }
    }
}

impl PartialOrd for Date {
    fn partial_cmp(&self, other: &Date) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl str::FromStr for Date {
    type Err = FromStrError;
    
    /// Converts the string representation of a date into a date.
    ///
    /// The format is `[abc]?\d{4}(\d{2}-(\d{2})?)?[?]?`
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse(mut s: &str) -> Result<Date, ::std::num::ParseIntError> {
            // Precision
            //
            let prec = if s.starts_with('c') {
                s = &s[1..];
                Precision::Circa
            }
            else if s.starts_with('<') || s.starts_with('b') {
                s = &s[1..];
                Precision::Before
            }
            else if s.starts_with('>') || s.starts_with('a') {
                s = &s[1..];
                Precision::After
            }
            else {
                Precision::Exact
            };

            // Doubt
            //
            let doubt = if s.ends_with('?') {
                s = &s[..s.len() - 1];
                true
            }
            else {
                false
            };

            // Year
            //
            let year = if let Some(pos) = s.find('-') {
                let (year_str, rest) = s.split_at(pos);
                s = &rest[1..];
                i16::from_str(year_str)?
            }
            else {
                return Ok(Date::new(i16::from_str(s)?, None, None, prec,
                                    doubt))
            };

            // Month
            //
            let month = if let Some(pos) = s.find('-') {
                let (month_str, rest) = s.split_at(pos);
                s = &rest[1..];
                Some(u8::from_str(month_str)?)
            }
            else {
                return Ok(Date::new(year, Some(u8::from_str(s)?), None,
                                    prec, doubt))
            };

            Ok(Date::new(year, month, Some(u8::from_str(s)?), prec, doubt))
        }
        
        let date = parse(s)?;
        if date.is_valid() {
            Ok(date)
        }
        else {
            Err(FromStrError)
        }
    }
}


//------------ EventDate -----------------------------------------------------

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EventDate(List<Marked<Date>>);

impl EventDate {
    /// Returns the sort order of two event dates.
    ///
    /// This is not the same as the ordering of those dates.
    pub fn sort_cmp(&self, other: &Self) -> cmp::Ordering {
        match (self.0.first(), other.0.first()) {
            (None, None) => cmp::Ordering::Equal,
            (None, Some(_)) => cmp::Ordering::Less,
            (Some(_), None) => cmp::Ordering::Greater,
            (Some(left), Some(right)) => left.cmp(right)
        }
    }
}

impl ops::Deref for EventDate {
    type Target = List<Marked<Date>>;

    fn deref(&self) -> &List<Marked<Date>> {
        &self.0
    }
}

impl<C> FromYaml<C> for EventDate {
    fn from_yaml(
        value: Value,
        context: &C,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        match value.try_into_null() {
            Ok(_value) => Ok(EventDate(List::new())),
            Err(value) => {
                Ok(EventDate(
                    List::from_yaml(
                        value, context, report
                    )?
                ))
            }
        }
    }
}

impl PartialEq for EventDate {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_slice() == other.0.as_slice()
    }
}

impl Eq for EventDate { }


//------------ DateError -----------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct FromStrError;

impl fmt::Display for FromStrError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("invalid date")
    }
}

impl From<::std::num::ParseIntError> for FromStrError {
    fn from(_: ::std::num::ParseIntError) -> FromStrError {
        FromStrError
    }
}

