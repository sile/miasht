use std::io;
use httparse;

use header;

// TODO: impl Error

#[derive(Debug)]
pub enum Error {
    UnknownMethod(String),
    UnknownVersion(u8),
    ParseFailure(httparse::Error),
    HeaderParse(header::ParseError),
    TooLargeRequestHeaderPart,
    TooLargeNonBodyPart,
    ServerAborted,
    Io(io::Error),
    BindFailure(io::Error),
}
