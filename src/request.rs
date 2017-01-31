use std::io::{self, Read};
use httparse;
use futures::{Future, Poll, Async};

use {Version, Headers, Result};
use error::Error;
use method::Method;
use connection::Connection;
use response::Response;
use status::Status;

#[derive(Debug)]
pub struct Request<S> {
    connection: Connection<S>,
    method: Method,
    version: Version,
    path: &'static str,
    headers: &'static [httparse::Header<'static>],
}
impl<S> Request<S> {
    pub fn read_from(connection: Connection<S>) -> ReadRequest<S> {
        ReadRequest(Some(connection))
    }
    pub fn method(&self) -> Method {
        self.method
    }
    pub fn version(&self) -> Version {
        self.version
    }
    pub fn path(&self) -> &str {
        self.path
    }
    pub fn headers(&self) -> Headers {
        Headers { headers: self.headers }
    }
    pub fn into_body(self) -> RequestBody<S> {
        RequestBody {
            connection: self.connection,
            version: self.version,
        }
    }
}

#[derive(Debug)]
pub struct RequestBody<S> {
    version: Version,
    connection: Connection<S>,
}
impl<S> RequestBody<S> {
    pub fn into_response(mut self, status: Status) -> Response<S> {
        self.connection.flush_read_buffer();
        Response::new(self.connection, self.version, status)
    }
}
impl<S: Read> Read for RequestBody<S> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.connection.read(buf)
    }
}

#[derive(Debug)]
pub struct ReadRequest<S>(Option<Connection<S>>);
impl<S> ReadRequest<S>
    where S: Read
{
    fn try_parse(&mut self) -> Result<Option<Request<S>>> {
        let mut connection = self.0.take().expect("Cannot poll ReadRequest twice");
        let (bytes, headers) = unsafe { connection.buffer.bytes_and_headers() };
        let mut req = httparse::Request::new(headers);
        match req.parse(bytes) {
            Err(e) => Err(Error::ParseFailure(e)),
            Ok(httparse::Status::Partial) => {
                if connection.is_buffer_full() {
                    Err(Error::TooLargeRequestHeaderPart)
                } else {
                    self.0 = Some(connection);
                    Ok(None)
                }
            }
            Ok(httparse::Status::Complete(body_offset)) => {
                connection.buffer.head = body_offset;
                let method = Method::from_str(req.method.unwrap())?;
                let version = Version::from_u8(req.version.unwrap())?;
                Ok(Some(Request {
                    connection: connection,
                    method: method,
                    version: version,
                    path: req.path.unwrap(),
                    headers: req.headers,
                }))
            }
        }
    }
    fn fill_buffer(&mut self) -> Result<bool> {
        if let Some(ref mut connection) = self.0 {
            match connection.fill_buffer() {
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => Ok(false),
                Err(e) => Err(Error::Io(e)),
                Ok(0) => {
                    Err(Error::Io(io::Error::new(io::ErrorKind::UnexpectedEof,
                                                 "Unexpected EOF while reading HTTP request")))
                }
                Ok(_) => Ok(true),
            }
        } else {
            unreachable!()
        }
    }
}
impl<S> Future for ReadRequest<S>
    where S: Read
{
    type Item = Request<S>;
    type Error = Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Some(request) = self.try_parse()? {
            Ok(Async::Ready(request))
        } else if let true = self.fill_buffer()? {
            self.poll()
        } else {
            Ok(Async::NotReady)
        }
    }
}