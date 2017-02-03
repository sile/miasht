#![allow(unused_imports)]
#![allow(unused_variables)]
use std::fmt;
use std::io::{self, Read, Take};

use {Result, TransportStream};
use header::{Headers, GetHeaders};
use client::Response;
use super::headers::ContentLength;

// TODO: timeout, gzip

#[derive(Debug)]
pub enum BodyReader<R> {
    Chuncked(ChunkedBodyReader<R>),
    Fixed(FixedLengthBodyReader<R>),
    Raw(R),
}
impl<R> BodyReader<R>
    where R: TransportStream + GetHeaders
{
    pub fn new(inner: R) -> Result<Self> {
        if let Some(h) = inner.get_headers().parse::<ContentLength>()? {
            Ok(BodyReader::Fixed(FixedLengthBodyReader::new(inner, h.len())))
        } else {
            Ok(BodyReader::Raw(inner))
        }
    }
}
impl<R> BodyReader<R> {
    pub fn into_inner(self) -> R {
        match self {
            BodyReader::Chuncked(r) => r.into_inner(),
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
    pub fn new(inner: R, limit: u64) -> Self {
        FixedLengthBodyReader(inner.take(limit))
    }
}
impl<R> FixedLengthBodyReader<R> {
    pub fn into_inner(self) -> R {
        self.0.into_inner()
    }
}
impl<R> fmt::Debug for FixedLengthBodyReader<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FixedLengthBodyReader(_)")
    }
}
