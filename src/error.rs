use std::io;
use httparse;

use header;

// TODO: impl Error

#[derive(Debug)]
pub enum Error {
    UnknownMethod(String),
    UnknownVersion(u8),
    TooLargeNonBodyPart,
    ParseFailure(httparse::Error),
    HeaderParse(header::ParseError),
    ServerAborted,
    Io(io::Error),
}
