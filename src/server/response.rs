use std::io::{self, Write};
use futures::{Future, Poll, Async};

use {Error, TransportStream};
use status::RawStatus;
use header::Header;
use super::Connection;

pub fn builder<T>(mut connection: Connection<T>, status: RawStatus) -> ResponseBuilder<T>
    where T: TransportStream
{
    connection.inner.buffer.enter_write_phase();
    let _ = write!(connection.inner.buffer,
                   "{} {}\r\n",
                   connection.version,
                   status);
    ResponseBuilder(connection)
}

#[derive(Debug)]
pub struct ResponseBuilder<T>(Connection<T>);
impl<T: TransportStream> ResponseBuilder<T> {
    pub fn add_raw_header(&mut self, name: &str, value: &[u8]) -> &mut Self {
        let _ = write!(self.0.inner.buffer, "{}: ", name);
        let _ = self.0.inner.buffer.write_all(value);
        let _ = write!(self.0.inner.buffer, "\r\n");
        self
    }
    pub fn add_header<H: Header>(&mut self, header: &H) -> &mut Self {
        let _ = write!(self.0.inner.buffer, "{}\r\n", header);
        self
    }
    pub fn finish(mut self) -> Response<T> {
        let _ = write!(self.0.inner.buffer, "\r\n");
        Response(Some(self.0))
    }
}

#[derive(Debug)]
pub struct Response<T>(Option<Connection<T>>);
impl<T: TransportStream> Write for Response<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if let Some(ref mut c) = self.0 {
            c.inner.flush_buffer()?;
            c.inner.stream.write(buf)
        } else {
            Err(io::Error::new(io::ErrorKind::WriteZero,
                               "Cannot write into finished response"))
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
impl<T: TransportStream> Future for Response<T> {
    type Item = Connection<T>;
    type Error = Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.flush() {
            Err(e) => {
                if e.kind() != io::ErrorKind::WouldBlock {
                    bail!(e);
                }
                Ok(Async::NotReady)
            }
            Ok(()) => {
                let connection = self.0.take().expect("Cannot poll Response twice");
                Ok(Async::Ready(connection))
            }
        }
    }
}
