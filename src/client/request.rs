use std::io::{self, Write};
use futures::{Future, Poll, Async};

use {Error, Method, Status};
use header::Header;
use connection::TransportStream;
use super::Connection;

pub fn builder<T>(mut connection: Connection<T>, method: Method, path: &str) -> RequestBuilder<T>
    where T: TransportStream
{
    connection.inner.buffer.enter_write_phase();
    let _ = write!(connection.inner.buffer,
                   "{} {} {}\r\n",
                   method,
                   path,
                   connection.version);
    RequestBuilder(connection)
}

#[derive(Debug)]
pub struct RequestBuilder<T>(Connection<T>);
impl<T: TransportStream> RequestBuilder<T> {
    pub fn add_raw_header(&mut self, name: &str, value: &[u8]) -> &mut Self {
        let _ = write!(self.0.inner.buffer, "{}: ", name);
        let _ = self.0.inner.buffer.write_all(value);
        let _ = write!(self.0.inner.buffer, "\r\n");
        self
    }
    pub fn add_header<'a, H: Header<'a>>(&mut self, header: &H) -> &mut Self {
        let _ = write!(self.0.inner.buffer, "{}: ", H::name());
        let _ = header.write_value(&mut self.0.inner.buffer);
        let _ = write!(self.0.inner.buffer, "\r\n");
        self
    }
    pub fn finish(mut self) -> Request<T> {
        let _ = write!(self.0.inner.buffer, "\r\n");
        Request(Some(self.0))
    }
}

#[derive(Debug)]
pub struct Request<T>(Option<Connection<T>>);
impl<T: TransportStream> Write for Request<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if let Some(ref mut c) = self.0 {
            c.inner.flush_buffer()?;
            c.inner.stream.write(buf)
        } else {
            Err(io::Error::new(io::ErrorKind::WriteZero,
                               "Cannot write into finished request"))
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        if let Some(ref mut c) = self.0 {
            c.inner.flush_buffer()?;
            c.inner.stream.flush()?;
        }
        Ok(())
    }
}
impl<T: TransportStream> Future for Request<T> {
    type Item = Connection<T>;
    type Error = Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.flush() {
            Err(e) => {
                track_assert_eq!(e.kind(),
                                 io::ErrorKind::WouldBlock,
                                 Status::InternalServerError);
                Ok(Async::NotReady)
            }
            Ok(()) => {
                let connection = self.0.take().expect("Cannot poll Request twice");
                Ok(Async::Ready(connection))
            }
        }
    }
}
