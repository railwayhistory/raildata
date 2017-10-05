
use ::documents::document::Document;
use ::documents::types::Key;
use ::documents::types::list::ListError;
use ::store::{Link, Variant};
use super::error::Error;
use super::yaml::Value;


pub trait Context {
    fn get_link<T>(&mut self, key: &Key) -> Link<T>
                where T: Variant<Item=Document>;
    fn push_error<E: Into<Error>>(&mut self, error: E);
}


pub trait Constructable: Sized {
    fn construct<C: Context>(value: Value, context: &mut C)
                             -> Result<Self, Failed>;
}


#[derive(Clone, Copy, Debug)]
pub struct Failed;


impl<T: Constructable> Constructable for Option<T> {
    fn construct<C: Context>(value: Value, context: &mut C)
                             -> Result<Self, Failed> {
        if value.is_null() {
            Ok(None)
        }
        else {
            T::construct(value, context).map(Some)
        }
    }
}

impl<T: Constructable> Constructable for Vec<T> {
    fn construct<C: Context>(value: Value, context: &mut C)
                             -> Result<Self, Failed> {
        let location = value.location();
        Ok(match value.try_into_sequence() {
            Ok(seq) => {
                if seq.is_empty() {
                    context.push_error((ListError::Empty, location));
                    return Err(Failed)
                }
                else {
                    let mut res = Vec::with_capacity(seq.len());
                    let mut err = false;
                    for item in seq.into_value() {
                        if let Ok(item) = T::construct(item, context) {
                            res.push(item)
                        }
                        else {
                            err = true
                        }
                    }
                    if err { 
                        return Err(Failed)
                    }
                    res
                }
            }
            Err(value) => {
                let value = T::construct(value, context)?;
                vec![value]
            }
        })
    }
}

impl Constructable for f64 {
    fn construct<C: Context>(value: Value, context: &mut C)
                             -> Result<Self, Failed> {
        value.into_float(context).map(|v| v.into_value())
    }
}

