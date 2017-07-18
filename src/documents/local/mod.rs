

pub mod code;
mod de;


use super::DocumentType;
use self::code::CountryCode;

pub fn is_valid_key(key: &str, doctype: DocumentType) -> bool {
    if !key.starts_with(doctype.key_prefix()) {
        return false;
    }
    let key = &key[doctype.key_prefix().len()..];
    let (country, key) = match CountryCode::from_prefix(key) {
        Some(val) => val,
        None => return false
    };
    match country {
        CountryCode::De => de::is_valid_key(key, doctype),
        _ => true
    }
}
