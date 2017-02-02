use std::io;
use httparse;

use header;

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
