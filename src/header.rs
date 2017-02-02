use std::fmt;
use std::str;
use std::error;
use std::slice;
use std::ascii::AsciiExt;
use httparse;

#[derive(Debug)]
pub struct Headers<'a>(&'a [httparse::Header<'a>]);
impl<'a> Headers<'a> {
    pub fn new(headers: &'a [httparse::Header<'a>]) -> Self {
        Headers(headers)
    }
    pub fn parse<H: Header>(&self) -> Result<Option<H>, ParseValueError<H::Error>> {
        if let Some(v) = self.get(H::name()) {
            H::parse_value_bytes(v).map(Some)
        } else {
            Ok(None)
        }
    }
    pub fn get(&self, name: &str) -> Option<&[u8]> {
        self.iter().find(|h| h.0.eq_ignore_ascii_case(name)).map(|h| h.1)
    }
    pub fn iter(&self) -> Iter {
        Iter(self.0.iter())
    }
}

#[derive(Debug)]
pub struct Iter<'a>(slice::Iter<'a, httparse::Header<'a>>);
impl<'a> Iterator for Iter<'a> {
    type Item = (&'a str, &'a [u8]);
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|h| (h.name, h.value))
    }
}

pub trait Header: Sized + fmt::Display {
    type Error;
    fn name() -> &'static str;
    fn parse_value_bytes(value: &[u8]) -> Result<Self, ParseValueError<Self::Error>> {
        let s = str::from_utf8(value).map_err(|e| {
                ParseValueError::InvalidUtf8 {
                    name: Self::name(),
                    reason: e,
                }
            })?;
        Self::parse_value_str(s).map_err(|e| {
            ParseValueError::Malformed {
                name: Self::name(),
                reason: e,
            }
        })
    }
    fn parse_value_str(value: &str) -> Result<Self, Self::Error>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseValueError<E> {
    InvalidUtf8 {
        name: &'static str,
        reason: str::Utf8Error,
    },
    Malformed { name: &'static str, reason: E },
}
impl<E> fmt::Display for ParseValueError<E>
    where E: fmt::Display
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseValueError::InvalidUtf8 { name, ref reason } => {
                write!(f,
                       "Invalid UTF-8 in HTTP header {:?}: reason={}",
                       name,
                       reason)
            }
            ParseValueError::Malformed { name, ref reason } => {
                write!(f,
                       "Malformed HTTP header value: name={:?}, reason={}",
                       name,
                       reason)
            }
        }
    }
}
impl<E> error::Error for ParseValueError<E>
    where E: error::Error
{
    fn description(&self) -> &str {
        match *self {
            ParseValueError::InvalidUtf8 { .. } => "Invalid UTF-8 in HTTP header value",
            ParseValueError::Malformed { .. } => "Malformed HTTP header value",
        }
    }
    fn cause(&self) -> Option<&error::Error> {
        match *self {
            ParseValueError::InvalidUtf8 { ref reason, .. } => Some(reason),
            ParseValueError::Malformed { ref reason, .. } => Some(reason),
        }
    }
}
