#[macro_use]
extern crate log;
extern crate fibers;
extern crate futures;
extern crate httparse;

use std::fmt;

pub mod server;

// See: https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Method {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Connect,
    Options,
    Trace,
    Patch,
    Other(String),
}
impl Method {
    pub fn from_str(method: &str) -> Self {
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
            other => Method::Other(other.to_string()),
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
impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
