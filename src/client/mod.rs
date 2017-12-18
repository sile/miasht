use std::net::SocketAddr;
use fibers::net::{self, TcpStream};
use futures::{Future, Poll};

pub use self::request::{Request, RequestBuilder};
pub use self::response::{Response, ReadResponse};

use {Error, Method, Version};
use defaults;
use connection::{self, TransportStream};

mod request;
mod response;

/// HTTP client.
#[derive(Debug, Clone)]
pub struct Client {
    max_response_header_count: usize,
    min_buffer_size: usize,
    max_buffer_size: usize,
    version: Version,

    nodelay: bool,
}
impl Client {
    pub fn new() -> Self {
        Client {
            max_response_header_count: defaults::MAX_HEADER_COUNT,
            min_buffer_size: defaults::MIN_BUFFER_SIZE,
            max_buffer_size: defaults::MAX_BUFFER_SIZE,
            version: Version::default(),
            nodelay: true,
        }
    }
    pub fn max_response_header_count(&mut self, count: usize) -> &mut Self {
        self.max_response_header_count = count;
        self
    }
    pub fn min_buffer_size(&mut self, size: usize) -> &mut Self {
        assert!(size <= self.max_buffer_size);
        self.min_buffer_size = size;
        self
    }
    pub fn max_buffer_size(&mut self, size: usize) -> &mut Self {
        assert!(self.min_buffer_size <= size);
        self.max_buffer_size = size;
        self
    }
    pub fn version(&mut self, version: Version) -> &mut Self {
        self.version = version;
        self
    }
    pub fn nodelay(&mut self, b: bool) -> &mut Self {
        self.nodelay = b;
        self
    }
    pub fn connect(&self, server_addr: SocketAddr) -> Connect {
        Connect {
            client: self.clone(),
            future: TcpStream::connect(server_addr),
            nodelay: self.nodelay,
        }
    }
}

#[derive(Debug)]
pub struct Connection<T> {
    inner: connection::Connection<T>,
    version: Version,
}
impl<T: TransportStream> Connection<T> {
    fn new(stream: T, client: &Client) -> Self {
        let max_header_count = client.max_response_header_count;
        let inner = connection::Connection::new(
            stream,
            client.min_buffer_size,
            client.max_buffer_size,
            max_header_count,
        );
        Connection {
            inner: inner,
            version: client.version,
        }
    }
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

#[derive(Debug)]
pub struct Connect {
    client: Client,
    future: net::futures::Connect,
    nodelay: bool,
}
impl Future for Connect {
    type Item = Connection<TcpStream>;
    type Error = Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok(
            track!(self.future.poll().map_err(Error::from))?.map(|socket| {
                unsafe {
                    let _ = socket.with_inner(|socket| socket.set_nodelay(self.nodelay));
                }
                Connection::new(socket, &self.client)
            }),
        )
    }
}
