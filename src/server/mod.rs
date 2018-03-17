pub use self::request::{ReadRequest, Request};
pub use self::response::{Response, ResponseBuilder};

use {TransportStream, Version};
use connection;
use status::RawStatus;

mod request;
mod response;

#[derive(Debug)]
pub struct Connection<T> {
    inner: connection::Connection<T>,
    version: Version,
}
impl<T: TransportStream> Connection<T> {
    pub fn new(
        stream: T,
        min_buffer_size: usize,
        max_buffer_size: usize,
        max_header_count: usize,
    ) -> Self {
        let inner =
            connection::Connection::new(stream, min_buffer_size, max_buffer_size, max_header_count);
        Connection {
            inner: inner,
            version: Version::default(),
        }
    }
    pub fn read_request(self) -> ReadRequest<T> {
        ReadRequest::new(self)
    }
    pub fn build_response<'a, S>(self, status: S) -> ResponseBuilder<T>
    where
        S: Into<RawStatus<'a>>,
    {
        response::builder(self, status.into())
    }
    pub fn into_raw_stream(self) -> T {
        self.inner.stream
    }
}
impl<T> AsMut<connection::Connection<T>> for Connection<T> {
    fn as_mut(&mut self) -> &mut connection::Connection<T> {
        &mut self.inner
    }
}
