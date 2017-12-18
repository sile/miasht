use std::io::{self, Write};
use futures::{Async, Future, Poll};

use {Error, Status, TransportStream};
use status::RawStatus;
use header::{Header, HeadersMut};
use super::Connection;

pub fn builder<T>(mut connection: Connection<T>, status: RawStatus) -> ResponseBuilder<T>
where
    T: TransportStream,
{
    connection.inner.buffer.enter_write_phase();
    let _ = write!(
        connection.inner.buffer,
        "{} {}\r\n",
        connection.version, status
    );
    ResponseBuilder(connection)
}

#[derive(Debug)]
pub struct ResponseBuilder<T>(Connection<T>);
impl<T: TransportStream> ResponseBuilder<T> {
    pub fn headers_mut(&mut self) -> HeadersMut {
        HeadersMut::new(&mut self.0.inner.buffer)
    }
    pub fn add_raw_header(&mut self, name: &str, value: &[u8]) -> &mut Self {
        self.headers_mut().add_raw_header(name, value);
        self
    }
    pub fn add_header<'a, H: Header<'a>>(&mut self, header: &H) -> &mut Self {
        self.headers_mut().add_header(header);
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
            Err(io::Error::new(
                io::ErrorKind::WriteZero,
                "Cannot write into finished response",
            ))
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
                track_assert_eq!(
                    e.kind(),
                    io::ErrorKind::WouldBlock,
                    Status::InternalServerError
                );
                Ok(Async::NotReady)
            }
            Ok(()) => {
                let connection = self.0.take().expect("Cannot poll Response twice");
                Ok(Async::Ready(connection))
            }
        }
    }
}
