use std::fmt;
use std::io::{self, Read, Take};

use {Result, Error, Status, Metadata};
use header::Headers;
use super::headers::{ContentLength, TransferEncoding};

// TODO: timeout, gzip

#[derive(Debug)]
pub enum BodyReader<R> {
    Chunked(ChunkedBodyReader<R>),
    Fixed(FixedLengthBodyReader<R>),
    Raw(R),
}
impl<R> BodyReader<R>
    where R: Read + Metadata
{
    pub fn new(inner: R) -> Result<Self> {
        if let Some(h) = inner.headers().parse::<ContentLength>()? {
            Ok(BodyReader::Fixed(FixedLengthBodyReader(inner.take(h.len()))))
        } else if let true = Self::is_chunked(inner.headers())? {
            Ok(BodyReader::Chunked(ChunkedBodyReader(inner)))
        } else {
            Ok(BodyReader::Raw(inner))
        }
    }
    fn is_chunked(headers: &Headers) -> Result<bool> {
        if let Some(h) = headers.parse::<TransferEncoding>()? {
            h.chunked_or_else(|unknown| {
                    let error = format!("Cannot understand transfer-coding {:?}", unknown);
                    Error::with_status(Status::NotImplemented, error)
                })?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    pub fn into_inner(self) -> R {
        match self {
            BodyReader::Chunked(r) => r.into_inner(),
            BodyReader::Fixed(r) => r.into_inner(),
            BodyReader::Raw(r) => r,
        }
    }
}

#[derive(Debug)]
pub struct ChunkedBodyReader<R>(R);
impl<R> ChunkedBodyReader<R> {
    pub fn into_inner(self) -> R {
        self.0
    }
}

pub struct FixedLengthBodyReader<R>(Take<R>);
impl<R: Read> FixedLengthBodyReader<R> {
    pub fn into_inner(self) -> R {
        self.0.into_inner()
    }
}
impl<R> fmt::Debug for FixedLengthBodyReader<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FixedLengthBodyReader(_)")
    }
}
impl<R: Read> Read for FixedLengthBodyReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}
