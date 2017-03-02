use std::io::{self, Read, BufRead};
use httparse;
use futures::{Future, Poll, Async};

use {Error, Version, Method, Status};
use {Metadata, TransportStream};
use status::RawStatus;
use header::Headers;
use super::Connection;

#[derive(Debug)]
pub struct ReadRequest<T>(Option<Connection<T>>);
impl<T: TransportStream> ReadRequest<T> {
    pub fn new(mut connection: Connection<T>) -> Self {
        connection.inner.buffer.enter_read_phase();
        ReadRequest(Some(connection))
    }
}
impl<T: TransportStream> Future for ReadRequest<T> {
    type Item = Request<T>;
    type Error = Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut connection = self.0.take().expect("Cannot poll ReadRequest twice");
        let (bytes, headers) = unsafe { connection.inner.buffer_and_headers() };
        let mut req = httparse::Request::new(headers);
        if let httparse::Status::Complete(body_offset) = track_try!(req.parse(bytes)) {
            connection.inner.buffer.consume(body_offset);
            let version = if req.version.unwrap() == 0 {
                Version::Http1_0
            } else {
                debug_assert_eq!(req.version.unwrap(), 1);
                Version::Http1_1
            };
            let method = if let Some(method) = Method::try_from_str(req.method.unwrap()) {
                method
            } else {
                track_panic!(Status::BadRequest,
                             "Unknown HTTP method: {}",
                             req.method.unwrap().to_string());
            };
            Ok(Async::Ready(Request {
                version: version,
                path: req.path.unwrap(),
                method: method,
                headers: Headers::new(req.headers),
                connection: connection,
            }))
        } else {
            let filled = track_try!(connection.inner.fill_buffer());
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
pub struct Request<T> {
    version: Version,
    path: &'static str,
    method: Method,
    headers: Headers<'static>,
    connection: Connection<T>,
}
impl<T> Request<T> {
    pub fn version(&self) -> Version {
        self.version
    }
    pub fn path(&self) -> &str {
        self.path
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
impl<T: TransportStream> Read for Request<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if !self.connection.inner.buffer.is_empty() {
            self.connection.inner.buffer.read(buf)
        } else {
            self.connection.inner.stream.read(buf)
        }
    }
}
impl<T> Metadata for Request<T> {
    fn version(&self) -> Version {
        self.version
    }
    fn method(&self) -> Option<Method> {
        Some(self.method)
    }
    fn path(&self) -> Option<&str> {
        Some(self.path)
    }
    fn status(&self) -> Option<&RawStatus> {
        None
    }
    fn headers(&self) -> &Headers {
        &self.headers
    }
}
