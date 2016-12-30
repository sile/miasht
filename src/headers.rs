use std::io::{Error, ErrorKind, Result};
use std::fmt;
use std::str;
use std::u64;
use std::error;

use {Header, Headers};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContentLength(u64);
impl<'a> Header<'a> for ContentLength {
    fn parse(headers: &'a Headers) -> Option<Result<Self>> {
        headers.get_bytes("Content-Length")
            .map(|bytes| decimal_bytes_to_u64(bytes).map(ContentLength))
    }
}
impl fmt::Display for ContentLength {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Content-Length: {}", self.0)
    }
}

fn decimal_bytes_to_u64(bytes: &[u8]) -> Result<u64> {
    str::from_utf8(bytes)
        .map_err(to_invalid_data_error)
        .and_then(|s| u64::from_str_radix(s, 10).map_err(to_invalid_data_error))
}

fn to_invalid_data_error<E>(error: E) -> Error
    where E: error::Error + Send + Sync + 'static
{
    Error::new(ErrorKind::InvalidData, Box::new(error))
}
