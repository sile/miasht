use std::io;
use httparse;

#[derive(Debug)]
pub enum Error {
    UnknownMethod(String),
    UnknownVersion(u8),
    ParseFailure(httparse::Error),
    TooLargeRequestHeaderPart,
    TooLargeNonBodyPart,
    ServerAborted,
    Io(io::Error),
    BindFailure(io::Error),
}
