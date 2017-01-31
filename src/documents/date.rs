//! The date type.

use std::{cmp, str};
use ::collection::CollectionBuilder;
use ::load::yaml::{FromYaml, ValueItem};


//------------ Precision -----------------------------------------------------

/// The precision of a date.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
#[derive(Clone, Debug, Eq, PartialEq)]
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
        Date{year: year, month: month, day: day, precision: precision,
             doubt: doubt}
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

impl FromYaml for Date {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        let item = item.into_string_item(builder)?;
        match str::FromStr::from_str(&item) {
            Ok(res) => Ok(res),
            Err(err) => {
                builder.error((item.source(), err));
                Err(())
            }
        }
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
    type Err = String;
    
    /// Converts the string representation of a date into a date.
    ///
    /// The format is `[c|<|>]year[-month[-day]][?]`
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse(mut s: &str) -> Result<Date, ::std::num::ParseIntError> {
            // Precision
            //
            let prec = if s.starts_with('c') {
                s = &s[1..];
                Precision::Circa
            }
            else if s.starts_with('<') {
                s = &s[1..];
                Precision::Before
            }
            else if s.starts_with('>') {
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
        
        let date = match parse(s) {
            Ok(date) => date,
            Err(_) => return Err("invalid date".into())
        };

        if date.is_valid() {
            Ok(date)
        }
        else {
            Err("invalid date".into())
        }
    }
}
