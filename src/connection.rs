use std::cmp;
use std::io::{self, Read, Write};
use httparse;
use fibers::net::TcpStream;

pub trait TransportStream: Read + Write {}
impl TransportStream for TcpStream {}

pub type UnsafeHeader = httparse::Header<'static>;

// TODO: read/writeをちゃんとハンドリング
// (pipelineの場合でも正常に機能するように)
//
// 具体的には、フェーズがシフトする際には、
// 未処理の部分は、前方に移動させて、
// headをその末尾にセットするようにする。
#[derive(Debug)]
pub struct ByteBuffer {
    bytes: Vec<u8>,
    min_len: usize,
    max_len: usize,
    head: usize,
    tail: usize,
}
impl ByteBuffer {
    pub fn new(min_len: usize, max_len: usize) -> Self {
        assert!(min_len <= max_len);
        ByteBuffer {
            bytes: vec![0; min_len],
            min_len: min_len,
            max_len: max_len,
            head: 0,
            tail: 0,
        }
    }
    pub fn is_full(&self) -> bool {
        self.bytes.len() == self.max_len && self.tail == self.max_len
    }
    pub fn is_empty(&self) -> bool {
        self.head == self.tail
    }
    pub fn consume(&mut self, size: usize) {
        self.head += size;
        assert!(self.head <= self.tail);
    }
    fn reset(&mut self) {
        self.head = 0;
        self.tail = 0;
    }
    pub fn expand_if_needed(&mut self) {
        if self.bytes.len() == self.tail {
            let new_len = cmp::min(self.bytes.len() * 2, self.max_len);
            self.bytes.resize(new_len, 0);
        }
    }
}
impl Read for ByteBuffer {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let size = cmp::min(buf.len(), self.tail - self.head);
        (&mut buf[..size]).copy_from_slice(&self.bytes[self.head..self.head + size]);
        self.head += size;
        Ok(size)
    }
}
impl Write for ByteBuffer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.is_full() {
            Err(io::Error::new(io::ErrorKind::WriteZero, "ByteBuffer overflowed"))
        } else {
            self.expand_if_needed();
            let size = cmp::min(buf.len(), self.bytes.len() - self.tail);
            (&mut self.bytes[self.tail..self.tail + size]).copy_from_slice(buf);
            self.tail += size;
            Ok(size)
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        if self.is_full() {
            Err(io::Error::new(io::ErrorKind::WriteZero, "ByteBuffer overflowed"))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug)]
pub struct HeaderBuffer {
    headers: Vec<UnsafeHeader>,
}
impl HeaderBuffer {
    pub fn new(max_len: usize) -> Self {
        HeaderBuffer { headers: vec![httparse::EMPTY_HEADER; max_len] }
    }
}

#[derive(Debug)]
pub struct Connection<T> {
    byte_buffer: ByteBuffer,
    header_buffer: HeaderBuffer,
    stream: T,
}
impl<T: TransportStream> Connection<T> {
    pub fn new(stream: T, byte_buffer: ByteBuffer, header_buffer: HeaderBuffer) -> Self {
        Connection {
            stream: stream,
            byte_buffer: byte_buffer,
            header_buffer: header_buffer,
        }
    }
    pub fn buffer(&self) -> &ByteBuffer {
        &self.byte_buffer
    }
    pub fn buffer_mut(&mut self) -> &mut ByteBuffer {
        &mut self.byte_buffer
    }
    pub fn flush_buffer(&mut self) -> io::Result<()> {
        let written_size = {
            let buf = &self.byte_buffer.bytes[self.byte_buffer.head..self.byte_buffer.tail];
            self.stream.write(buf)?
        };
        self.byte_buffer.head += written_size;
        if self.byte_buffer.is_empty() {
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::WouldBlock, "Would block"))
        }
    }
    pub fn fill_buffer(&mut self) -> io::Result<bool> {
        self.byte_buffer.expand_if_needed();
        let buf = &mut self.byte_buffer.bytes[self.byte_buffer.tail..];
        assert!(!buf.is_empty()); // TODO
        match self.stream.read(buf) {
            Err(e) => {
                if e.kind() == io::ErrorKind::WouldBlock {
                    Ok(false)
                } else {
                    Err(e)
                }
            }
            Ok(0) => Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Unexpected Eof")),
            Ok(size) => {
                self.byte_buffer.tail += size;
                Ok(true)
            }
        }
    }
    pub fn stream_mut(&mut self) -> &mut T {
        &mut self.stream
    }
    pub fn reset(&mut self) {
        self.byte_buffer.reset();
    }

    pub unsafe fn bytes_and_headers
        (&mut self)
         -> (&'static [u8], &'static mut [httparse::Header<'static>]) {
        let bytes = &self.byte_buffer.bytes[self.byte_buffer.head..self.byte_buffer.tail];
        let mut headers = &mut self.header_buffer.headers[..];
        (&*(bytes as *const _) as &'static _, &mut *(headers as *mut _) as &'static mut _)
    }
}
