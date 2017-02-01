#[macro_use]
extern crate log;
extern crate fibers;
extern crate futures;
extern crate httparse;
extern crate handy_async;

//pub mod server;
//pub mod server2;
//pub mod client;
pub mod client2;
// pub mod headers;

//pub mod route;
//pub mod request;
//pub mod response;
pub mod error;
//pub mod connection;

pub mod connection2;
pub mod request2;
pub mod response2;
pub use error::Error;

pub use header::{Header, Headers};
pub use method::Method;
pub use status::{Status, RawStatus};
pub use version::Version;

pub mod builtin;

mod header;
mod method;
mod status;
mod version;

pub mod iterators {
    pub use header::Iter as HeaderIter;
}

pub type Result<T> = ::std::result::Result<T, error::Error>;

// pub trait Header: fmt::Display {
//     fn parse(headers: &Headers) -> io::Result<Option<Self>> where Self: Sized;
//     fn write(&self, buf: &mut Vec<u8>);
// }

// #[derive(Debug, Clone)]
// pub struct Headers<'a> {
//     headers: &'a [httparse::Header<'a>],
// }
// impl<'a> Headers<'a> {
//     pub fn get_bytes(&self, name: &str) -> Option<&[u8]> {
//         use std::ascii::AsciiExt;
//         self.headers.iter().find(|h| h.name.eq_ignore_ascii_case(name)).map(|h| h.value)
//     }
//     pub fn get<H: Header>(&self) -> io::Result<Option<H>> {
//         H::parse(self)
//     }
// }


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
