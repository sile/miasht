use std::io::{self, Write};
use futures::{Future, Poll, Async};

use {Method, Error};
use header::Header;
use connection2::TransportStream;
use super::Connection;

#[derive(Debug)]
pub struct Request<T>(Connection<T>);
impl<T: TransportStream> Request<T> {
    pub fn new(mut connection: Connection<T>, method: Method, path: &str) -> Self {
        connection.inner.reset();
        let _ = write!(connection.inner.buffer_mut(),
                       "{} {} {}\r\n",
                       method,
                       path,
                       connection.version);
        Request(connection)
    }
    pub fn add_raw_header(&mut self, name: &str, value: &[u8]) -> &mut Self {
        let _ = write!(self.0.inner.buffer_mut(), "{}: ", name);
        let _ = self.0.inner.buffer_mut().write_all(value);
        let _ = write!(self.0.inner.buffer_mut(), "\r\n");
        self
    }
    pub fn add_header<H: Header>(&mut self, header: &H) -> &mut Self {
        let _ = write!(self.0.inner.buffer_mut(), "{}\r\n", header);
        self
    }
    pub fn into_body_writer(mut self) -> RequestBodyWriter<T> {
        let _ = write!(self.0.inner.buffer_mut(), "\r\n");
        RequestBodyWriter(self.0)
    }
    pub fn finish(self) -> FinishRequest<T> {
        self.into_body_writer().finish()
    }
}

#[derive(Debug)]
pub struct RequestBodyWriter<T>(Connection<T>);
impl<T: TransportStream> RequestBodyWriter<T> {
    pub fn finish(self) -> FinishRequest<T> {
        FinishRequest(Some(self))
    }
}
impl<T: TransportStream> Write for RequestBodyWriter<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if !self.0.inner.buffer().is_empty() {
            self.0.inner.flush_buffer()?;
        }
        self.0.inner.stream_mut().write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.0.inner.flush_buffer()?;
        self.0.inner.stream_mut().flush()
    }
}

#[derive(Debug)]
pub struct FinishRequest<T>(Option<RequestBodyWriter<T>>);
impl<T: TransportStream> Future for FinishRequest<T> {
    type Item = super::Connection<T>;
    type Error = Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut inner = self.0.take().expect("Cannot poll FinishRequest twice");
        match inner.flush() {
            Err(e) => {
                if e.kind() == io::ErrorKind::WouldBlock {
                    self.0 = Some(inner);
                    Ok(Async::NotReady)
                } else {
                    Err(Error::Io(e))
                }
            }
            Ok(()) => Ok(Async::Ready(inner.0)),
        }
    }
}
