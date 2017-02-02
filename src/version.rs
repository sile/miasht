use std::fmt;

use Error;

/// HTTP version.
///
/// # Examples
///
/// ```
/// use miasht::Version;
///
/// assert_eq!(Version::Http1_0.to_string(), "HTTP/1.0");
/// assert_eq!(Version::Http1_1.to_string(), "HTTP/1.1");
/// ```
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum Version {
    /// HTTP/1.0.
    Http1_0,

    /// HTTP/1.1.
    Http1_1,
}
impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Version::Http1_0 => write!(f, "HTTP/1.0"),
            Version::Http1_1 => write!(f, "HTTP/1.1"),
        }
    }
}
impl Default for Version {
    fn default() -> Self {
        Version::Http1_1
    }
}

pub fn try_from_u8(value: u8) -> Result<Version, Error> {
    match value {
        0 => Ok(Version::Http1_0),
        1 => Ok(Version::Http1_1),
        _ => Err(Error::UnknownVersion(value)),
    }
}
