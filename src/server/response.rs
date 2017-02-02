use std::io::{self, Write};
use futures::{Future, Poll, Async};

use {Error, Status, TransportStream};
use header::Header;
use super::Connection;

#[derive(Debug)]
pub struct Response<T>(Connection<T>);
impl<T: TransportStream> Response<T> {
    pub fn new(mut connection: Connection<T>, status: Status) -> Self {
        connection.inner.reset();
        let _ = write!(connection.inner.buffer_mut(),
                       "{} {}\r\n",
                       connection.version,
                       status);
        Response(connection)
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
    pub fn into_body_writer(mut self) -> ResponseBodyWriter<T> {
        let _ = write!(self.0.inner.buffer_mut(), "\r\n");
        ResponseBodyWriter(self.0)
    }
    pub fn finish(self) -> FinishResponse<T> {
        self.into_body_writer().finish()
    }
}

#[derive(Debug)]
pub struct ResponseBodyWriter<T>(Connection<T>);
impl<T: TransportStream> ResponseBodyWriter<T> {
    pub fn finish(self) -> FinishResponse<T> {
        FinishResponse(Some(self))
    }
}
impl<T: TransportStream> Write for ResponseBodyWriter<T> {
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
pub struct FinishResponse<T>(Option<ResponseBodyWriter<T>>);
impl<T: TransportStream> Future for FinishResponse<T> {
    type Item = super::Connection<T>;
    type Error = Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut inner = self.0.take().expect("Cannot poll FinishResponse twice");
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
