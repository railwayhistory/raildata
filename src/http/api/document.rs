use std::sync::Arc;
use httools::{Request, RequestPath, Response};
use httools::response::ContentType;
use crate::http::state::State;


pub fn all(
    state: Arc<State>, _request: Request, mut path: RequestPath
) -> Result<Response, Response> {
    match path.next_and_last() {
        Ok(Some(key)) => {
            match state.store().get(key) {
                Some(link) => {
                    Ok(Response::ok(
                        ContentType::JSON,
                        link.document(state.store()).json(state.store())
                    ))
                }
                None => Ok(Response::not_found())
            }
        }
        _ => Ok(Response::not_found())
    }
}

