
use std::{f64, fmt, ops};
use std::collections::HashMap;
use yaml_rust::scanner::{Marker, ScanError, TokenType, TScalarStyle};
use yaml_rust::parser::{Event, MarkedEventReceiver, Parser};
use ::types::Marked;
use super::construct::{Constructable, ConstructContext, Failed};


//------------ Constructor ---------------------------------------------------

pub trait Constructor {
    fn construct(&mut self, doc: Value);
}

impl Constructor for Vec<Value> {
    fn construct(&mut self, doc: Value) {
        self.push(doc)
    }
}


//------------ Loader --------------------------------------------------------

pub struct Loader<C: Constructor> {
    constructor: C,
    nodes: Vec<Value>,
    keys: Vec<Option<Value>>,
}

impl<C: Constructor> Loader<C> {
    pub fn new(constructor: C) -> Self {
        Loader { constructor, nodes: Vec::new(), keys: Vec::new() }
    }

    pub fn load<I>(&mut self, source: I) -> Result<(), ScanError>
                where I: IntoIterator<Item=char> {
        let mut parser = Parser::new(source.into_iter());
        try!(parser.load(self, true));
        Ok(())
    }

    pub fn load_from_str(&mut self, source: &str) -> Result<(), ScanError> {
        self.load(source.chars())
    }
}

impl<C: Constructor> MarkedEventReceiver for Loader<C> {
    fn on_event(&mut self, ev: Event, mark: Marker) {
        match ev {
            Event::DocumentStart => {
                assert!(self.nodes.is_empty())
            }
            Event::DocumentEnd => {
                if let Some(node) = self.nodes.pop() {
                    self.constructor.construct(node)
                }
            }
            Event::SequenceStart(_) => {
                self.nodes.push(Value::sequence(mark))
            }
            Event::SequenceEnd => {
                let node = self.nodes.pop().unwrap();
                self.push_value(node);
            }
            Event::MappingStart(_) => {
                self.nodes.push(Value::mapping(mark));
                self.keys.push(None);
            }
            Event::MappingEnd => {
                self.keys.pop().unwrap();
                let node = self.nodes.pop().unwrap();
                self.push_value(node);
            }
            Event::Scalar(value, style, _, tag) => {
                let plain = style == TScalarStyle::Plain;
                self.push_value(
                    Value::scalar(
                        value, plain,
                        tag.and_then(|ttype| {
                            if let TokenType::Tag(x, y) = ttype {
                                Some((x, y))
                            }
                            else {
                                None
                            }
                        }),
                        mark))
            }
            Event::Alias(_) => {
                self.push_value(Value::alias(mark))
            }
            _ => { }
        }
    }
}

impl<C: Constructor> Loader<C> {
    fn push_value(&mut self, value: Value) {
        if self.nodes.is_empty() {
            self.nodes.push(value)
        }
        else {
            match *self.nodes.last_mut().unwrap().as_value_mut() {
                PlainValue::Sequence(ref mut sequence) => {
                    sequence.push(value)
                }
                PlainValue::Mapping(ref mut mapping) => {
                    if let Some(key) = self.keys.last_mut().unwrap().take() {
                        mapping.insert(key, value)
                    }
                    else {
                        *self.keys.last_mut().unwrap() = Some(value)
                    }
                }
                _ => unreachable!()
            }
        }
    }
}


//------------ PlainValue and Value ------------------------------------------

pub type Value = Marked<PlainValue>;

#[derive(Clone, Debug)]
pub enum PlainValue {
    Sequence(Sequence),
    Mapping(Mapping),
    Scalar(Scalar),
    Error(ValueError),
}

impl Value {
    pub fn sequence(mark: Marker) -> Self {
        Value::new(PlainValue::Sequence(Sequence::default()), mark.into())
    }

    pub fn mapping(mark: Marker) -> Self {
        Value::new(PlainValue::Mapping(Mapping::default()), mark.into())
    }

    pub fn scalar(value: String, plain: bool, tag: Option<(String, String)>,
                  mark: Marker) -> Self {
        match Scalar::new(value, plain, tag) {
            Ok(scalar) => Value::new(PlainValue::Scalar(scalar), mark.into()),
            Err(err) => Value::new(PlainValue::Error(err), mark.into())
        }
    }

    pub fn alias(mark: Marker) -> Self {
        Value::new(PlainValue::Error(ValueError::Alias), mark.into())
    }
}

impl Value {
    pub fn into_mapping(self, context: &mut ConstructContext)
                        -> Result<Marked<Mapping>, Failed> {
        self.try_map(|plain| {
            if let PlainValue::Mapping(res) = plain { Ok(res) }
            else { Err(YamlError::type_mismatch(Type::Mapping, plain)) }
        }).map_err(|err| { context.push_error(err); Failed })
    }

    pub fn try_into_mapping(self) -> Result<Marked<Mapping>, Self> {
        self.try_map(|plain| {
            if let PlainValue::Mapping(res) = plain { Ok(res) }
            else { Err(plain) }
        })
    }

    pub fn into_sequence(self, context: &mut ConstructContext)
                         -> Result<Marked<Sequence>, Failed> {
        self.try_map(|plain| {
            if let PlainValue::Sequence(res) = plain { Ok(res) }
            else { Err(YamlError::type_mismatch(Type::Sequence, plain)) }
        }).map_err(|err| { context.push_error(err); Failed })
    }

    pub fn try_into_sequence(self) -> Result<Marked<Sequence>, Self> {
        self.try_map(|plain| {
            if let PlainValue::Sequence(res) = plain { Ok(res) }
            else { Err(plain) }
        })
    }

    fn into_scalar(self, further: Type, context: &mut ConstructContext)
                   -> Result<Marked<Scalar>, Failed> {
        self.try_map(|plain| {
            if let PlainValue::Scalar(res) = plain { Ok(res) }
            else { Err(YamlError::type_mismatch(further, plain)) }
        }).map_err(|err| { context.push_error(err); Failed })
    }

    fn try_into_scalar(self) -> Result<Marked<Scalar>, Self> {
        self.try_map(|plain| {
            if let PlainValue::Scalar(res) = plain { Ok(res) }
            else { Err(plain) }
        })
    }

    pub fn into_string(self, context: &mut ConstructContext)
                       -> Result<Marked<String>, Failed> {
        self.into_scalar(Type::String, context)?.try_map(|scalar| {
            if let Scalar::String(res) = scalar { Ok(res) }
            else { Err(YamlError::type_mismatch(Type::String, scalar)) }
        }).map_err(|err| { context.push_error(err); Failed })
    }

    pub fn into_boolean(self, context: &mut ConstructContext)
                        -> Result<Marked<bool>, Failed> {
        self.into_scalar(Type::Boolean, context)?.try_map(|scalar| {
            if let Scalar::Boolean(res) = scalar { Ok(res) }
            else { Err(YamlError::type_mismatch(Type::Boolean, scalar)) }
        }).map_err(|err| { context.push_error(err); Failed })
    }

    pub fn into_float(self, context: &mut ConstructContext)
                      -> Result<Marked<f64>, Failed> {
        self.into_scalar(Type::Float, context)?.try_map(|scalar| {
            if let Scalar::Float(res) = scalar { Ok(res) }
            else { Err(YamlError::type_mismatch(Type::Float, scalar)) }
        }).map_err(|err| { context.push_error(err); Failed })
    }

    pub fn into_integer(self, context: &mut ConstructContext)
                        -> Result<Marked<i64>, Failed> {
        self.into_scalar(Type::Integer, context)?.try_map(|scalar| {
            if let Scalar::Integer(res) = scalar { Ok(res) }
            else { Err(YamlError::type_mismatch(Type::Integer, scalar)) }
        }).map_err(|err| { context.push_error(err); Failed })
    }

    pub fn try_into_integer(self) -> Result<Marked<i64>, Self> {
        self.try_into_scalar()?.try_map(|scalar| {
            if let Scalar::Integer(res) = scalar { Ok(res) }
            else { Err(PlainValue::Scalar(scalar)) }
        })
    }

    pub fn is_null(&self) -> bool {
        if let PlainValue::Scalar(Scalar::Null) = *self.as_value() { true }
        else { false }
    }
}


//------------ Mapping -------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct Mapping {
    items: HashMap<Marked<String>, Value>,
    errors: Option<Vec<Marked<ValueError>>>
}

impl Mapping {
    pub fn insert(&mut self, key: Value, value: Value) {
        let key = key.try_map(|key| match key {
            PlainValue::Scalar(Scalar::String(key)) => Ok(key),
            _ => Err(ValueError::InvalidMappingKey(key.into())),
        });
        match key {
            Ok(key) => { self.items.insert(key, value); }
            Err(err) => self.errors.get_or_insert_with(Vec::new).push(err)
        }
    }

    pub fn check(&mut self, context: &mut ConstructContext)
                 -> Result<(), Failed> {
        if let Some(ref mut errors) = self.errors {
            if !errors.is_empty() {
                for error in errors.drain(..) {
                    context.push_error(error)
                }
                return Err(Failed)
            }
        }
        Ok(())
    }
}

impl IntoIterator for Mapping {
    type Item = (Marked<String>, Value);
    type IntoIter = ::std::collections::hash_map::IntoIter<Marked<String>,
                                                           Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl IntoIterator for Marked<Mapping> {
    type Item = (Marked<String>, Value);
    type IntoIter = ::std::collections::hash_map::IntoIter<Marked<String>,
                                                           Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_value().items.into_iter()
    }
}


//------------ MarkedMapping -------------------------------------------------

/// A marked mapping.
///
/// This is a type alias that may spare you an extra import.
pub type MarkedMapping = Marked<Mapping>;

impl MarkedMapping {
    pub fn take<C>(&mut self, key: &str, context: &mut ConstructContext)
                   -> Result<C, Failed>
                where C: Constructable {
        if let Some(value) = self.as_value_mut().items.remove(key) {
            C::construct(value, context)
        }
        else {
            context.push_error(
                Marked::new(YamlError::MissingKey(key.into()),
                            self.location())
            );
            Err(Failed)
        }
    }

    pub fn take_default<C>(&mut self, key: &str,
                           context: &mut ConstructContext)
                           -> Result<C, Failed>
                        where C: Constructable + Default {
        if let Some(value) = self.as_value_mut().items.remove(key) {
            C::construct(value, context)
        }
        else {
            Ok(C::default())
        }
    }

    pub fn take_opt<C>(&mut self, key: &str, context: &mut ConstructContext)
                       -> Result<Option<C>, Failed>
                where C: Constructable {
        if let Some(value) = self.as_value_mut().items.remove(key) {
            C::construct(value, context).map(Some)
        }
        else {
            Ok(None)
        }
    }

    pub fn exhausted(mut self, context: &mut ConstructContext)
                     -> Result<(), Failed> {
        let mut failed = self.check(context).is_err();
        if !self.items.is_empty() {
            for (key, _) in self.into_value().items {
                context.push_error(key.map(|key| UnexpectedKey(key)));
            }
            failed = true;
        }
        if failed { Err(Failed) }
        else { Ok(()) }
    }
}


//------------ Sequence ------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct Sequence(Vec<Value>);

impl ops::Deref for Sequence {
    type Target = Vec<Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for Sequence {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for Sequence {
    type Item = Value;
    type IntoIter = ::std::vec::IntoIter<Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}


//------------ Scalar --------------------------------------------------------

#[derive(Clone, Debug)]
pub enum Scalar {
    String(String),
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
}

impl Scalar {
    pub fn new(value: String, plain: bool, tag: Option<(String, String)>)
               -> Result<Self, ValueError> {
        if !plain {
            Ok(Scalar::String(value))
        }
        else if let Some((handle, suffix)) = tag {
            if handle.as_str() == "!!" {
                match suffix.as_str() {
                    "str" => Ok(Scalar::String(value)),
                    "null" => Ok(Scalar::Null),
                    "bool" => value.parse().map(Scalar::Boolean)
                                   .map_err(|_| ValueError::InvalidBool),
                    "int" => value.parse().map(Scalar::Integer)
                                  .map_err(|_| ValueError::InvalidInt),
                    "float" => value.parse().map(Scalar::Float)
                                    .map_err(|_| ValueError::InvalidFloat),
                    _ => Err(ValueError::UnknownTag(handle, suffix)),
                }
            }
            else {
                Err(ValueError::UnknownTag(handle, suffix))
            }
        }
        else {
            // Untagged plain: Follow rules of core schema.
            if value.starts_with("0x") {
                if let Ok(n) = i64::from_str_radix(&value[2..], 16) {
                    return Ok(Scalar::Integer(n));
                }
            }
            if value.starts_with("0o") {
                if let Ok(n) = i64::from_str_radix(&value[2..], 8) {
                    return Ok(Scalar::Integer(n));
                }
            }
            if value.starts_with('+') {
                if let Ok(n) = value[1..].parse::<i64>() {
                    return Ok(Scalar::Integer(n));
                }
            }
            if let Ok(n) = value.parse::<i64>() {
                return Ok(Scalar::Integer(n))
            }
            if let Ok(x) = value.parse::<f64>() {
                return Ok(Scalar::Float(x))
            }
            Ok(match value.as_ref() {
                "null" | "Null" | "NULL" | "~" | "" => Scalar::Null,
                "true" | "True" | "TRUE" => Scalar::Boolean(true),
                "false" | "False" | "FALSE"  => Scalar::Boolean(false),
                ".inf" | ".Inf" | ".INF" | "+.inf" | "+.Inf" | "+.INF"
                    => Scalar::Float(f64::INFINITY),
                "-.inf" | "-.Inf" | "-.INF"
                    => Scalar::Float(f64::NEG_INFINITY),
                ".nan" | "NaN" | ".NAN" => Scalar::Float(f64::NAN),
                _ => Scalar::String(value)
            })
        }
    }
}


//------------ ValueError ----------------------------------------------------

#[derive(Clone, Debug)]
pub enum ValueError {
    InvalidMappingKey(Type),
    InvalidBool,
    InvalidInt,
    InvalidFloat,
    Alias,
    UnknownTag(String, String),
}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ValueError::*;

        match *self {
            InvalidMappingKey(ref key_type) => {
                write!(f, "mapping key cannot be a {}", key_type)
            }
            InvalidBool => f.write_str("invalid boolean"),
            InvalidInt => f.write_str("invalid integer"),
            InvalidFloat => f.write_str("invalid float"),
            Alias => f.write_str("aliases are not allowed"),
            UnknownTag(ref handle, ref suffix) => {
                write!(f, "unknown tag !{}{}", handle, suffix)
            }
        }
    }
}



//------------ YamlError -----------------------------------------------------

#[derive(Clone, Debug)]
pub enum YamlError {
    BadValue(ValueError),
    TypeMismatch { expected: Type, received: Type },
    MissingKey(String),
}

impl YamlError {
    fn type_mismatch<R: Into<Type>>(expected: Type, received: R) -> Self {
        YamlError::TypeMismatch { expected, received: received.into() }
    }
}

impl fmt::Display for YamlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::YamlError::*;

        match *self {
            BadValue(ref err) => fmt::Display::fmt(err, f),
            TypeMismatch { ref expected, ref received } => {
                write!(f, "expected {}, got {}", expected, received)
            }
            MissingKey(ref key) => {
                write!(f, "missing mandatory key {}", key)
            }
        }
    }
}

impl From<ValueError> for YamlError {
    fn from(err: ValueError) -> YamlError {
        YamlError::BadValue(err)
    }
}


//------------ Type ----------------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Type {
    Sequence,
    Mapping,
    String,
    Null,
    Boolean,
    Integer,
    Float,
    Error,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Type::*;

        f.write_str(match *self {
            Sequence => "sequence",
            Mapping => "mapping",
            String => "string",
            Null => "null",
            Boolean => "boolean",
            Integer => "integer",
            Float => "float",
            Error => "error",
        })
    }
}

impl From<Value> for Type {
    fn from(value: Value) -> Type {
        value.into_value().into()
    }
}

impl From<PlainValue> for Type {
    fn from(value: PlainValue) -> Type {
        match value {
            PlainValue::Sequence(_) => Type::Sequence,
            PlainValue::Mapping(_) => Type::Mapping,
            PlainValue::Scalar(scalar) => scalar.into(),
            PlainValue::Error(_) => Type::Error,
        }
    }
}

impl From<Scalar> for Type {
    fn from(scalar: Scalar) -> Type {
        match scalar {
            Scalar::String(_) => Type::String,
            Scalar::Null => Type::Null,
            Scalar::Boolean(_) => Type::Boolean,
            Scalar::Integer(_) => Type::Integer,
            Scalar::Float(_) => Type::Float,
        }
    }
}


//------------ UnexpectedKey -------------------------------------------------

pub struct UnexpectedKey(String);

impl fmt::Display for UnexpectedKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unexpected key '{}'", self.0)
    }
}

