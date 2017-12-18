extern crate fibers;
extern crate futures;
extern crate handy_async;
extern crate httparse;
#[macro_use]
extern crate trackable;

pub use client::Client;
pub use server::Server;
pub use method::Method;
pub use status::Status;
pub use traits::Metadata;
pub use version::Version;
pub use connection::TransportStream;
pub use error::Error;

pub mod builtin;
pub mod header;
pub mod client;
pub mod server;
pub mod status;
mod error;
mod traits;
mod method;
mod version;
mod connection;
mod unsafe_types;

pub mod defaults {
    pub const MAX_HEADER_COUNT: usize = 32;
    pub const MIN_BUFFER_SIZE: usize = 1024;
    pub const MAX_BUFFER_SIZE: usize = 8096;
}

pub type Result<T> = ::std::result::Result<T, Error>;
