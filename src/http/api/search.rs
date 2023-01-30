use std::cmp;
use std::str::FromStr;
use std::sync::Arc;
use httools::{Request, RequestPath, Response};
use httools::json::JsonBuilder;
use crate::document::combined;
use crate::http::state::State;
use crate::types::LanguageCode;


//------------ search --------------------------------------------------------

pub fn all(
    state: Arc<State>, request: Request, mut path: RequestPath
) -> Result<Response, Response> {
    match path.next_and_last() {
        Ok(Some("names")) => search_names(state, request, false),
        Ok(Some("coords")) => search_names(state, request, true),
        _ => Ok(Response::not_found())
    }
}

fn search_names(
    state: Arc<State>, request: Request, coord: bool
) -> Result<Response, Response> {
    let query = request.query();

    let q = match query.get_first("q") {
        Some(q) => q,
        _ => {
            return Ok(JsonBuilder::ok(|json| {
                json.member_array("items", |_| { })
            }))
        }
    };

    let lang = query.get_first("lang").and_then(|lang| {
        LanguageCode::from_str(lang).ok()
    }).unwrap_or(LanguageCode::ENG);

    let count = query.get_first("num").and_then(|num| {
        usize::from_str(num).ok()
    }).map(|count| cmp::min(count, 100)).unwrap_or(20);

    if coord {
        Ok(JsonBuilder::ok(|json| {
            json.member_array("items", |json| {
                let found = state.catalogue().search_name(
                    q
                ).filter_map(|(name, link)| {
                    let doc = link.data(state.store());
                    let meta = link.meta(state.store());
                    let coord = match meta {
                        combined::Meta::Point(ref meta) => meta.coord?,
                        _ => return None
                    };
                    Some((name, coord, doc))
                }).take(count);
                for (name, coord, doc) in found {
                    json.array_object(|json| {
                        json.member_str("name", name);
                        json.member_str("type", doc.doctype());
                        json.member_str("title", doc.name(lang.into()));
                        json.member_str("key", doc.key());
                        json.member_object("coords", |json| {
                            json.member_raw("lat", coord.lat);
                            json.member_raw("lon", coord.lon);
                        });
                    })
                }
            })
        }))
    }
    else {
        Ok(JsonBuilder::ok(|json| {
            json.member_array("items", |json| {
                let found = state.catalogue().search_name(q).take(count);
                for (name, link) in found {
                    let doc = link.data(state.store());
                    json.array_object(|json| {
                        json.member_str("name", name);
                        json.member_str("type", doc.doctype());
                        json.member_str("title", doc.name(lang.into()));
                        json.member_str("key", doc.key());
                    })
                }
            })
        }))
    }
}

