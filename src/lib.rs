#[macro_use]
extern crate log;
extern crate fibers;
extern crate futures;
extern crate httparse;
extern crate handy_async;

mod connection2;

pub use client::Client;
pub use server::Server;
pub use error::Error;
pub use method::Method;
pub use status::{Status, RawStatus};
pub use version::Version;
pub use connection2::TransportStream;

pub mod builtin;
pub mod header;
pub mod client;
pub mod server;
mod error;
mod method;
mod status;
mod version;

pub type Result<T> = ::std::result::Result<T, error::Error>;

pub mod defaults {
    pub const MAX_HEADER_COUNT: usize = 32;
    pub const MIN_BUFFER_SIZE: usize = 1024;
    pub const MAX_BUFFER_SIZE: usize = 8096;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
