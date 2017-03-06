use std::fmt;
use std::cmp;
use std::io::{self, Read, Take};
use httparse;

use {Result, Metadata};
use super::headers::{ContentLength, TransferEncoding};

pub trait IoExt: Sized {
    fn into_body_reader(self) -> Result<BodyReader<Self>>
        where Self: Read + Metadata
    {
        BodyReader::new(self)
    }
    fn max_length(self, max_len: u64) -> MaxLength<Self>
        where Self: Read
    {
        MaxLength::new(self, max_len)
    }
}
impl<T: Sized> IoExt for T {}

pub enum BodyReader<R> {
    Chunked(ChunkedBodyReader<R>),
    FixedLength(Take<R>),
    Raw(R),
}
impl<R> BodyReader<R>
    where R: Read + Metadata
{
    pub fn new(inner: R) -> Result<Self> {
        if let Some(h) = track_try!(inner.headers().parse::<ContentLength>()) {
            Ok(BodyReader::FixedLength(inner.take(h.len())))
        } else if let true = track_try!(inner.headers().parse::<TransferEncoding>()).is_some() {
            Ok(BodyReader::Chunked(ChunkedBodyReader::new(inner)))
        } else if inner.is_request() {
            Ok(BodyReader::FixedLength(inner.take(0)))
        } else {
            Ok(BodyReader::Raw(inner))
        }
    }
    pub fn into_inner(self) -> R {
        match self {
            BodyReader::Chunked(r) => r.into_inner(),
            BodyReader::FixedLength(r) => r.into_inner(),
            BodyReader::Raw(r) => r,
        }
    }
}
impl<R: Read> Read for BodyReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match *self {
            BodyReader::Chunked(ref mut r) => r.read(buf),
            BodyReader::FixedLength(ref mut r) => r.read(buf),
            BodyReader::Raw(ref mut r) => r.read(buf),
        }
    }
}
impl<R> fmt::Debug for BodyReader<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BodyReader::Chunked(_) => write!(f, "Chunked(_)"),
            BodyReader::FixedLength(_) => write!(f, "FixedLength(_)"),
            BodyReader::Raw(_) => write!(f, "Raw(_)"),
        }
    }
}

#[derive(Debug)]
pub struct ChunkedBodyReader<R> {
    inner: R,
    buf: [u8; 32],
    buf_offset: usize,
    chunk_remaining: Option<u64>,
    is_last: bool,
}
impl<R: Read> ChunkedBodyReader<R> {
    fn new(inner: R) -> Self {
        ChunkedBodyReader {
            inner: inner,
            buf: [0; 32],
            buf_offset: 0,
            chunk_remaining: None,
            is_last: false,
        }
    }
    pub fn into_inner(self) -> R {
        self.inner
    }
    fn fill_buf_byte(&mut self) -> io::Result<()> {
        if self.buf_offset == self.buf.len() {
            Err(io::Error::new(io::ErrorKind::InvalidData, "Chunk size is too large"))
        } else {
            let buf = &mut self.buf[self.buf_offset..][..1];
            if 0 == self.inner.read(buf)? {
                Err(io::Error::new(io::ErrorKind::UnexpectedEof,
                                   "Unexpected EOF while reading chunked HTTP body"))
            } else {
                self.buf_offset += 1;
                Ok(())
            }
        }
    }
}
impl<R: Read> Read for ChunkedBodyReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if let Some(0) = self.chunk_remaining {
            while self.buf_offset != 2 {
                self.fill_buf_byte()?;
            }
            if &self.buf[0..2] == b"\r\n" {
                self.chunk_remaining = None;
                self.read(buf)
            } else {
                Err(io::Error::new(io::ErrorKind::InvalidData,
                                   r#"Chunk must be terminated with '\r\n'"#))
            }
        } else if let Some(remaining) = self.chunk_remaining {
            let size = cmp::min(remaining, buf.len() as u64) as usize;
            let read_size = self.inner.read(&mut buf[..size])?;
            self.chunk_remaining = Some(remaining - read_size as u64);
            Ok(read_size)
        } else if self.is_last {
            Ok(0)
        } else {
            loop {
                self.fill_buf_byte()?;
                match httparse::parse_chunk_size(&self.buf[..self.buf_offset]) {
                    Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidData, e.to_string())),
                    Ok(httparse::Status::Partial) => {}
                    Ok(httparse::Status::Complete((_, chunk_size))) => {
                        self.buf_offset = 0;
                        self.chunk_remaining = Some(chunk_size);
                        self.is_last = chunk_size == 0;
                        return self.read(buf);
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct MaxLength<R> {
    inner: R,
    read_bytes: u64,
    max_bytes: u64,
}
impl<R: Read> MaxLength<R> {
    pub fn new(inner: R, max_len: u64) -> Self {
        MaxLength {
            inner: inner,
            read_bytes: 0,
            max_bytes: max_len,
        }
    }
    pub fn into_inner(self) -> R {
        self.inner
    }
}
impl<R: Read> Read for MaxLength<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.read_bytes == self.max_bytes {
            let message = format!("Maximum length ({} bytes) exceeded", self.max_bytes);
            Err(io::Error::new(io::ErrorKind::InvalidData, message))
        } else {
            let size = cmp::min(buf.len() as u64, self.max_bytes - self.read_bytes) as usize;
            let read_size = self.inner.read(&mut buf[..size])?;
            self.read_bytes += read_size as u64;
            Ok(read_size)
        }
    }
}
