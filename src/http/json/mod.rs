pub mod document;
pub mod types;


pub trait StateBuildJson {
    fn json(
        &self,
        json: &mut httools::json::JsonValue,
        state: &crate::http::state::State,
    );
}

impl<T: StateBuildJson> StateBuildJson for crate::types::Marked<T> {
    fn json(
        &self,
        json: &mut httools::json::JsonValue,
        state: &crate::http::state::State,
    ) {
        self.as_value().json(json, state)
    }
}

impl StateBuildJson for String {
    fn json(
        &self,
        json: &mut httools::json::JsonValue,
        _state: &crate::http::state::State,
    ) {
        json.string(self)
    }
}

impl StateBuildJson for u8 {
    fn json(
        &self,
        json: &mut httools::json::JsonValue,
        _state: &crate::http::state::State,
    ) {
        json.raw(self)
    }
}

impl<T: StateBuildJson> StateBuildJson for Option<T> {
    fn json(
        &self,
        json: &mut httools::json::JsonValue,
        state: &crate::http::state::State,
    ) {
        match self.as_ref() {
            Some(some) => some.json(json, state),
            None => json.raw("null")
        }
    }
}

