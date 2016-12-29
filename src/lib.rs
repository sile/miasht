#[macro_use]
extern crate log;
extern crate fibers;
extern crate futures;
extern crate httparse;

use std::fmt;
use std::io::Result;

pub mod server;

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

pub trait Header<'a>: Sized + fmt::Display {
    fn parse(headers: &'a [httparse::Header]) -> Result<Option<Self>>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
