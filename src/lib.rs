#[macro_use]
extern crate log;
extern crate fibers;
extern crate futures;
extern crate httparse;
extern crate handy_async;

use std::fmt;
use std::io::Result;

pub mod server;
pub mod client;
pub mod headers;

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

#[allow(non_camel_case_types)]
pub enum Version {
    Http1_0,
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

pub trait Header {
    fn parse(headers: &Headers) -> Result<Option<Self>> where Self: Sized;
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
    pub fn get<H: Header>(&self) -> Result<Option<H>> {
        H::parse(self)
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
