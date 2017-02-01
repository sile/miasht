use std::net::SocketAddr;
use fibers::Spawn;
use fibers::net::{TcpListener, TcpStream};
use fibers::net::streams::Incoming;
use fibers::sync::{mpsc, oneshot};
use futures::{self, Future, Stream, Poll, Async};
use futures::future::Either;

use Result;
use error::Error;
use connection::{self, Connection};

pub enum Command {
    Stop,
}

pub trait Server {
    type Handler: ClientHandler;

    #[allow(unused_variables)]
    fn before_listen(&mut self, listener: &mut TcpListener) -> Result<()> {
        Ok(())
    }
    fn create_buffer(&mut self) -> connection::Buffer {
        connection::Buffer::new()
    }
    fn create_handler(&mut self, client: SocketAddr) -> Self::Handler;

    fn start<S>(self, bind_addr: SocketAddr, spawner: S) -> HttpServer
        where Self: Sized + Send + 'static,
              S: Spawn + Clone + Send + 'static
    {
        HttpServer::start(self, bind_addr, spawner)
    }
}

pub struct SimpleServer<T, F> {
    state: T,
    callback: fn(T, Connection<TcpStream>) -> F,
}
impl<T, F> SimpleServer<T, F>
    where T: Clone + Send + 'static,
          F: futures::IntoFuture<Item = (), Error = ()> + 'static,
          F::Future: Send + 'static
{
    pub fn new(state: T, callback: fn(T, Connection<TcpStream>) -> F) -> Self {
        SimpleServer {
            state: state,
            callback: callback,
        }
    }
}
impl<T, F> Server for SimpleServer<T, F>
    where T: Clone + Send + 'static,
          F: futures::IntoFuture<Item = (), Error = ()> + 'static,
          F::Future: Send + 'static
{
    type Handler = SimplClientHandler<T, F>;
    fn create_handler(&mut self, _client: SocketAddr) -> Self::Handler {
        SimplClientHandler {
            state: self.state.clone(),
            callback: self.callback,
        }
    }
}

pub struct SimplClientHandler<T, F> {
    state: T,
    callback: fn(T, Connection<TcpStream>) -> F,
}
impl<T, F> ClientHandler for SimplClientHandler<T, F>
    where T: Clone + Send + 'static,
          F: futures::IntoFuture<Item = (), Error = ()> + 'static,
          F::Future: Send + 'static
{
    type Future = F::Future;
    fn handle(self, connection: Connection<TcpStream>) -> Self::Future {
        (self.callback)(self.state, connection).into_future()
    }
}


pub trait ClientHandler: Sized + Send + 'static {
    type Future: Future<Item = (), Error = ()> + Send + 'static;
    fn handle(self, connection: Connection<TcpStream>) -> Self::Future;
    #[allow(unused_variables)]
    fn on_error(self, error: Error) {}
}

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
                        let handler = self.server.create_handler(address);
                        let buffer = self.server.create_buffer();
                        self.spawner.spawn(socket.then(move |result| match result {
                            Err(e) => {
                                handler.on_error(Error::Io(e));
                                Either::A(futures::failed(()))
                            }
                            Ok(socket) => {
                                let connection = Connection::with_buffer(socket, buffer);
                                Either::B(handler.handle(connection))
                            }
                        }));
                        continue 'toplevel;
                    }
                }
            }
            return Ok(Async::NotReady);
        }
    }
}

pub struct HttpServer {
    command_tx: mpsc::Sender<Command>,
    monitor: oneshot::Monitor<(), Error>,
}
impl HttpServer {
    pub fn start<S, T>(mut server: S, bind_addr: SocketAddr, spawner: T) -> Self
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
        HttpServer {
            monitor: monitor,
            command_tx: command_tx,
        }
    }
    pub fn stop(self) -> JoinHandle {
        let _ = self.command_tx.send(Command::Stop);
        JoinHandle(self)
    }
    pub fn join(self) -> JoinHandle {
        JoinHandle(self)
    }
}

pub struct JoinHandle(HttpServer);
impl Future for JoinHandle {
    type Item = ();
    type Error = Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.0.monitor.poll().map_err(|e| e.unwrap_or(Error::ServerAborted))
    }
}
