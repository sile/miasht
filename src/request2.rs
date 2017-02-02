use std::io::{self, Write};

use {Version, Method};
use header::Header;
use connection2::{Connection, TransportStream};
use response2::ReadResponse;

#[derive(Debug)]
pub struct RequestBuilder<T>(Connection<T>);
impl<T: TransportStream> RequestBuilder<T> {
    pub fn new(mut connection: Connection<T>,
               method: Method,
               path: &str,
               version: Version)
               -> Self {
        connection.reset();
        let _ = write!(connection.buffer_mut(),
                       "{} {} {}\r\n",
                       method,
                       path,
                       version);
        RequestBuilder(connection)
    }
    pub fn add_header<H: Header>(&mut self, header: &H) -> &mut Self {
        let _ = write!(self.0.buffer_mut(), "{}\r\n", header);
        self
    }
    pub fn into_body(mut self) -> RequestBodyWriter<T> {
        let _ = write!(self.0.buffer_mut(), "\r\n");
        RequestBodyWriter(self.0)
    }
}

#[derive(Debug)]
pub struct RequestBodyWriter<T>(Connection<T>);
impl<T: TransportStream> RequestBodyWriter<T> {
    pub fn read_response(self) -> ReadResponse<T> {
        ReadResponse::new(self)
    }
    pub fn into_connection(self) -> Connection<T> {
        self.0
    }
    // TODO: finish
}
impl<T: TransportStream> Write for RequestBodyWriter<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if !self.0.buffer().is_empty() {
            self.0.flush_buffer()?;
        }
        self.0.stream_mut().write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.0.flush_buffer()?;
        self.0.stream_mut().flush()
    }
}
