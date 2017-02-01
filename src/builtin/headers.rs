use std::fmt;
use std::u64;

use Header;
use error::HeaderParseError;

/// `Content-Length` header.
///
/// # Examples
///
/// ```
/// use miasht::Header;
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
impl Header for ContentLength {
    fn name() -> &'static str {
        "Content-Length"
    }
    fn parse_value_str(value: &str) -> Result<Self, HeaderParseError> {
        Ok(ContentLength(u64::from_str_radix(value, 10)?))
    }
}
impl fmt::Display for ContentLength {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Content-Length: {}", self.0)
    }
}
