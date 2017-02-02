use std::fmt;
use std::str;
use std::error;
use std::slice;
use std::ascii::AsciiExt;
use httparse;

pub type ParseError = Box<error::Error + Send + Sync>;

#[derive(Debug)]
pub struct Headers<'a>(&'a [httparse::Header<'a>]);
impl<'a> Headers<'a> {
    pub fn new(headers: &'a [httparse::Header<'a>]) -> Self {
        Headers(headers)
    }
    pub fn parse<H: Header>(&self) -> Result<Option<H>, ParseError> {
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
    fn name() -> &'static str;
    fn parse_value_bytes(value: &[u8]) -> Result<Self, ParseError> {
        Self::parse_value_str(str::from_utf8(value)?)
    }
    fn parse_value_str(value: &str) -> Result<Self, ParseError>;
}
