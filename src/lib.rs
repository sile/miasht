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
pub use version::Version;
pub use connection::TransportStream;

pub mod builtin;
pub mod header;
pub mod client;
pub mod server;
pub mod status;
pub mod io;
mod method;
mod version;
mod connection;

pub mod defaults {
    pub const MAX_HEADER_COUNT: usize = 32;
    pub const MIN_BUFFER_SIZE: usize = 1024;
    pub const MAX_BUFFER_SIZE: usize = 8096;
}

error_chain!{
    errors {
        TooLargeNonBodyPart {
            description("Too large non-body part")
        }
        ServerAborted {
            description("HTTP server is unintentionally exited")
        }
        UnknownMethod(method: String) {
            description("Unknown HTTP method")
            display("Unknown HTTP method: {:?}", method)
        }
    }
    foreign_links {
        Parse(httparse::Error);
        Io(std::io::Error);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
