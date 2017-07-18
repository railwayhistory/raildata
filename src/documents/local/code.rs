
pub enum CountryCode {
    At,
    Be,
    Ch,
    Cz,
    Dd,
    De,
    Dk,
    Fr,
    Gb,
    Lt,
    Nl,
    No,
    Pl,
    Ru,
    Int,
}

impl CountryCode {
    pub fn from_prefix(key: &str) -> Option<(Self, &str)> {
        let (country, key) = 
            if      key.starts_with("at") { (CountryCode::At, &key[2..]) }
            else if key.starts_with("be") { (CountryCode::Be, &key[2..]) }
            else if key.starts_with("ch") { (CountryCode::Ch, &key[2..]) }
            else if key.starts_with("cz") { (CountryCode::Cz, &key[2..]) }
            else if key.starts_with("dd") { (CountryCode::Dd, &key[2..]) }
            else if key.starts_with("de") { (CountryCode::De, &key[2..]) }
            else if key.starts_with("dk") { (CountryCode::Dk, &key[2..]) }
            else if key.starts_with("fr") { (CountryCode::Fr, &key[2..]) }
            else if key.starts_with("gb") { (CountryCode::Gb, &key[2..]) }
            else if key.starts_with("lt") { (CountryCode::Lt, &key[2..]) }
            else if key.starts_with("nl") { (CountryCode::Nl, &key[2..]) }
            else if key.starts_with("no") { (CountryCode::No, &key[2..]) }
            else if key.starts_with("pl") { (CountryCode::Pl, &key[2..]) }
            else if key.starts_with("ru") { (CountryCode::Ru, &key[2..]) }
            else if key.starts_with("int") { (CountryCode::Int, &key[3..]) }
            else { return None };
        if key.is_empty() {
            Some((country, key))
        }
        else {
            if key.starts_with(".") {
                Some((country, &key[1..]))
            }
            else {
                None
            }
        }
    }
}

