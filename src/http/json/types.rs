use std::{fmt, hash};
use httools::json::{JsonObject, JsonValue};
use crate::http::json::StateBuildJson;
use crate::http::state::State;
use crate::types::date::{Date, EventDate, Precision};
use crate::types::local::{CodedText, CountryCode};
use crate::types::{List, Set};

impl Date {
    pub fn json(&self, json: &mut JsonValue) {
        json.object(|json| {
            json.raw("year", self.year());
            if let Some(month) = self.month() {
                json.raw("month", month);
                if let Some(day) = self.day() {
                    json.raw("day", day)
                }
            }
            if self.doubt() {
                json.raw("doubt", "true")
            }
            match self.precision() {
                Precision::Exact => {}
                Precision::Circa => { json.string("precision", "circa") }
                Precision::Before => { json.string("precision", "before") }
                Precision::After => { json.string("precision", "after") }
            }
        })
    }
}

impl EventDate {
    pub fn json(&self, json: &mut JsonObject) {
        let mut iter = self.iter();
        let first = match iter.next() {
            Some(date) => date,
            None => {
                json.raw("date", "null");
                return
            }
        };
        json.value("date", |json| first.json(json));
        if self.len() > 1 {
            json.array("alternativeDates", |json| {
                for date in iter {
                    json.value(|json| date.json(json));
                }
            })
        }
    }
}

impl StateBuildJson for CountryCode {
    fn json(&self, json: &mut JsonValue, _state: &State) {
        json.string(self.as_str())
    }
}

impl<C: Ord + fmt::Display> StateBuildJson for CodedText<C> {
    fn json(&self, json: &mut JsonValue, _state: &State) {
        if let Some(text) = self.as_plain() {
            json.string(text);
        }
        else {
            json.object(|json| {
                for (code, text) in self.iter() {
                    let code = match code {
                        Some(code) => code,
                        None => continue
                    };
                    json.string(code, text);
                }
            })
        }
    }
}

impl<T> List<T> {
    pub fn json_strings(&self, json: &mut JsonValue)
    where T: fmt::Display {
        json.array(|json| self.iter().for_each(|item| json.string(item)))
    }
}

impl<T: StateBuildJson> StateBuildJson for List<T> {
    fn json(&self, json: &mut JsonValue, state: &State) {
        json.array(|json| {
            self.iter().for_each(|item| {
                json.value(|json| item.json(json, state))
            })
        })
    }
}

impl<T: StateBuildJson + Eq + hash::Hash> StateBuildJson for Set<T> {
    fn json(&self, json: &mut JsonValue, state: &State) {
        json.array(|json| {
            self.iter().for_each(|item| {
                json.value(|json| item.json(json, state))
            })
        })
    }
}

