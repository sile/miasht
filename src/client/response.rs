use std::io::{self, Read, BufRead};
use httparse;
use futures::{Future, Poll, Async};

use {Version, Error, Metadata, Method};
use status::RawStatus;
use header::Headers;
use connection::TransportStream;
use unsafe_types::UnsafeRawStatus;
use super::Connection;

#[derive(Debug)]
pub struct ReadResponse<T>(Option<Connection<T>>);
impl<T: TransportStream> ReadResponse<T> {
    pub fn new(mut connection: Connection<T>) -> Self {
        connection.inner.buffer.enter_read_phase();
        ReadResponse(Some(connection))
    }
}
impl<T: TransportStream> Future for ReadResponse<T> {
    type Item = Response<T>;
    type Error = Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut connection = self.0.take().expect("Cannot poll ReadResponse twice");
        let (bytes, headers) = unsafe { connection.inner.buffer_and_headers() };
        let mut res = httparse::Response::new(headers);
        if let httparse::Status::Complete(body_offset) = res.parse(bytes)? {
            connection.inner.buffer.consume(body_offset);
            let version = if res.version.unwrap() == 0 {
                Version::Http1_0
            } else {
                debug_assert_eq!(res.version.unwrap(), 1);
                Version::Http1_1
            };
            let status = RawStatus::new(res.code.unwrap(), res.reason.unwrap());
            Ok(Async::Ready(Response {
                version: version,
                status: status,
                headers: Headers::new(res.headers),
                connection: connection,
            }))
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

#[derive(Debug)]
pub struct Response<T> {
    version: Version,
    status: UnsafeRawStatus,
    headers: Headers<'static>,
    connection: Connection<T>,
}
impl<T> Response<T> {
    pub fn version(&self) -> Version {
        self.version
    }
    pub fn status(&self) -> &RawStatus {
        &self.status
    }
    pub fn headers(&self) -> &Headers {
        &self.headers
    }
    pub fn finish(self) -> Connection<T> {
        self.connection
    }
}
impl<T: TransportStream> Read for Response<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if !self.connection.inner.buffer.is_empty() {
            self.connection.inner.buffer.read(buf)
        } else {
            self.connection.inner.stream.read(buf)
        }
    }
}
impl<T> Metadata for Response<T> {
    fn version(&self) -> Version {
        self.version
    }
    fn headers(&self) -> &Headers {
        &self.headers
    }
    fn status(&self) -> Option<&RawStatus> {
        Some(&self.status)
    }
    fn method(&self) -> Option<Method> {
        None
    }
}
