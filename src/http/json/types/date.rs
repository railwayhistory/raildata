use httools::json::JsonObject;
use crate::types::date::{Date, EventDate, Precision};

impl Date {
    pub fn json(&self, json: &mut JsonObject) {
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
        json.object("date", |json| first.json(json));
        if self.len() > 1 {
            json.array("alternativeDates", |json| {
                for date in iter {
                    json.object(|json| date.json(json));
                }
            })
        }
    }
}

