#[macro_use]
extern crate error_chain;
extern crate fibers;
extern crate futures;
extern crate httparse;
extern crate handy_async;

pub use client::Client;
pub use server::Server;
pub use method::Method;
pub use status::Status;
pub use traits::Metadata;
pub use version::Version;
pub use connection::TransportStream;

pub mod builtin;
pub mod header;
pub mod client;
pub mod server;
pub mod status;
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

error_chain! {
    errors {
        ServerAborted {
            description("HTTP server is unintentionally exited")
        }
        UnknownMethod(method: String) {
            description("Unknown HTTP method")
            display("Unknown HTTP method: {:?}", method)
        }
        WrongHeader(error: header::ParseValueError<Box<std::error::Error + Send + Sync>>) {
            description("Wrong HTTP header")
            display("Wrong HTTP header: {}", error)
            cause(error)    
        }
        Timeout {
            description("Timed out")
        }
        WithStatus(status: Status, error: Box<std::error::Error + Send + Sync>) {
            description(error.description())
            display("{} (status={:?})", error, status.to_string())
            cause(error)
        }
    }
    foreign_links {
        Parse(httparse::Error);
        Io(std::io::Error);
    }
}
impl<E> From<header::ParseValueError<E>> for Error
    where E: std::error::Error + Send + Sync + 'static
{
    fn from(f: header::ParseValueError<E>) -> Self {
        ErrorKind::WrongHeader(f.boxed()).into()
    }
}
impl Error {
    pub fn with_status<E>(status: Status, error: E) -> Self
        where E: Into<Box<std::error::Error + Send + Sync>>
    {
        ErrorKind::WithStatus(status, error.into()).into()
    }
    pub fn proposed_status(&self) -> Option<Status> {
        match *self.kind() {
            ErrorKind::Parse(_) |
            ErrorKind::UnknownMethod(_) |
            ErrorKind::WrongHeader(_) => Some(Status::BadRequest),
            ErrorKind::WithStatus(status, _) => Some(status),
            ErrorKind::ServerAborted => Some(Status::InternalServerError),
            _ => None
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
