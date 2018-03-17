use std;
use std::io;
use std::sync::mpsc::RecvError;
use httparse;
use trackable::error::TrackableError;
use trackable::error::{ErrorKind, ErrorKindExt};

use header;
use status::Status;

#[derive(Debug, Clone)]
pub struct Error(TrackableError<Status>);
derive_traits_for_trackable_error_newtype!(Error, Status);
impl From<io::Error> for Error {
    fn from(f: io::Error) -> Self {
        Status::InternalServerError.cause(f).into()
    }
}
impl From<httparse::Error> for Error {
    fn from(f: httparse::Error) -> Self {
        Status::BadRequest.cause(f).into()
    }
}
impl<E: std::error::Error + Send + Sync + 'static> From<header::ParseValueError<E>> for Error {
    fn from(f: header::ParseValueError<E>) -> Self {
        Status::BadRequest.cause(f).into()
    }
}
impl From<RecvError> for Error {
    fn from(f: RecvError) -> Self {
        Status::BadRequest.cause(f).into()
    }
}

impl ErrorKind for Status {}
