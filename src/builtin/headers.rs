use std::fmt;
use std::str;
use std::u64;

use Header;
use error::HeaderParseError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContentLength(pub u64);
impl ContentLength {
    pub fn len(&self) -> u64 {
        self.0
    }
}
impl Header for ContentLength {
    fn name() -> &'static str {
        "Content-Length"
    }
    fn parse(value: &[u8]) -> Result<Self, HeaderParseError> {
        decimal_bytes_to_u64(value).map(ContentLength)
    }
}
impl fmt::Display for ContentLength {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Content-Length: {}", self.0)
    }
}

fn decimal_bytes_to_u64(bytes: &[u8]) -> Result<u64, HeaderParseError> {
    let value = str::from_utf8(bytes)?;
    let value = u64::from_str_radix(value, 10)?;
    Ok(value)
}
