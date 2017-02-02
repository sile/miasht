use fibers::net::TcpStream;
use futures::{self, Future, Finished};

use {Error, Server, TransportStream};
use defaults;
use server::{Connection, HandleSocket, HandleConnection};
use connection2::{ByteBuffer, HeaderBuffer};

pub type RawConnection = Connection<TcpStream>;

#[derive(Debug)]
pub struct SimpleHttpServer<A, F> {
    max_request_header_count: usize,
    min_buffer_size: usize,
    max_buffer_size: usize,
    argument: A,
    callback: fn(A, RawConnection) -> F,
}
impl<A, F> SimpleHttpServer<A, F>
    where A: Clone + Send + 'static,
          F: Future<Item = (), Error = ()> + Send + 'static
{
    pub fn new(argument: A, callback: fn(A, RawConnection) -> F) -> Self {
        SimpleHttpServer {
            max_request_header_count: defaults::MAX_HEADER_COUNT,
            min_buffer_size: defaults::MIN_BUFFER_SIZE,
            max_buffer_size: defaults::MAX_BUFFER_SIZE,
            argument: argument,
            callback: callback,
        }
    }
}
impl<A, F> SimpleHttpServer<A, F> {
    pub fn max_request_header_count(&mut self, count: usize) -> &mut Self {
        self.max_request_header_count = count;
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
}
impl<A, F> Server for SimpleHttpServer<A, F>
    where A: Clone + Send + 'static,
          F: Future<Item = (), Error = ()> + Send + 'static
{
    type Transport = TcpStream;
    type SocketHandler = RawSocketHandler;
    type ConnectionHandler = ConnectionHandleCallback<A, TcpStream, F>;
    fn create_handlers(&mut self) -> (Self::SocketHandler, Self::ConnectionHandler) {
        let socket_handler = RawSocketHandler {
            max_request_header_count: self.max_request_header_count,
            min_buffer_size: self.min_buffer_size,
            max_buffer_size: self.max_buffer_size,
        };
        let connection_handler = ConnectionHandleCallback::new(self.argument.clone(),
                                                               self.callback);
        (socket_handler, connection_handler)
    }
}

#[derive(Debug)]
pub struct RawSocketHandler {
    max_request_header_count: usize,
    min_buffer_size: usize,
    max_buffer_size: usize,
}
impl HandleSocket for RawSocketHandler {
    type Transport = TcpStream;
    type Future = Finished<Connection<Self::Transport>, Error>;
    fn handle(self, socket: TcpStream) -> Self::Future {
        let buffer = ByteBuffer::new(self.min_buffer_size, self.max_buffer_size);
        let headers = HeaderBuffer::new(self.max_request_header_count);
        futures::finished(Connection::new(socket, buffer, headers))
    }
}

#[derive(Debug)]
pub struct ConnectionHandleCallback<A, T, F> {
    argument: A,
    callback: fn(A, Connection<T>) -> F,
}
impl<A, T, F> ConnectionHandleCallback<A, T, F>
    where A: Send + 'static,
          T: TransportStream + 'static,
          F: Future<Item = (), Error = ()> + Send + 'static
{
    pub fn new(argument: A, callback: fn(A, Connection<T>) -> F) -> Self {
        ConnectionHandleCallback {
            argument: argument,
            callback: callback,
        }
    }
}
impl<A, T, F> HandleConnection for ConnectionHandleCallback<A, T, F>
    where A: Send + 'static,
          T: TransportStream + 'static,
          F: Future<Item = (), Error = ()> + Send + 'static
{
    type Transport = T;
    type Future = F;
    fn handle(self, connection: Connection<Self::Transport>) -> Self::Future {
        (self.callback)(self.argument, connection)
    }
}
