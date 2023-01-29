use std::net::SocketAddr;
use std::sync::Arc;
use httools::{RequestPath, Response};
use httools::response::ContentType;
use httools::server::serve;
use tokio::runtime::Runtime;
use crate::store::FullStore;
use crate::catalogue::Catalogue;

pub struct State {
    store: FullStore,

    #[allow(dead_code)]
    catalogue: Catalogue,
}

impl State {
    pub fn new(store: FullStore, catalogue: Catalogue) -> Self {
        State { store, catalogue }
    }

    pub fn new_arc(store: FullStore, catalogue: Catalogue) -> Arc<Self> {
        Arc::new(Self::new(store, catalogue))
    }
}

pub fn http(addr: SocketAddr, state: Arc<State>) {
    let rt = Runtime::new().unwrap();

    rt.block_on(
        serve(addr, state, |state, request| {
            async move {
                let path = request.path();
                match path.segment() {
                    "document" => document(state, path),
                    _ => Ok(Response::not_found())
                }
            }
        })
    );
}

fn document(
    state: Arc<State>, mut path: RequestPath
) -> Result<Response, Response> {
    match path.next_and_last() {
        Ok(Some(key)) => {
            match state.store.get(key) {
                Some(link) => {
                    Ok(Response::ok(
                        ContentType::JSON,
                        link.document(&state.store).json(&state.store)
                    ))
                }
                None => Ok(Response::not_found())
            }
        }
        _ => Ok(Response::not_found())
    }
}

