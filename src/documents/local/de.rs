use ::documents::DocumentType;

pub fn is_valid_key(key: &str, doctype: DocumentType) -> bool {
    match doctype {
        DocumentType::Line => is_valid_line_key(key),
        _ => true,
    }
}

fn is_valid_line_key(key: &str) -> bool {
    // Currently, all German line numbers are four ASCII digits
    let key = key.as_bytes();
    key.len() == 4
        && key[0] >= b'0' && key[0] <= b'9'
        && key[1] >= b'0' && key[0] <= b'9'
        && key[2] >= b'0' && key[0] <= b'9'
        && key[3] >= b'0' && key[0] <= b'9'
}
