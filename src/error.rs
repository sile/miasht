use std::io;
use httparse;

pub use header::HeaderParseError;

#[derive(Debug)]
pub enum Error {
    UnknownMethod(String),
    UnknownVersion(u8),
    ParseFailure(httparse::Error),
    HeaderParse(HeaderParseError),
    TooLargeRequestHeaderPart,
    TooLargeNonBodyPart,
    ServerAborted,
    Io(io::Error),
    BindFailure(io::Error),
}
