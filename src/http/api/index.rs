use std::cmp;
use std::str::FromStr;
use std::sync::Arc;
use httools::{Request, RequestPath, Response};
use httools::json::JsonBuilder;
use crate::document::line;
use crate::http::state::State;
use crate::store::FullStore;
use crate::types::local::{CountryCode, LanguageCode};


//------------ index ---------------------------------------------------------

pub fn all(
    state: Arc<State>, request: Request, mut path: RequestPath
) -> Result<Response, Response> {
    match path.next() {
        Some("lines") => index_lines(state, request, path),
        _ => Ok(Response::not_found())
    }
}

fn index_lines(
    state: Arc<State>, request: Request, mut path: RequestPath
) -> Result<Response, Response> {
    fn build(
        iter: impl Iterator<Item = line::Link>,
        start: usize, end: usize, len: usize,
        lang: LanguageCode, store: &FullStore
    ) -> Result<Response, Response> {
        Ok(JsonBuilder::ok(|json| json.object(|json| {
            json.raw("start", start);
            json.raw("num", end - start);
            json.raw("len", len);
            json.array("items", |json| {
                for line in iter {
                    let line = line.document(store);
                    json.object(|json| {
                        json.string("key", line.data().key());
                        json.string("code", line.data().code());
                        json.string(
                            "title", line.data().name(lang.into())
                        );
                        json.array("junctions", |json| {
                            for point in line.junctions(store) {
                                json.object(|json| {
                                    json.string("key", point.data().key());
                                    json.string(
                                        "title", point.data().name(lang)
                                    );
                                })
                            }
                        })
                    })
                }
            })
        })))
    }

    let query = request.query();
    let start = query.get_first("start").and_then(|num| {
        usize::from_str(num).ok()
    }).unwrap_or(0);
    let count = query.get_first("num").and_then(|num| {
        usize::from_str(num).ok()
    });
    let lang = query.get_first("lang").and_then(|lang| {
        LanguageCode::from_str(lang).ok()
    }).unwrap_or(LanguageCode::ENG);

    match path.next_and_last() {
        Ok(Some(code)) => {
            let code = CountryCode::from_str(code).map_err(|_| {
                Response::not_found()
            })?;
            let country = state.catalogue().countries.get(&code).ok_or_else(|| {
                Response::not_found()
            })?;
            let list = &country.xrefs(state.store()).line_regions;
            let end = match count {
                Some(count) => cmp::min(list.len(), start + count),
                None => list.len(),
            };
            let range = &list[start..end];
            build(
                range.iter().map(|item| item.0),
                start, end, list.len(), lang,
                state.store()
            )
        }
        Ok(None) => {
            let list = &state.catalogue().lines;
            let end = match count {
                Some(count) => cmp::min(list.len(), start + count),
                None => list.len(),
            };
            let range = &list[start..end];
            build(
                range.iter().copied(),
                start, end, list.len(), lang,
                state.store()
            )
        }
        Err(_) => Ok(Response::not_found())
    }
}


