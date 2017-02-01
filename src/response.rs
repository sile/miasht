use std::io::{self, Write};

use connection::Connection;

use {Version, Header};
use status::Status;

#[derive(Debug)]
pub struct Response<S> {
    connection: Connection<S>,
}
impl<S> Response<S> {
    pub fn new(mut connection: Connection<S>, version: Version, status: Status) -> Self {
        connection.flush_read_buffer();
        connection.buffer.bytes.clear();
        connection.buffer.tail = 1; // XXX:
        write!(connection.buffer.bytes, "{} {}\r\n", version, status).unwrap();
        Response { connection: connection }
    }

    pub fn add_header<H: Header>(&mut self, header: &H) -> &mut Self {
        header.write(&mut self.connection.buffer.bytes);
        self.connection.buffer.bytes.extend_from_slice(b"\r\n");
        self
    }

    pub fn into_body(mut self) -> ResponseBody<S> {
        self.connection.buffer.bytes.extend_from_slice(b"\r\n");
        ResponseBody(self.connection)
    }
}

#[derive(Debug)]
pub struct ResponseBody<S>(Connection<S>);
impl<S> ResponseBody<S> {
    pub fn into_connection(self) -> Connection<S> {
        self.0
    }
}
impl<S: Write> ResponseBody<S> {
    fn write_pre_body_bytes(&mut self) -> io::Result<()> {
        if self.0.buffer.tail == 0 {
            Ok(())
        } else {
            let written_size = self.0.stream.write(&self.0.buffer.bytes[self.0.buffer.head..])?;
            self.0.buffer.head += written_size;
            if self.0.buffer.head == self.0.buffer.bytes.len() {
                self.0.buffer.reset();
                Ok(())
            } else {
                Err(io::Error::new(io::ErrorKind::WouldBlock, "Would block"))
            }
        }
    }
}
impl<S: Write> Write for ResponseBody<S> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write_pre_body_bytes()?;
        self.0.stream.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.write_pre_body_bytes()?;
        self.0.stream.flush()
    }
}
