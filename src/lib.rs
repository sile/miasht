#[macro_use]
extern crate log;
extern crate fibers;
extern crate futures;
extern crate httparse;
extern crate handy_async;

use std::fmt;
use std::io;

pub mod server;
pub mod server2;
pub mod client;
pub mod client2;
pub mod headers;

pub mod route;
pub mod method;
pub mod request;
pub mod response;
pub mod error;
pub mod connection;
pub mod status;

pub mod connection2;
pub mod request2;
pub mod response2;
pub use error::Error;

pub use version::Version;
pub use method::Method;

mod version;
mod method;

pub type Result<T> = ::std::result::Result<T, error::Error>;

// See: https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Method<'a> {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Connect,
    Options,
    Trace,
    Patch,
    Other(&'a str),
}
impl<'a> Method<'a> {
    pub fn from_str(method: &'a str) -> Self {
        match method {
            "GET" => Method::Get,
            "HEAD" => Method::Head,
            "POST" => Method::Post,
            "PUT" => Method::Put,
            "DELETE" => Method::Delete,
            "CONNECT" => Method::Connect,
            "OPTIONS" => Method::Options,
            "TRACE" => Method::Trace,
            "PATCH" => Method::Patch,
            other => Method::Other(other),
        }
    }
    pub fn as_str(&self) -> &str {
        match *self {
            Method::Get => "GET",
            Method::Head => "HEAD",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
            Method::Connect => "CONNECT",
            Method::Options => "OPTIONS",
            Method::Trace => "TRACE",
            Method::Patch => "PATCH",
            Method::Other(ref s) => s,
        }
    }
}
impl<'a> fmt::Display for Method<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub trait Header: fmt::Display {
    fn parse(headers: &Headers) -> io::Result<Option<Self>> where Self: Sized;
    fn write(&self, buf: &mut Vec<u8>);
}

#[derive(Debug, Clone)]
pub struct Headers<'a> {
    headers: &'a [httparse::Header<'a>],
}
impl<'a> Headers<'a> {
    pub fn get_bytes(&self, name: &str) -> Option<&[u8]> {
        use std::ascii::AsciiExt;
        self.headers.iter().find(|h| h.name.eq_ignore_ascii_case(name)).map(|h| h.value)
    }
    pub fn get<H: Header>(&self) -> io::Result<Option<H>> {
        H::parse(self)
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
