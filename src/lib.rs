#[macro_use]
extern crate log;
extern crate fibers;
extern crate futures;
extern crate httparse;
extern crate handy_async;

//pub mod server;
//pub mod server2;
//pub mod client;
//pub mod client2;
// pub mod headers;

//pub mod route;
//pub mod request;
//pub mod response;
//pub mod connection;

mod connection2;
pub mod request2;
pub mod response2;

pub use client::Client;
pub use error::Error;
pub use method::Method;
pub use status::{Status, RawStatus};
pub use version::Version;

pub mod builtin;
pub mod header;
pub mod client;
mod error;
mod method;
mod status;
mod version;

pub type Result<T> = ::std::result::Result<T, error::Error>;

pub mod defaults {
    use Version;

    pub const MAX_HEADER_COUNT: usize = 32;
    pub const MIN_BUFFER_SIZE: usize = 1024;
    pub const MAX_BUFFER_SIZE: usize = 8096;
    pub const VERSION: Version = Version::Http1_1;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
