use httparse;
use futures::{Future, Poll, Async};

use {Error, Version, Method, TransportStream};
use version;
use header::Headers;
use io::BodyReader;
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
        match req.parse(bytes) {
            Err(e) => Err(Error::ParseFailure(e)),
            Ok(httparse::Status::Partial) => {
                if connection.inner.buffer().is_full() {
                    Err(Error::TooLargeNonBodyPart)
                } else {
                    let filled = connection.inner.fill_buffer().map_err(Error::Io)?;
                    self.0 = Some(connection);
                    if filled {
                        self.poll()
                    } else {
                        Ok(Async::NotReady)
                    }
                }
            }
            Ok(httparse::Status::Complete(body_offset)) => {
                connection.inner.buffer_mut().consume(body_offset);
                let version = version::try_from_u8(req.version.unwrap())?;
                let method = Method::try_from_str(req.method.unwrap())
                    .ok_or_else(|| Error::UnknownMethod(req.method.unwrap().to_string()))?;
                Ok(Async::Ready(Request {
                    version: version,
                    method: method,
                    headers: Headers::new(req.headers),
                    connection: connection,
                }))
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
impl<T: TransportStream> Request<T> {
    pub fn version(&self) -> Version {
        self.version
    }
    pub fn method(&self) -> Method {
        self.method
    }
    pub fn headers(&self) -> &Headers {
        &self.headers
    }
    pub fn into_body_reader(self) -> BodyReader<Connection<T>, T> {
        let mut connection = self.connection;
        connection.version = self.version;
        BodyReader::new(connection)
    }
    pub fn finish(self) -> Connection<T> {
        self.into_body_reader().finish()
    }
}