use std::io::{self, Read};
use httparse;
use futures::{Future, Poll, Async};

use {Error, ErrorKind, Version, Method, TransportStream};
use header::{Headers, GetHeaders};
use super::Connection;

#[derive(Debug)]
pub struct ReadRequest<T>(Option<Connection<T>>);
impl<T: TransportStream> ReadRequest<T> {
    pub fn new(mut connection: Connection<T>) -> Self {
        connection.inner.reset();
        ReadRequest(Some(connection))
    }
}
impl<T: TransportStream> Future for ReadRequest<T> {
    type Item = Request<T>;
    type Error = Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut connection = self.0.take().expect("Cannot poll ReadRequest twice");
        let (bytes, headers) = unsafe { connection.inner.bytes_and_headers() };
        let mut req = httparse::Request::new(headers);
        if let httparse::Status::Complete(body_offset) = req.parse(bytes)? {
            connection.inner.buffer_mut().consume(body_offset);
            let version = if req.version.unwrap() == 0 {
                Version::Http1_0
            } else {
                debug_assert_eq!(req.version.unwrap(), 1);
                Version::Http1_1
            };
            let method = Method::try_from_str(req.method.unwrap())
                    .ok_or_else(|| ErrorKind::UnknownMethod(req.method.unwrap().to_string()))?;
            Ok(Async::Ready(Request {
                version: version,
                method: method,
                headers: Headers::new(req.headers),
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
pub struct Request<T> {
    version: Version,
    method: Method,
    headers: Headers<'static>,
    connection: Connection<T>,
}
impl<T> Request<T> {
    pub fn version(&self) -> Version {
        self.version
    }
    pub fn method(&self) -> Method {
        self.method
    }
    pub fn headers(&self) -> &Headers {
        &self.headers
    }
    pub fn finish(mut self) -> Connection<T> {
        self.connection.version = self.version;
        self.connection
    }
}
impl<T> GetHeaders for Request<T> {
    fn get_headers(&self) -> &Headers {
        self.headers()
    }
}
impl<T: TransportStream> Read for Request<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if !self.connection.inner.buffer().is_empty() {
            self.connection.inner.buffer_mut().read(buf)
        } else {
            self.connection.inner.stream_mut().read(buf)
        }
    }
}
