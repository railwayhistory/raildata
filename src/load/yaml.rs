
use std::{f64, fmt, ops};
use std::collections::HashMap;
use std::str::FromStr;
use yaml_rust::scanner::{Marker, ScanError, TokenType, TScalarStyle};
use yaml_rust::parser::{Event, MarkedEventReceiver, Parser};
use crate::types::{IntoMarked, Location, Marked};
use super::report::{Failed, Message, PathReporter, ResultExt};


//------------ Constructor ---------------------------------------------------

pub trait Constructor {
    fn construct(&mut self, doc: Value);
}

impl Constructor for Vec<Value> {
    fn construct(&mut self, doc: Value) {
        self.push(doc)
    }
}

impl<F: FnMut(Value)> Constructor for F {
    fn construct(&mut self, doc: Value) {
        self(doc)
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
        parser.load(self, true)?;
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
            match *self.nodes.last_mut().unwrap() {
                Value::Sequence(ref mut sequence) => {
                    sequence.push(value)
                }
                Value::Mapping(ref mut mapping) => {
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


//------------ Value ---------------------------------------------------------

#[derive(Clone, Debug)]
pub enum Value {
    Sequence(Sequence),
    Mapping(Mapping),
    Scalar(Scalar),
    Error(Marked<ValueError>),
}

impl Value {
    fn sequence(mark: Marker) -> Self {
        Value::Sequence(Sequence::new(mark.into()))
    }

    fn mapping(mark: Marker) -> Self {
        Value::Mapping(Mapping::new(mark.into()))
    }

    fn scalar(
        value: String,
        plain: bool,
        tag: Option<(String, String)>,
        mark: Marker
    ) -> Self {
        match Scalar::new(value, plain, tag, mark.into()) {
            Ok(scalar) => Value::Scalar(scalar),
            Err(err) => Value::Error(err)
        }
    }

    fn alias(mark: Marker) -> Self {
        Value::Error(Marked::new(ValueError::Alias, mark.into()))
    }

    pub fn location(&self) -> Location {
        match *self {
            Value::Sequence(ref inner) => inner.location,
            Value::Mapping(ref inner) => inner.location,
            Value::Scalar(ref inner) => inner.location(),
            Value::Error(ref inner) => inner.location(),
        }
    }

    pub fn report_error<M: Message>(
        self,
        message: M,
        report: &mut PathReporter
    ) -> Failed {
        let message = match self {
            Value::Sequence(inner) => Marked::new(message, inner.location),
            Value::Mapping(inner) => Marked::new(message, inner.location),
            Value::Scalar(inner) => inner.into_error(message),
            Value::Error(inner) => {
                report.error(inner);
                return Failed
            }
        };
        report.error(message);
        Failed
    }
}

impl Value {
    pub fn try_into_mapping(self) -> Result<Mapping, Self> {
        match self {
            Value::Mapping(res) => Ok(res),
            _ => Err(self)
        }
    }

    pub fn into_mapping(
        self,
        report: &mut PathReporter
    ) -> Result<Mapping, Failed> {
        self.try_into_mapping().map_err(|err| {
            let msg = TypeMismatch::new(Type::Mapping, &err);
            err.report_error(msg, report)
        })
    }

    pub fn try_into_sequence(self) -> Result<Sequence, Self> {
        match self {
            Value::Sequence(res) => Ok(res),
            _ => Err(self)
        }
    }

    pub fn into_sequence(
        self,
        report: &mut PathReporter
    ) -> Result<Sequence, Failed> {
        self.try_into_sequence().map_err(|err| {
            let msg = TypeMismatch::new(Type::Sequence, &err);
            err.report_error(msg, report)
        })
    }

    fn try_into_scalar(self) -> Result<Scalar, Self> {
        match self {
            Value::Scalar(res) => Ok(res),
            _ => Err(self)
        }
    }

    pub fn try_into_string(self) -> Result<Marked<String>, Self> {
        match self.try_into_scalar() {
            Ok(scalar) => match scalar {
                Scalar::String(res) => Ok(res),
                err => Err(Value::Scalar(err))
            }
            Err(err) => Err(err)
        }
    }

    pub fn into_string(
        self,
        report: &mut PathReporter
    ) -> Result<Marked<String>, Failed> {
        self.try_into_string().map_err(|err| {
            let msg = TypeMismatch::new(Type::String, &err);
            err.report_error(msg, report)
        })
    }

    pub fn try_into_null(self) -> Result<Marked<()>, Self> {
        match self.try_into_scalar() {
            Ok(scalar) => match scalar {
                Scalar::Null(res) => Ok(res),
                err => Err(Value::Scalar(err))
            }
            Err(err) => Err(err)
        }
    }


    pub fn into_null(
        self,
        report: &mut PathReporter
    ) -> Result<Marked<()>, Failed> {
        self.try_into_null().map_err(|err| {
            let msg = TypeMismatch::new(Type::Null, &err);
            err.report_error(msg, report)
        })
    }

    pub fn is_null(&self) -> bool {
        if let Value::Scalar(Scalar::Null(_)) = *self { true }
        else { false }
    }

    pub fn try_into_boolean(self) -> Result<Marked<bool>, Self> {
        match self.try_into_scalar() {
            Ok(scalar) => match scalar {
                Scalar::Boolean(res) => Ok(res),
                err => Err(Value::Scalar(err))
            }
            Err(err) => Err(err)
        }
    }

    pub fn into_boolean(
        self,
        report: &mut PathReporter
    ) -> Result<Marked<bool>, Failed> {
        self.try_into_boolean().map_err(|err| {
            let msg = TypeMismatch::new(Type::Boolean, &err);
            err.report_error(msg, report)
        })
    }

    pub fn try_into_integer(self) -> Result<Marked<i64>, Self> {
        match self.try_into_scalar() {
            Ok(scalar) => match scalar {
                Scalar::Integer(res) => Ok(res),
                err => Err(Value::Scalar(err))
            }
            Err(err) => Err(err)
        }
    }

    pub fn into_integer(
        self, report: &mut PathReporter
    ) -> Result<Marked<i64>, Failed> {
        self.try_into_integer().map_err(|err| {
            let msg = TypeMismatch::new(Type::Integer, &err);
            err.report_error(msg, report)
        })
    }

    pub fn try_into_float(self) -> Result<Marked<f64>, Self> {
        match self.try_into_scalar() {
            Ok(scalar) => match scalar {
                Scalar::Float(res) => Ok(res),
                err => Err(Value::Scalar(err))
            }
            Err(err) => Err(err)
        }
    }

    pub fn into_float(
        self,
        report: &mut PathReporter
    ) -> Result<Marked<f64>, Failed> {
        self.try_into_float().map_err(|err| {
            let msg = TypeMismatch::new(Type::Float, &err);
            err.report_error(msg, report)
        })
    }
}


//------------ Mapping -------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct Mapping {
    items: HashMap<Marked<String>, Value>,
    errors: Vec<Marked<ValueError>>,
    location: Location,
}

impl Mapping {
    fn new(location: Location) -> Self {
        Mapping {
            items: HashMap::new(),
            errors: Vec::new(),
            location
        }
    }

    fn insert(&mut self, key: Value, value: Value) {
        match key.try_into_string() {
            Ok(key) => { self.items.insert(key, value); },
            Err(err) => {
                let location = err.location();
                let err = ValueError::InvalidMappingKey(err.into());
                self.errors.push(err.marked(location))
            }
        }
    }
}

impl Mapping {
    pub fn location(&self) -> Location {
        self.location
    }

    pub fn take<C, T: FromYaml<C>>(
        &mut self,
        key: &str,
        context: &C,
        report: &mut PathReporter
    ) -> Result<T, Failed> {
        if let Some(value) = self.items.remove(key) {
            T::from_yaml(value, context, report)
        }
        else {
            report.error(MissingKey(key.into()).marked(self.location));
            Err(Failed)
        }
    }

    pub fn take_default<C, T: FromYaml<C> + Default>(
        &mut self,
        key: &str,
        context: &C,
        report: &mut PathReporter
    ) -> Result<T, Failed> {
        if let Some(value) = self.items.remove(key) {
            T::from_yaml(value, context, report)
        }
        else {
            Ok(T::default())
        }
    }

    pub fn take_opt<C, T: FromYaml<C>>(
        &mut self,
        key: &str,
        context: &C,
        report: &mut PathReporter
    ) -> Result<Option<T>, Failed> {
        if let Some(value) = self.items.remove(key) {
            T::from_yaml(value, context, report).map(Some)
        }
        else {
            Ok(None)
        }
    }

    pub fn exhausted(
        mut self, report: &mut PathReporter
    ) -> Result<(), Failed> {
        let mut failed = self.check(report).is_err();
        if !self.items.is_empty() {
            for (key, _) in self.items {
                report.error(key.map(|key| UnexpectedKey(key)));
            }
            failed = true;
        }
        if failed { Err(Failed) }
        else { Ok(()) }
    }

    pub fn check(&mut self, report: &mut PathReporter) -> Result<(), Failed> {
        if self.errors.is_empty() {
            Ok(())
        }
        else {
            for error in self.errors.drain(..) {
                report.error(error)
            }
            Err(Failed)
        }
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


//------------ Sequence ------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct Sequence {
    items: Vec<Value>,
    location: Location,
}

impl Sequence {
    fn new(location: Location) -> Self {
        Sequence {
            items: Vec::new(),
            location
        }
    }

    pub fn location(&self) -> Location {
        self.location
    }
}

impl ops::Deref for Sequence {
    type Target = Vec<Value>;

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl ops::DerefMut for Sequence {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.items
    }
}

impl IntoIterator for Sequence {
    type Item = Value;
    type IntoIter = ::std::vec::IntoIter<Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}


//------------ Scalar --------------------------------------------------------

#[derive(Clone, Debug)]
pub enum Scalar {
    String(Marked<String>),
    Null(Marked<()>),
    Boolean(Marked<bool>),
    Integer(Marked<i64>),
    Float(Marked<f64>),
}

impl Scalar {
    pub fn new(
        value: String,
        plain: bool,
        tag: Option<(String, String)>,
        location: Location
    ) -> Result<Self, Marked<ValueError>> {
        if !plain {
            Ok(Scalar::String(value.marked(location)))
        }
        else if let Some((handle, suffix)) = tag {
            if handle.as_str() == "!!" {
                match suffix.as_str() {
                    "str" => Ok(Scalar::String(value.marked(location))),
                    "null" => Ok(Scalar::Null(().marked(location))),
                    "bool" => {
                        bool::from_str(&value).map(|value| {
                            Scalar::Boolean(value.marked(location))
                        }).map_err(|_| ValueError::InvalidBool.marked(location))
                    }
                    "int" => {
                        i64::from_str(&value).map(|value| {
                            Scalar::Integer(value.marked(location))
                        }).map_err(|_| ValueError::InvalidInt.marked(location))
                    }
                    "float" => {
                        f64::from_str(&value).map(|value| {
                            Scalar::Float(value.marked(location))
                        }).map_err(|_| {
                            ValueError::InvalidFloat.marked(location)
                        })
                    }
                    _ => {
                        Err(
                            ValueError::UnknownTag(handle, suffix)
                                .marked(location)
                        )
                    }
                }
            }
            else {
                Err(ValueError::UnknownTag(handle, suffix).marked(location))
            }
        }
        else {
            // Untagged plain: Follow rules of core schema.
            if value.starts_with("0x") {
                if let Ok(n) = i64::from_str_radix(&value[2..], 16) {
                    return Ok(Scalar::Integer(n.marked(location)));
                }
            }
            if value.starts_with("0o") {
                if let Ok(n) = i64::from_str_radix(&value[2..], 8) {
                    return Ok(Scalar::Integer(n.marked(location)));
                }
            }
            if value.starts_with('+') {
                if let Ok(n) = value[1..].parse::<i64>() {
                    return Ok(Scalar::Integer(n.marked(location)));
                }
            }
            if let Ok(n) = value.parse::<i64>() {
                return Ok(Scalar::Integer(n.marked(location)))
            }
            if let Ok(x) = value.parse::<f64>() {
                return Ok(Scalar::Float(x.marked(location)))
            }
            Ok(match value.as_ref() {
                "null" | "Null" | "NULL" | "~" | "" => {
                    Scalar::Null(().marked(location))
                }
                "true" | "True" | "TRUE" => {
                    Scalar::Boolean(true.marked(location))
                }
                "false" | "False" | "FALSE"  => {
                    Scalar::Boolean(false.marked(location))
                }
                ".inf" | ".Inf" | ".INF" | "+.inf" | "+.Inf" | "+.INF" => {
                    Scalar::Float(f64::INFINITY.marked(location))
                }
                "-.inf" | "-.Inf" | "-.INF" => {
                    Scalar::Float(f64::NEG_INFINITY.marked(location))
                }
                ".nan" | "NaN" | ".NAN" => {
                    Scalar::Float(f64::NAN.marked(location))
                }
                _ => Scalar::String(value.marked(location))
            })
        }
    }

    fn location(&self) -> Location {
        match *self {
            Scalar::String(ref inner) => inner.location(),
            Scalar::Null(ref inner) => inner.location(),
            Scalar::Boolean(ref inner) => inner.location(),
            Scalar::Integer(ref inner) => inner.location(),
            Scalar::Float(ref inner) => inner.location(),
        }
    }

    fn into_error<M: Message>(
        self,
        message: M
    ) -> Marked<M> {
        message.marked(self.location())
    }
}


//------------ FromYaml ------------------------------------------------------

/// A type that can be constructed from a Yaml value.
pub trait FromYaml<C>: Sized {
    fn from_yaml(
        value: Value,
        context: &C,
        report: &mut PathReporter
    ) -> Result<Self, Failed>;
}

impl<C, T: FromYaml<C>> FromYaml<C> for Option<T> {
    fn from_yaml(
        value: Value,
        context: &C,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        if value.is_null() {
            Ok(None)
        }
        else {
            T::from_yaml(value, context, report).map(Some)
        }
    }
}

impl<C, T: FromYaml<C>> FromYaml<C> for Marked<Option<T>> {
    fn from_yaml(
        value: Value,
        context: &C,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        if value.is_null() {
            Ok(Marked::new(None, value.location()))
        }
        else {
            let location = value.location();
            T::from_yaml(value, context, report).map(|value| {
                Marked::new(Some(value), location)
            })
        }
    }
}

impl<C> FromYaml<C> for Marked<bool> {
    fn from_yaml(
        value: Value,
        _context: &C,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        value.into_boolean(report)
    }
}

impl<C> FromYaml<C> for bool {
    fn from_yaml(
        value: Value,
        _context: &C,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        value.into_boolean(report).map(Marked::into_value)
    }
}

impl<C> FromYaml<C> for Marked<String> {
    fn from_yaml(
        value: Value,
        _context: &C,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        value.into_string(report)
    }
}

impl<C> FromYaml<C> for String {
    fn from_yaml(
        value: Value,
        _context: &C,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        value.into_string(report).map(Marked::into_value)
    }
}

impl<C> FromYaml<C> for Marked<u8> {
    fn from_yaml(
        value: Value,
        _: &C,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        value.into_integer(report)?.try_map(|int| {
            if int < 0 || int > ::std::u8::MAX as i64 {
                Err(RangeError::new(0, ::std::u8::MAX as i64, int))
            }
            else {
                Ok(int as u8)
            }
         }).or_error(report)
    }
}

impl<C> FromYaml<C> for u8 {
    fn from_yaml(
        value: Value,
        context: &C,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        Marked::from_yaml(value, context, report).map(Marked::into_value)
    }
}

impl<C> FromYaml<C> for Marked<f64> {
    fn from_yaml(
        value: Value,
        _: &C,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        value.into_float(report)
    }
}

impl<C> FromYaml<C> for f64 {
    fn from_yaml(
        value: Value,
        _: &C,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        value.into_float(report).map(Marked::into_value)
    }
}

impl<C, T: FromYaml<C>> FromYaml<C> for Vec<T> {
    fn from_yaml(
        value: Value,
        context: &C,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let location = value.location();
        match value.try_into_sequence() {
            Ok(seq) =>{
                if seq.is_empty() {
                    report.error(EmptySequence.marked(location));
                    Err(Failed)
                }
                else {
                    let mut res = Ok(Vec::with_capacity(seq.len()));
                    for item in seq {
                        match  T::from_yaml(item, context, report) {
                            Ok(item) => {
                                if let Ok(ref mut vec) = res {
                                    vec.push(item)
                                }
                            }
                            Err(Failed) => res = Err(Failed),
                        }
                    }
                    res
                }
            }
            Err(value) => {
                T::from_yaml(value, context, report).map(|value| {
                    vec![value]
                })
            }
        }
    }
}


//============ Errors ========================================================

//------------ ValueError ----------------------------------------------------

/// A error happened when creating a value from its YAML representation.
#[derive(Clone, Debug, Display)]
pub enum ValueError {
    #[display(fmt="mapping key cannot be a {}", _0)]
    InvalidMappingKey(Type),

    #[display(fmt="invalid boolean")]
    InvalidBool,

    #[display(fmt="invalid integer")]
    InvalidInt,

    #[display(fmt="invalid float")]
    InvalidFloat,

    #[display(fmt="aliases are not allowed")]
    Alias,

    #[display(fmt="unknown tag !{}{}", _0, _1)]
    UnknownTag(String, String),
}


//------------ TypeMismatch --------------------------------------------------

#[derive(Clone, Debug, Display)]
#[display(fmt="expected {}, got {}", expected, received)]
pub struct TypeMismatch {
    expected: Type,
    received: Type
}

impl TypeMismatch {
    pub fn new<T: Into<Type>>(expected: Type, received: T) -> Self {
        TypeMismatch { expected, received: received.into() }
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
        Type::from(&value)
    }
}

impl<'a> From<&'a Value> for Type {
    fn from(value: &'a Value) -> Type {
        match *value {
            Value::Sequence(_) => Type::Sequence,
            Value::Mapping(_) => Type::Mapping,
            Value::Scalar(ref scalar) => scalar.into(),
            Value::Error(_) => Type::Error,
        }
    }
}

impl<'a> From<&'a Scalar> for Type {
    fn from(scalar: &'a Scalar) -> Type {
        match *scalar {
            Scalar::String(_) => Type::String,
            Scalar::Null(_) => Type::Null,
            Scalar::Boolean(_) => Type::Boolean,
            Scalar::Integer(_) => Type::Integer,
            Scalar::Float(_) => Type::Float,
        }
    }
}


//------------ MissingKey ----------------------------------------------------

#[derive(Clone, Debug, Display)]
#[display(fmt="missing key {}", _0)]
pub struct MissingKey(String);


//------------ UnexpectedKey -------------------------------------------------

#[derive(Clone, Debug, Display)]
#[display(fmt="unexpected key {}", _0)]
pub struct UnexpectedKey(String);


//------------ RangeError ----------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct RangeError {
    low: i64,
    hi: i64,
    is: i64
}

impl RangeError {
    pub fn new(low: i64, hi: i64, is: i64) -> Self {
        RangeError { low, hi, is }
    }
}

impl fmt::Display for RangeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "value {} is outside of range {} to {}",
               self.is, self.low, self.hi)
    }
}


//------------ EmptySequence -------------------------------------------------

#[derive(Clone, Copy, Debug, Display)]
#[display(fmt="empty sequence")]
pub struct EmptySequence;

