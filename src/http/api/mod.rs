mod document;
mod index;
mod search;

//------------ serve ---------------------------------------------------------

use std::net::SocketAddr;
use std::sync::Arc;
use httools::Response;
use super::state::State;

pub async fn serve(addr: SocketAddr, state: Arc<State>) {
    httools::server::serve(addr, state, |state, request| {
        async move {
            request.require_get()?;
            let path = request.path();
            match path.segment() {
                "document" => self::document::all(state, request, path),
                "index" => self::index::all(state, request, path),
                "search" => self::search::all(state, request, path),
                _ => Ok(Response::not_found())
            }
        }
    }).await
}

