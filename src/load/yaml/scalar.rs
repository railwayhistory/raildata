
use std::str::FromStr;
use yaml_rust::scanner::{Marker, TokenType};
use ::collection::CollectionBuilder;
use ::load::path::Path;
use super::stream::{FromYaml, Value, ValueItem};
use super::mapping::Mapping;
use super::vars::Vars;


//------------ Parsing scalars nodes -----------------------------------------

pub fn parse_scalar(value: &str, tag: &Option<TokenType>, path: &Path,
                    mark: Marker, vars: &Vars)
                    -> Result<ValueItem, String> {
    // Our own tags.
    if is_tag(tag, "date") {
        date(path, value, mark)
    }
    else if is_tag(tag, "keyloc") {
        keyloc(path, value, mark)
    }
    else if is_tag(tag, "keyref") {
        keyref(path, value, mark)
    }
    else if is_tag(tag, "noderef") {
        noderef(path, value, mark)
    }
    else if is_tag(tag, "pathref") {
        pathref(path, value, mark)
    }
    else if is_tag(tag, "var") {
        var(vars, value)
    }

    // Core schema types
    else if let Some(res) = null(value, tag) {
        res?;
        Ok(ValueItem::new(Value::Null, path.clone(), Some(mark)))
    }
    else if let Some(res) = boolean(value, tag) {
        Ok(ValueItem::new(Value::Bool(res?), path.clone(), Some(mark)))
    }
    else if let Some(res) = int(value, tag) {
        Ok(ValueItem::new(Value::Int(res?), path.clone(), Some(mark)))
    }
    else if let Some(res) = float(value, tag) {
        Ok(ValueItem::new(Value::Float(Float::new(res?)),
                          path.clone(), Some(mark)))
    }
    else if let Some(res) = string(value, tag) {
        Ok(ValueItem::new(Value::String(res?), path.clone(), Some(mark)))
    }
    else {
        Err(format!("unknown scalar value {:?}", tag))
    }
}

fn date(path: &Path, value: &str, mark: Marker)
        -> Result<ValueItem, String> {
    Ok(ValueItem::new(Value::String(value.into()), path.clone(), Some(mark)))
}

fn keyloc(path: &Path, value: &str, mark: Marker)
          -> Result<ValueItem, String> {
    let mut value = value.split_whitespace();
    let line = value.next().ok_or("invalid !keyloc value")?;
    let location = value.next().ok_or("invalid !keyloc value")?;
    if value.next().is_some() {
        return Err("invalid !keyloc value".into())
    }

    let mut res = Mapping::new();
    res.insert("line".into(), ValueItem::new(Value::String(line.into()),
                                             path.clone(), Some(mark)));
    res.insert("location".into(),
               ValueItem::new(Value::String(location.into()), path.clone(),
                              Some(mark)));
    Ok(ValueItem::new(Value::Mapping(res), path.clone(), Some(mark)))
}

fn keyref(path: &Path, value: &str, mark: Marker)
          -> Result<ValueItem, String> {
    Ok(ValueItem::new(Value::String(value.into()), path.clone(), Some(mark)))
}

fn noderef(path: &Path, value: &str, mark: Marker)
           -> Result<ValueItem, String> {
    let mut value = value.split_whitespace();
    let path_key = value.next().ok_or("invalid !noderef value")?;
    let node = value.next().ok_or("invalid !noderef value")?;
    let side = value.next();
    let distance = match side {
        Some(_) => match value.next() {
            Some(side) => {
                if value.next().is_some() {
                    return Err("invalid !noderef value".into())
                }
                side
            }
            None => "0",
        },
        None => "0"
    };
    let distance = f64::from_str(distance)
                       .map_err(|_| "invalid distance in !noderef")?;

    let mut res = Mapping::new();
    res.insert("path".into(),
               ValueItem::new(Value::String(path_key.into()), path.clone(),
                              Some(mark)));
    res.insert("node".into(),
               ValueItem::new(Value::String(node.into()), path.clone(),
                              Some(mark)));
    if let Some(side) = side {
        res.insert("side".into(),
                   ValueItem::new(Value::String(side.into()), path.clone(),
                                  Some(mark)));
    }
    res.insert("distance".into(),
               ValueItem::new(Value::Float(Float::new(distance)), path.clone(),
                              Some(mark)));
    Ok(ValueItem::new(Value::Mapping(res), path.clone(), Some(mark)))
}

fn pathref(path: &Path, value: &str, mark: Marker)
           -> Result<ValueItem, String> {
    let mut value = value.split_whitespace();
    let path_key = value.next().ok_or("invalid !noderef value")?;
    let start = value.next().ok_or("invalid !noderef value")?;
    let end = value.next().ok_or("invalid !noderef value")?;
    let offset = if let Some(word) = value.next() {
        if value.next().is_some() {
            return Err("invalid !keyloc value".into())
        }
        match Float::from_str(word) {
            Ok(x) => Some(x),
            Err(_) => return Err("invalid !keyloc value".into())
        }
    }
    else {
        None
    };

    let mut res = Mapping::new();
    res.insert("path".into(),
               ValueItem::new(Value::String(path_key.into()), path.clone(),
                              Some(mark)));
    res.insert("start".into(),
               ValueItem::new(Value::String(start.into()), path.clone(),
                              Some(mark)));
    res.insert("end".into(),
               ValueItem::new(Value::String(end.into()), path.clone(),
                              Some(mark)));
    if let Some(offset) = offset {
        res.insert("offset".into(),
                   ValueItem::new(Value::Float(offset), path.clone(),
                                  Some(mark)));
    }
    Ok(ValueItem::new(Value::Mapping(res), path.clone(), Some(mark)))
}

fn var(vars: &Vars, value: &str)
       -> Result<ValueItem, String> {
    vars.get(value)
          .ok_or_else(|| format!("unresolved variable {}", value))
}

fn null(value: &str, tag: &Option<TokenType>) -> Option<Result<(), String>> {
    if is_sec_tag(tag, "null") {
        Some(Ok(())) }
    else if is_plain(tag) {
        match value {
            "" | "null" | "Null" | "NULL" | "~" => Some(Ok(())),
            _ => None
        }
    }
    else {
        None
    }
}

pub fn boolean(value: &str, tag: &Option<TokenType>)
               -> Option<Result<bool, String>> {
    fn parse(value: &str) -> Result<bool, ()> {
        match value {
            "true" | "True" | "TRUE" => Ok(true),
            "false" | "False" | "FALSE" => Ok(false),
            _ => Err(()),
        }
    }

    if is_sec_tag(tag, "bool") {
        Some(parse(value).map_err(|_| "invalid boolean value".into()))
    }
    else if is_plain(tag) {
        parse(value).ok().map(Ok)
    }
    else {
        None
    }
}

pub fn int(value: &str, tag: &Option<TokenType>)
            -> Option<Result<i64, String>> {
    if is_sec_tag(tag, "int") {
        // XXX If I read the spec correctly, the tagged int can only
        //     read decimal numbers.
        Some(value.parse().map_err(|_| "invalid integer value".into()))
    }
    else if is_plain(tag) {
        if value.starts_with("0o") {
            i64::from_str_radix(&value[2..], 8).ok().map(Ok)
        }
        else if value.starts_with("0x") {
            i64::from_str_radix(&value[2..], 16).ok().map(Ok)
        }
        else {
            i64::from_str_radix(value, 10).ok().map(Ok)
        }
    }
    else {
        None
    }
}

pub fn float(value: &str, tag: &Option<TokenType>)
            -> Option<Result<f64, String>> {
    
    fn parse(value: &str) -> Result<f64, ::std::num::ParseFloatError> {
        match value {
            ".inf" | ".Inf" | ".INF" |
            "+.inf" | "+.Inf" | "+.INF"
                => Ok(::std::f64::INFINITY),
            "-.inf" | "-.Inf" | "-.INF"
                => Ok(-::std::f64::INFINITY),
            ".nan" | ".NaN" | ".NAN"
                => Ok(::std::f64::NAN),
            _ => value.parse()
        }
    }

    if is_sec_tag(tag, "float") {
        Some(parse(value).map_err(|_| "invalid float value".into()))
    }
    else if is_plain(tag) {
        parse(value).ok().map(Ok)
    }
    else {
        None
    }
}

pub fn string(value: &str, tag: &Option<TokenType>)
              -> Option<Result<String, String>> {
    if is_sec_tag(tag, "str") {
        Some(Ok(value.into()))
    }
    else if is_plain(tag) {
        Some(Ok(value.into()))
    }
    else {
        None
    }
}


//------------- Helper Functions ---------------------------------------------

fn is_tag(tag: &Option<TokenType>, suffix: &str) -> bool {
    match *tag {
        Some(TokenType::Tag(ref h, ref s)) => {
            h == "!" && s == suffix
        }
        _ => false
    }
}

fn is_sec_tag(tag: &Option<TokenType>, suffix: &str) -> bool {
    match *tag {
        Some(TokenType::Tag(ref h, ref s)) => {
            h == "!!" && s == suffix
        }
        _ => false
    }
}

fn is_plain(tag: &Option<TokenType>) -> bool {
    match *tag {
        Some(TokenType::Tag(..)) => false,
        _ => true
    }
}


//------------ Float ---------------------------------------------------------

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Float(String);

impl Float {
    pub fn new(f: f64) -> Self {
        Float(format!("{}", f))
    }

    pub fn to_f64(&self) -> f64 {
        self.0.parse::<f64>().unwrap()
    }
}

impl From<f64> for Float {
    fn from(f: f64) -> Self {
        Float::new(f)
    }
}

impl From<Float> for f64 {
    fn from(f: Float) -> Self {
        f.to_f64()
    }
}

impl FromStr for Float {
    type Err = ::std::num::ParseFloatError;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        src.parse::<f64>()?;
        Ok(Float(src.into()))
    }
}

impl FromYaml for Float {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        item.into_float(builder)
    }
}
