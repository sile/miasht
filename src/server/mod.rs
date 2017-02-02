use std::net::SocketAddr;
use fibers::Spawn;
use fibers::sync::{mpsc, oneshot};
use fibers::net::{TcpListener, TcpStream};
use fibers::net::streams::Incoming;
use futures::{self, Future, Stream, Poll, Async};
use futures::future::Either;

pub use self::request::{Request, ReadRequest};
pub use self::response::Response;

use {Result, Error, Status, TransportStream, Version};
use connection2::{self, ByteBuffer, HeaderBuffer};

mod request;
mod response;

pub trait Server {
    type Transport: TransportStream;
    type SocketHandler: HandleSocket<Transport = Self::Transport>;
    type ConnectionHandler: HandleConnection<Transport = Self::Transport>;

    #[allow(unused_variables)]
    fn before_listen(&mut self, listener: &mut TcpListener) -> Result<()> {
        Ok(())
    }
    fn create_handlers(&mut self) -> (Self::SocketHandler, Self::ConnectionHandler);
    fn start<S>(self, bind_addr: SocketAddr, spawner: S) -> ServerHandle
        where Self: Sized + Send + 'static,
              S: Spawn + Clone + Send + 'static
    {
        ServerHandle::start(self, bind_addr, spawner)
    }
}

pub trait HandleSocket: Sized + Send + 'static {
    type Transport: TransportStream;
    type Future: Future<Item = Connection<Self::Transport>, Error = Error> + Send + 'static;
    fn handle(self, socket: TcpStream) -> Self::Future;
}

pub trait HandleConnection: Sized + Send + 'static {
    type Transport: TransportStream;
    type Future: Future<Item = (), Error = ()> + Send + 'static;

    fn handle(self, connection: Connection<Self::Transport>) -> Self::Future;

    #[allow(unused_variables)]
    fn on_error(self, client: SocketAddr, error: Error) {}
}

#[derive(Debug)]
pub struct Connection<T> {
    inner: connection2::Connection<T>,
    version: Version,
}
impl<T: TransportStream> Connection<T> {
    pub fn new(stream: T, buffer: ByteBuffer, headers: HeaderBuffer) -> Self {
        let inner = connection2::Connection::new(stream, buffer, headers);
        Connection {
            inner: inner,
            version: Version::default(),
        }
    }
    pub fn read_request(self) -> ReadRequest<T> {
        ReadRequest::new(self)
    }
    pub fn response(self, status: Status) -> Response<T> {
        Response::new(self, status)
    }
}
impl<T> AsMut<connection2::Connection<T>> for Connection<T> {
    fn as_mut(&mut self) -> &mut connection2::Connection<T> {
        &mut self.inner
    }
}

#[derive(Debug)]
enum Command {
    Stop,
}

#[derive(Debug)]
pub struct ServerHandle {
    command_tx: mpsc::Sender<Command>,
    monitor: oneshot::Monitor<(), Error>,
}
impl ServerHandle {
    fn start<S, T>(mut server: S, bind_addr: SocketAddr, spawner: T) -> ServerHandle
        where S: Server + Send + 'static,
              T: Spawn + Clone + Send + 'static
    {
        let (command_tx, command_rx) = mpsc::channel();
        let future = {
            let spawner = spawner.clone();
            TcpListener::bind(bind_addr)
                .map_err(|e| Error::BindFailure(e))
                .and_then(move |mut listener| if let Err(e) =
                    server.before_listen(&mut listener) {
                    Either::A(futures::failed(e))
                } else {
                    let server_loop = ServerLoop {
                        server: server,
                        spawner: spawner,
                        incoming: listener.incoming(),
                        command_rx: command_rx,
                    };
                    Either::B(server_loop)
                })
        };
        let monitor = spawner.spawn_monitor(future);
        ServerHandle {
            monitor: monitor,
            command_tx: command_tx,
        }
    }
    pub fn stop(self) -> Join {
        let _ = self.command_tx.send(Command::Stop);
        Join(self)
    }
    pub fn join(self) -> Join {
        Join(self)
    }
}

#[derive(Debug)]
pub struct Join(ServerHandle);
impl Future for Join {
    type Item = ();
    type Error = Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.0.monitor.poll().map_err(|e| e.unwrap_or(Error::ServerAborted))
    }
}

#[derive(Debug)]
struct ServerLoop<S, T> {
    server: S,
    spawner: T,
    incoming: Incoming,
    command_rx: mpsc::Receiver<Command>,
}
impl<S, T> ServerLoop<S, T> {
    fn handle_command(&mut self, command: Command) -> Option<Result<()>> {
        match command {
            Command::Stop => Some(Ok(())),
        }
    }
}
impl<S, T> Future for ServerLoop<S, T>
    where S: Server,
          T: Spawn
{
    type Item = ();
    type Error = Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        'toplevel: loop {
            match self.command_rx.poll().expect("unreachable") {
                Async::Ready(None) => return Ok(Async::Ready(())),
                Async::Ready(Some(command)) => {
                    if let Some(shutdown) = self.handle_command(command) {
                        return shutdown.map(|()| Async::Ready(()));
                    } else {
                        continue;
                    }
                }
                Async::NotReady => {}
            }
            // FIXME: delete below loop in fibers-v0.1.3
            for _ in 0..2 {
                match self.incoming.poll().map_err(Error::Io)? {
                    Async::NotReady => {
                        // return Ok(Async::NotReady);
                    }
                    Async::Ready(None) => unreachable!(),
                    Async::Ready(Some((socket, address))) => {
                        let (socket_handler, connection_handler) = self.server.create_handlers();
                        self.spawner.spawn(socket.map_err(Error::Io)
                            .and_then(move |socket| socket_handler.handle(socket))
                            .then(move |result| match result {
                                Err(e) => {
                                    connection_handler.on_error(address, e);
                                    Either::A(futures::failed(()))
                                }
                                Ok(connection) => Either::B(connection_handler.handle(connection)),
                            }));
                        continue 'toplevel;
                    }
                }
            }
            return Ok(Async::NotReady);
        }
    }
}
