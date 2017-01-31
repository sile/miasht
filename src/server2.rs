use std::io::{Read, Write};
use std::net::SocketAddr;
use fibers::Spawn;
use fibers::net::{TcpListener, TcpStream};
use fibers::sync::{mpsc, oneshot};
use futures::{Future, Stream, Poll, Async};

use Result;
use error::Error;
use request::Request;
use connection::Connection;

pub trait Server<T>
    where T: Read + Write
{
    type Handler: ClientHandler<T>;
    type Command: Send;
    fn before_listen(&mut self, listener: &mut TcpListener) -> Result<()> {
        Ok(())
    }
    fn serve_client(&mut self, client: SocketAddr) -> Result<Self::Handler>;
    fn serve_command(&mut self, command: Self::Command) -> Option<Result<()>>;
}

pub trait ClientHandler<T>: Sized + Send
    where T: Read + Write
{
    type Future: Future<Item = (Self, Connection<T>), Error = Connection<T>> + Send;
    // fn handle(self, connection: Connection<T>) -> { connection.read_request() }
    fn handle_request(self, request: Request<T>) -> Self::Future;
    fn handle_error(self, error: Error);
}

pub struct HttpServer<S>
    where S: Server<TcpStream>
{
    monitor: oneshot::Monitor<(), Error>,
    command_tx: mpsc::Sender<S::Command>,
}
impl<S> HttpServer<S>
    where S: Server<TcpStream>
{
    pub fn start<T>(bind_addr: SocketAddr, server: S, spawner: T) -> Self
        where T: Spawn + Clone + Send + 'static
    {
        panic!()
    }
    pub fn issue_command(&self, command: S::Command) -> Result<()> {
        self.command_tx.send(command).map_err(|_| Error::ServerDown)
    }
    // pub fn try_join(mut self) -> Result<Self, oneshot::MonitorError<S::Error>> {
    //     match self.monitor.poll() {
    //         Err(e) => Err(e),
    //         Ok(Async::NotReady) => Ok(self),
    //         Ok(Async::Ready(())) => unreachable!(),
    //     }
    // }
    pub fn join(self) -> JoinHandle {
        JoinHandle { monitor: self.monitor }
    }
}

pub struct JoinHandle {
    monitor: oneshot::Monitor<(), Error>,
}
impl Future for JoinHandle {
    type Item = ();
    type Error = Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.monitor.poll()
    }
}
