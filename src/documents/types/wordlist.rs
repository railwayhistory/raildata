use std::fmt;
use std::str::SplitWhitespace;
use ::load::construct::{Constructable, Context, Failed};
use ::load::yaml::{PlainValue, Scalar, Value};
use super::marked::Location;
use super::simple::Text;


pub struct WordList<'a> {
    iter: Option<SplitWhitespace<'a>>,
    location: Location,
}

impl<'a> WordList<'a> {
    pub fn new(s: &'a Text) -> WordList<'a> {
        WordList {
            iter: Some(s.split_whitespace()),
            location: s.location()
        }
    }

    pub fn take<C, Ctx>(&mut self, context: &mut Ctx)
                        -> Result<C, Failed>
            where C: Constructable,
                  Ctx: Context {
        match self.take_opt(context) {
            Ok(Some(res)) => Ok(res),
            Ok(None) => {
                context.push_error((MissingWord, self.location));
                self.iter = None;
                Err(Failed)
            }
            Err(_) => Err(Failed)
        }
    }

    pub fn take_opt<C, Ctx>(&mut self, context: &mut Ctx)
                            -> Result<Option<C>, Failed>
                    where C: Constructable,
                          Ctx: Context {
        if let Some(ref mut iter) = self.iter {
            match iter.next() {
                Some(word) => {
                    match Scalar::new(word.into(), true, None) {
                        Ok(value) => {
                            let value = Value::new(PlainValue::Scalar(value),
                                                   self.location);
                            if let Ok(res) = C::construct(value, context) {
                                return Ok(Some(res))
                            }
                            // fall through on error
                        }
                        Err(err) => {
                            context.push_error((err, self.location));
                            // fall through
                        }
                    }
                }
                None => return Ok(None)
            }
        }
        self.iter = None;
        Err(Failed)
    }

    pub fn exhausted<C: Context>(self, context: &mut C) -> Result<(), Failed> {
        if let Some(mut words) = self.iter {
            if let Some(_) = words.next() {
                context.push_error((TooManyWords, self.location));
                Err(Failed)
            }
            else {
                Ok(())
            }
        }
        else {
            // If the iterator is gone, something bad had happened before.
            Err(Failed)
        }
    }
}


#[derive(Clone, Copy, Debug)]
pub struct MissingWord;

impl fmt::Display for MissingWord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("missing words in word list")
    }
}


#[derive(Clone, Copy, Debug)]
pub struct TooManyWords;

impl fmt::Display for TooManyWords {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("extra words in word list")
    }
}

