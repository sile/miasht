use std::mem;
use std::io::{self, Read};
use futures::{Future, Poll, Async};
use handy_async::io::AsyncWrite;
use handy_async::io::futures::Flush;
use httparse;

use {Version, Headers};
use status::RawStatus;
use connection2::{Connection, TransportStream};
use request2::RequestBodyWriter;
use error::Error;

#[derive(Debug)]
pub enum ReadResponse<T> {
    Flush(Flush<RequestBodyWriter<T>>),
    Read(ReadResponseInner<T>),
    Polled,
}
impl<T: TransportStream> ReadResponse<T> {
    pub fn new(request: RequestBodyWriter<T>) -> Self {
        ReadResponse::Flush(request.async_flush())
    }
}
impl<T: TransportStream> Future for ReadResponse<T> {
    type Item = Response<T>;
    type Error = Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match mem::replace(self, ReadResponse::Polled) {
            ReadResponse::Flush(mut f) => {
                if let Async::Ready(request) = f.poll().map_err(|e| Error::Io(e.into_error()))? {
                    *self = ReadResponse::Read(ReadResponseInner(Some(request.into_connection())));
                    self.poll()
                } else {
                    Ok(Async::NotReady)
                }
            }
            ReadResponse::Read(mut f) => {
                if let Async::Ready(response) = f.poll()? {
                    Ok(Async::Ready(response))
                } else {
                    *self = ReadResponse::Read(f);
                    Ok(Async::NotReady)
                }
            }
            ReadResponse::Polled => panic!("Cannot poll ReadResponse twice"),
        }
    }
}

#[derive(Debug)]
pub struct ReadResponseInner<T>(Option<Connection<T>>);
impl<T: TransportStream> Future for ReadResponseInner<T> {
    type Item = Response<T>;
    type Error = Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut connection = self.0.take().expect("Cannot poll ReadResponseInner twice");
        let (bytes, headers) = unsafe { connection.bytes_and_headers() };
        let mut res = httparse::Response::new(headers);
        match res.parse(bytes) {
            Err(e) => Err(Error::ParseFailure(e)),
            Ok(httparse::Status::Partial) => {
                if connection.buffer().is_full() {
                    Err(Error::TooLargeNonBodyPart)
                } else {
                    let filled = connection.fill_buffer().map_err(Error::Io)?;
                    self.0 = Some(connection);
                    if filled {
                        self.poll()
                    } else {
                        Ok(Async::NotReady)
                    }
                }
            }
            Ok(httparse::Status::Complete(body_offset)) => {
                connection.buffer_mut().consume(body_offset);
                let version = match res.version.unwrap() {
                    0 => Version::Http1_0,
                    1 => Version::Http1_1,
                    v => Err(Error::UnknownVersion(v))?,
                };
                let status = RawStatus::new(res.code.unwrap(), res.reason.unwrap());
                Ok(Async::Ready(Response {
                    version: version,
                    status: status,
                    headers: Headers::new(res.headers),
                    connection: connection,
                }))
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
    pub fn into_body(self) -> ResponseBodyReader<T> {
        ResponseBodyReader(self.connection)
    }
}

#[derive(Debug)]
pub struct ResponseBodyReader<T>(Connection<T>);
impl<T: TransportStream> ResponseBodyReader<T> {
    pub fn into_connection(mut self) -> Connection<T> {
        self.0.reset();
        self.0
    }
}
impl<T: TransportStream> Read for ResponseBodyReader<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if !self.0.buffer().is_empty() {
            self.0.buffer_mut().read(buf)
        } else {
            self.0.stream_mut().read(buf)
        }
    }
}
