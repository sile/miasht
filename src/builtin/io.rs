use std::io::{self, Read};

use TransportStream;
use header::Headers;
use client::Response;

// TODO: timeout, gzip, max-length

#[derive(Debug)]
pub enum BodyReader<R> {
    Chuncked(ChunkedBodyReader<R>),
    Fixed(FixedLengthBodyReader<R>),
    Raw(R),
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
impl<T: TransportStream> From<Response<T>> for BodyReader<Response<T>> {
    fn from(f: Response<T>) -> Self {
        panic!()
    }
}

#[derive(Debug)]
pub struct ChunkedBodyReader<R>(R);
impl<R> ChunkedBodyReader<R> {
    pub fn into_inner(self) -> R {
        self.0
    }
}

#[derive(Debug)]
pub struct FixedLengthBodyReader<R>(R);
impl<R> FixedLengthBodyReader<R> {
    pub fn into_inner(self) -> R {
        self.0
    }
}
