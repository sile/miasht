use std::io::{Read, Write};
use std::time;
use fibers::time::timer;
use futures::{Future, Poll, Async};
use handy_async::io::{ReadFrom, AsyncWrite};
use handy_async::io::futures::ReadPattern;
use handy_async::io::futures::WriteAll;
use handy_async::pattern::read::{Utf8, All};

use {Error, ErrorKind};
use ResultExt;

pub trait FutureExt: Sized {
    fn timeout<E>(self, delay_from_now: time::Duration) -> Timeout<Self>
        where Self: Future<Error = E>,
              E: From<Error>
    {
        Timeout::new(self, delay_from_now)
    }
    fn write_all_bytes<B: AsRef<[u8]>>(self, buf: B) -> WriteAllBytes<Self, B>
        where Self: Write
    {
        WriteAllBytes::new(self, buf)
    }
    fn read_all_bytes(self) -> ReadAllBytes<Self>
        where Self: Read
    {
        ReadAllBytes::new(self)
    }
    fn read_all_str(self) -> ReadAllStr<Self>
        where Self: Read
    {
        ReadAllStr::new(self)
    }
}
impl<T> FutureExt for T where T: Sized {}

pub struct ReadAllBytes<R: Read>(ReadPattern<All, R>);
impl<R: Read> ReadAllBytes<R> {
    pub fn new(inner: R) -> Self {
        ReadAllBytes(All.read_from(inner))
    }
}
impl<R: Read> Future for ReadAllBytes<R> {
    type Item = (R, Vec<u8>);
    type Error = Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.0.poll().map_err(|e| e.into_error()).chain_err(|| "Cannot read all bytes")
    }
}

pub struct ReadAllStr<R: Read>(ReadPattern<Utf8<All>, R>);
impl<R: Read> ReadAllStr<R> {
    pub fn new(inner: R) -> Self {
        ReadAllStr(Utf8(All).read_from(inner))
    }
}
impl<R: Read> Future for ReadAllStr<R> {
    type Item = (R, String);
    type Error = Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.0.poll().map_err(|e| e.into_error()).chain_err(|| "Cannot read all UTF-8 string")
    }
}

#[derive(Debug)]
pub struct WriteAllBytes<W, B>(WriteAll<W, B>);
impl<W: Write, B: AsRef<[u8]>> WriteAllBytes<W, B> {
    pub fn new(writer: W, buf: B) -> Self {
        WriteAllBytes(writer.async_write_all(buf))
    }
}
impl<W: Write, B: AsRef<[u8]>> Future for WriteAllBytes<W, B> {
    type Item = W;
    type Error = Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.0
            .poll()
            .map_err(|e| e.into_error())
            .chain_err(|| "Cannot write all bytes")
            .map(|v| v.map(|(w, _)| w))
    }
}

pub struct Timeout<F> {
    future: F,
    timeout: timer::Timeout,
}
impl<F, T, E> Timeout<F>
    where F: Future<Item = T, Error = E>,
          E: From<Error>
{
    pub fn new(future: F, delay_from_now: time::Duration) -> Self {
        let timeout = timer::timeout(delay_from_now);
        Timeout {
            future: future,
            timeout: timeout,
        }
    }
}
impl<F, T, E> Future for Timeout<F>
    where F: Future<Item = T, Error = E>,
          E: From<Error>
{
    type Item = T;
    type Error = E;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Async::Ready(()) =
               self.timeout.poll().map_err(|_| "Timeout object unexpectedly aborted".into())? {
                   Err(E::from(ErrorKind::Timeout.into()))
        } else {
            self.future.poll()
        }
    }
}
