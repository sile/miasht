use std::io::{self, Read, Write};
use httparse;

#[derive(Debug)]
pub struct Buffer {
    pub bytes: Vec<u8>,
    pub head: usize,
    pub tail: usize,
    pub headers: Vec<httparse::Header<'static>>,
}
impl Buffer {
    pub fn new() -> Self {
        Buffer {
            bytes: vec![0; 1024],
            head: 0,
            tail: 0,
            headers: vec![httparse::EMPTY_HEADER; 32],
        }
    }
    pub unsafe fn bytes_and_headers
        (&mut self)
         -> (&'static [u8], &'static mut [httparse::Header<'static>]) {
        let bytes = &self.bytes[self.head..self.tail];
        let mut headers = &mut self.headers[..];
        (&*(bytes as *const _) as &'static _, &mut *(headers as *mut _) as &'static mut _)
    }
}

#[derive(Debug)]
pub struct Connection<S> {
    pub buffer: Buffer,
    pub stream: S,
}
impl<S> Connection<S>
    where S: Read + Write
{
    pub fn new(stream: S) -> Self {
        Connection {
            stream: stream,
            buffer: Buffer::new(),
        }
    }
}
impl<S: Read> Connection<S> {
    pub fn fill_buffer(&mut self) -> io::Result<usize> {
        let buffer = &mut self.buffer;
        let read_size = self.stream.read(&mut buffer.bytes[buffer.tail..])?;
        buffer.tail += read_size;
        Ok(read_size)
    }
}
impl<S> Connection<S> {
    pub fn is_buffer_full(&self) -> bool {
        self.buffer.bytes.len() == self.buffer.tail
    }
    pub fn flush_read_buffer(&mut self) {
        self.buffer.head = 0;
        self.buffer.tail = 0;
    }
}
impl<S: Read> Read for Connection<S> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.buffer.head < self.buffer.tail {
            use std::cmp;
            let size = cmp::min(buf.len(), self.buffer.tail - self.buffer.head);
            (&mut buf[..size])
                .copy_from_slice(&self.buffer.bytes[self.buffer.head..self.buffer.head + size]);
            self.buffer.head += size;
            Ok(size)
        } else {
            self.buffer.head = 0;
            self.buffer.tail = 0;
            self.stream.read(buf)
        }
    }
}
