pub use self::request::{Request, RequestBuilder};
pub use self::response::{ReadResponse, Response};

use {Method, Version};
use connection::{self, TransportStream};

mod request;
mod response;

#[derive(Debug)]
pub struct Connection<T> {
    inner: connection::Connection<T>,
    version: Version,
}
impl<T: TransportStream> Connection<T> {
    // fn new(stream: T, client: &Client) -> Self {
    //     let max_header_count = client.max_response_header_count;
    //     let inner = connection::Connection::new(
    //         stream,
    //         client.min_buffer_size,
    //         client.max_buffer_size,
    //         max_header_count,
    //     );
    //     Connection {
    //         inner: inner,
    //         version: client.version,
    //     }
    // }
    pub fn build_request(self, method: Method, path: &str) -> RequestBuilder<T> {
        request::builder(self, method, path)
    }

    pub fn read_response(self) -> ReadResponse<T> {
        ReadResponse::new(self)
    }
}
impl<T> AsMut<connection::Connection<T>> for Connection<T> {
    fn as_mut(&mut self) -> &mut connection::Connection<T> {
        &mut self.inner
    }
}
