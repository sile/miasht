use std::fmt;
use std::u64;
use std::io::{self, Write};
use std::ascii::AsciiExt;

use header::Header;
use super::NoError;

macro_rules! impl_display {
    ($header:ident) => {
        impl fmt::Display for $header {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                let mut value = Vec::new();
                self.write_value(&mut value).map_err(|_| fmt::Error)?;
                let value = String::from_utf8(value).map_err(|_| fmt::Error)?;
                write!(f, "{}: {}", $header::name(), value)
            }
        }
    };
    ($header:ident < $p:tt > ) => {
        impl < $p > fmt::Display for $header < $p > {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                let mut value = Vec::new();
                self.write_value(&mut value).map_err(|_| fmt::Error)?;
                let value = String::from_utf8(value).map_err(|_| fmt::Error)?;
                write!(f, "{}: {}", $header::name(), value)
            }
        }
    };
}

/// `Content-Length` header.
///
/// # Examples
///
/// ```
/// use miasht::header::Header;
/// use miasht::builtin::headers::ContentLength;
///
/// assert_eq!(ContentLength(10).to_string(), "Content-Length: 10");
/// assert_eq!(ContentLength::parse_value_str("10").ok(), Some(ContentLength(10)));
/// assert_eq!(ContentLength::parse_value_str("-10").ok(), None);
/// ```
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct ContentLength(pub u64);
impl ContentLength {
    pub fn len(&self) -> u64 {
        self.0
    }
}
impl<'a> Header<'a> for ContentLength {
    type Error = ::std::num::ParseIntError;
    fn name() -> &'static str {
        "Content-Length"
    }
    fn parse_value_str(value: &'a str) -> Result<Self, Self::Error> {
        Ok(ContentLength(u64::from_str_radix(value, 10)?))
    }
    fn write_value<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write!(writer, "{}", self.0)
    }
}
impl_display!(ContentLength);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransferEncoding<'a> {
    Chunked,
    Other(&'a str),
}
impl<'a> TransferEncoding<'a> {
    pub fn chunked_or_else<F, E>(&self, f: F) -> Result<(), E>
        where F: FnOnce(&'a str) -> E
    {
        if let TransferEncoding::Other(token) = *self {
            Err(f(token))
        } else {
            Ok(())
        }
    }
}
impl<'a> Header<'a> for TransferEncoding<'a> {
    type Error = NoError;
    fn name() -> &'static str {
        "Transfer-Encoding"
    }
    fn parse_value_str(value: &'a str) -> Result<Self, Self::Error> {
        if value.eq_ignore_ascii_case("chunked") {
            Ok(TransferEncoding::Chunked)
        } else {
            Ok(TransferEncoding::Other(value))
        }
    }
    fn write_value<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        match *self {
            TransferEncoding::Chunked => write!(writer, "chunked"),
            TransferEncoding::Other(s) => write!(writer, "{}", s),
        }
    }
}
impl_display!(TransferEncoding<'a>);
