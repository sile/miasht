use std::io::{self, Write};
use std::marker::PhantomData;
use futures::{Future, Poll, Async};

use {Error, TransportStream};
use connection::Connection;

#[derive(Debug)]
pub struct BodyWriter<C, T> {
    inner: C,
    _transport: PhantomData<T>,
}
impl<C, T> BodyWriter<C, T>
    where C: AsMut<Connection<T>>,
          T: TransportStream
{
    pub fn new(connection: C) -> Self {
        BodyWriter {
            inner: connection,
            _transport: PhantomData,
        }
    }
    pub fn finish(self) -> Finish<C, T> {
        Finish(Some(self))
    }
}
impl<C, T> Write for BodyWriter<C, T>
    where C: AsMut<Connection<T>>,
          T: TransportStream
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if !self.inner.as_mut().buffer().is_empty() {
            self.inner.as_mut().flush_buffer()?;
        }
        self.inner.as_mut().stream_mut().write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.inner.as_mut().flush_buffer()?;
        self.inner.as_mut().stream_mut().flush()
    }
}

#[derive(Debug)]
pub struct Finish<C, T>(Option<BodyWriter<C, T>>);
impl<C, T> Future for Finish<C, T>
    where C: AsMut<Connection<T>>,
          T: TransportStream
{
    type Item = C;
    type Error = Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut inner = self.0.take().expect("Cannot poll Finish twice");
        match inner.flush() {
            Err(e) => {
                if e.kind() != io::ErrorKind::WouldBlock {
                    bail!(e);
                }
                self.0 = Some(inner);
                Ok(Async::NotReady)
            }
            Ok(()) => Ok(Async::Ready(inner.inner)),
        }
    }
}
