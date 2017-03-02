use std::io;
use std::sync::mpsc::RecvError;
use httparse;
use handy_async::error::AsyncError;
use trackable::error::{TrackableError, IntoTrackableError};
use trackable::error::{ErrorKind, ErrorKindExt};

use header;
use status::Status;

pub type Error = TrackableError<Status>;

impl ErrorKind for Status {}
impl IntoTrackableError<io::Error> for Status {
    fn into_trackable_error(from: io::Error) -> Error {
        Status::InternalServerError.cause(from)
    }
}
impl<T> IntoTrackableError<AsyncError<T, io::Error>> for Status {
    fn into_trackable_error(from: AsyncError<T, io::Error>) -> Error {
        Status::InternalServerError.cause(from.into_error())
    }
}
impl IntoTrackableError<httparse::Error> for Status {
    fn into_trackable_error(from: httparse::Error) -> Error {
        Status::BadRequest.cause(from)
    }
}
impl<E: ::std::error::Error + Send + Sync + 'static> IntoTrackableError<header::ParseValueError<E>>
    for Status {
    fn into_trackable_error(from: header::ParseValueError<E>) -> Error {
        Status::BadRequest.cause(from)
    }
}
impl IntoTrackableError<RecvError> for Status {
    fn into_trackable_error(from: RecvError) -> Error {
        Status::BadRequest.cause(from)
    }
}
