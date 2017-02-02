use std::io::Write;

use TransportStream;
use status::RawStatus;
use header::Header;
use io::{BodyWriter, Finish};
use super::Connection;

#[derive(Debug)]
pub struct Response<T>(Connection<T>);
impl<T: TransportStream> Response<T> {
    pub fn new(mut connection: Connection<T>, status: RawStatus) -> Self {
        connection.inner.reset();
        let _ = write!(connection.inner.buffer_mut(),
                       "{} {}\r\n",
                       connection.version,
                       status);
        Response(connection)
    }
    pub fn add_raw_header(&mut self, name: &str, value: &[u8]) -> &mut Self {
        let _ = write!(self.0.inner.buffer_mut(), "{}: ", name);
        let _ = self.0.inner.buffer_mut().write_all(value);
        let _ = write!(self.0.inner.buffer_mut(), "\r\n");
        self
    }
    pub fn add_header<H: Header>(&mut self, header: &H) -> &mut Self {
        let _ = write!(self.0.inner.buffer_mut(), "{}\r\n", header);
        self
    }
    pub fn into_body_writer(mut self) -> BodyWriter<Connection<T>, T> {
        let _ = write!(self.0.inner.buffer_mut(), "\r\n");
        BodyWriter::new(self.0)
    }
    pub fn finish(self) -> Finish<Connection<T>, T> {
        self.into_body_writer().finish()
    }
}
