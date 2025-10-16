//! Helpers for dealing with things that are `io::Read`.

use std::{io, str};


pub struct Utf8Chars<R: io::Read> {
    rd: R,
    buf: [u8; 4],
    bufpos: usize,
    err: Option<Utf8Error>,
}

impl<R: io::Read> Utf8Chars<R> {
    pub fn new(rd: R) -> Self {
        Utf8Chars { rd, buf: [0; 4], bufpos: 0, err: None }
    }

    pub fn done(self) -> Result<R, Utf8Error> {
        if let Some(err) = self.err {
            Err(err)
        }
        else {
            Ok(self.rd)
        }
    }

    fn try_next(&mut self) -> Result<Option<char>, Utf8Error> {
        loop {
            match self.rd.read(&mut self.buf[self.bufpos..self.bufpos + 1]) {
                Ok(0) if self.bufpos == 0 => {
                    return Ok(None)
                }
                Ok(0) => {
                    let err = io::Error::new(io::ErrorKind::UnexpectedEof,
                                             "unexpected EOF");
                    return Err(Utf8Error::Io(err))
                }
                Ok(1) => { self.bufpos += 1; }
                Err(err) => return Err(Utf8Error::Io(err)),
                _ => unreachable!()
            }
            let width = utf8_char_width(self.buf[0]);
            if width == 0 { return Err(Utf8Error::Encoding) }
            if width == self.bufpos {
                let s = str::from_utf8(&self.buf[..self.bufpos])
                            .map_err(|_| Utf8Error::Encoding)?;
                self.bufpos = 0;
                return Ok(Some(s.chars().next().unwrap()))
            }
        }
    }
}

impl<R: io::Read> Iterator for Utf8Chars<R> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        if self.err.is_some() {
            None
        }
        else {
            match self.try_next() {
                Ok(some) => some,
                Err(err) => {
                    self.err = Some(err);
                    None
                }
            }
        }
    }
}


pub enum Utf8Error {
    Encoding,
    Io(io::Error),
}


//------------ Helper Functions ---------------------------------------------
//
// The following is currently experimental in core::str. We will switch to
// that once it stabilizes.

// https://tools.ietf.org/html/rfc3629
static UTF8_CHAR_WIDTH: [u8; 256] = [
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x1F
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x3F
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x5F
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x7F
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, // 0x9F
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, // 0xBF
0,0,2,2,2,2,2,2,2,2,2,2,2,2,2,2,
2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2, // 0xDF
3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3, // 0xEF
4,4,4,4,4,0,0,0,0,0,0,0,0,0,0,0, // 0xFF
];

/// Given a first byte, determines how many bytes are in this UTF-8 character.
#[inline]
fn utf8_char_width(b: u8) -> usize {
    UTF8_CHAR_WIDTH[b as usize] as usize
}


