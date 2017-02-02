use httparse;
use futures::{Future, Poll, Async};

use {Version, Error, ErrorKind};
use status::RawStatus;
use version;
use header::Headers;
use connection::TransportStream;
use io::BodyReader;
use super::Connection;

#[derive(Debug)]
pub struct ReadResponse<T>(Option<Connection<T>>);
impl<T: TransportStream> ReadResponse<T> {
    pub fn new(mut connection: Connection<T>) -> Self {
        connection.inner.reset();
        ReadResponse(Some(connection))
    }
}
impl<T: TransportStream> Future for ReadResponse<T> {
    type Item = Response<T>;
    type Error = Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut connection = self.0.take().expect("Cannot poll ReadResponse twice");
        let (bytes, headers) = unsafe { connection.inner.bytes_and_headers() };
        let mut res = httparse::Response::new(headers);
        if let httparse::Status::Complete(body_offset) = res.parse(bytes)? {
            connection.inner.buffer_mut().consume(body_offset);
            let version = version::from_u8(res.version.unwrap());
            let status = RawStatus::new(res.code.unwrap(), res.reason.unwrap());
            Ok(Async::Ready(Response {
                version: version,
                status: status,
                headers: Headers::new(res.headers),
                connection: connection,
            }))
        } else {
            if connection.inner.buffer().is_full() {
                Err(ErrorKind::TooLargeNonBodyPart.into())
            } else {
                let filled = connection.inner.fill_buffer()?;
                self.0 = Some(connection);
                if filled {
                    self.poll()
                } else {
                    Ok(Async::NotReady)
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Response<T> {
    version: Version,
    status: RawStatus<'static>,
    headers: Headers<'static>,
    connection: Connection<T>,
}
impl<T: TransportStream> Response<T> {
    pub fn version(&self) -> Version {
        self.version
    }
    pub fn status(&self) -> &RawStatus {
        &self.status
    }
    pub fn headers(&self) -> &Headers {
        &self.headers
    }
    pub fn into_body_reader(self) -> BodyReader<Connection<T>, T> {
        BodyReader::new(self.connection)
    }
    pub fn finish(self) -> Connection<T> {
        self.into_body_reader().finish()
    }
}
