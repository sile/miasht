use std::io;
use httparse;

pub enum Error {
    UnknownMethod(String),
    UnknownVersion(u8),
    ParseFailure(httparse::Error),
    TooLargeRequestHeaderPart,
    Io(io::Error),
}
