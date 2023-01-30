use std::sync::Arc;
use httools::{Request, RequestPath, Response};
use httools::json::JsonBuilder;
use crate::http::state::State;


pub fn all(
    state: Arc<State>, _request: Request, mut path: RequestPath
) -> Result<Response, Response> {
    match path.next_and_last() {
        Ok(Some(key)) => {
            match state.store().get(key) {
                Some(link) => {
                    Ok(JsonBuilder::ok(|json| {
                        json.object(|json| {
                            link.document(state.store()).json(json, &state)
                        })
                    }))
                }
                None => Ok(Response::not_found())
            }
        }
        _ => Ok(Response::not_found())
    }
}

