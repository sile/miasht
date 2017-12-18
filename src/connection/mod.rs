use std::io::{self, BufRead, Read, Write};
use httparse;
use fibers::net::TcpStream;

use unsafe_types::UnsafeHeader;
pub use self::buffer::Buffer;

mod buffer;

pub trait TransportStream: Read + Write {}
impl TransportStream for TcpStream {}

#[derive(Debug)]
pub struct Connection<T> {
    pub stream: T,
    pub buffer: Buffer,
    headers: Vec<UnsafeHeader>,
}
impl<T: TransportStream> Connection<T> {
    pub fn new(
        stream: T,
        min_buffer_size: usize,
        max_buffer_size: usize,
        max_header_count: usize,
    ) -> Self {
        let buffer = Buffer::new(min_buffer_size, max_buffer_size);
        Connection {
            stream: stream,
            buffer: buffer,
            headers: vec![httparse::EMPTY_HEADER; max_header_count],
        }
    }
    pub fn flush_buffer(&mut self) -> io::Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        let written_size = self.stream.write(self.buffer.fill_buf()?)?;
        self.buffer.consume(written_size);
        if self.buffer.is_empty() {
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::WouldBlock, "Would block"))
        }
    }
    pub fn fill_buffer(&mut self) -> io::Result<bool> {
        match self.buffer.fill_from(&mut self.stream) {
            Err(e) => {
                if e.kind() == io::ErrorKind::WouldBlock {
                    Ok(false)
                } else {
                    Err(e)
                }
            }
            Ok(0) => Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Unexpected Eof",
            )),
            Ok(_) => Ok(true),
        }
    }
    pub unsafe fn buffer_and_headers(&mut self) -> (&'static [u8], &'static mut [UnsafeHeader]) {
        let bytes = self.buffer.as_slice();
        let headers = &mut self.headers[..];
        (
            &*(bytes as *const _) as &'static _,
            &mut *(headers as *mut _) as &'static mut _,
        )
    }
}
