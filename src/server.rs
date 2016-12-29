use std::io::{Error, ErrorKind, Read};
use std::net::SocketAddr;
use std::sync::Arc;
use fibers::Spawn;
use fibers::net::{TcpListener, TcpStream};
use fibers::sync::oneshot::{Monitor, MonitorError};
use futures::{Future, IntoFuture, Poll, Stream};

pub struct HttpServerHandle {
    monitor: Monitor<(), Error>,
}
impl Future for HttpServerHandle {
    type Item = ();
    type Error = Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.monitor.poll().map_err(|e| {
            match e {
                MonitorError::Aborted => {
                    Error::new(ErrorKind::ConnectionAborted,
                               "The HTTP server exited unexpectedly")
                }
                MonitorError::Failed(e) => e,
            }
        })
    }
}
// impl stop,get_opts,set_opts,etc

pub trait HandleRequest: Clone + Send + 'static {
    type Future: Future<Item = Response, Error = Error> + Send;
    fn handle_request(self, req: Request, res: Response) -> Self::Future;
    fn handle_error(self, client: TcpStream, error: Error) {
        error!("HTTP connection between {:?}(server) and {:?}(client) is disconnected: {}",
               client.peer_addr(),
               client.local_addr(),
               error);

    }
}
impl<F, G> HandleRequest for Arc<Box<F>>
    where F: Fn(Request, Response) -> G + Sync + Send + 'static,
          G: IntoFuture<Item = Response, Error = Error>,
          G::Future: Send
{
    type Future = G::Future;
    fn handle_request(self, req: Request, res: Response) -> Self::Future {
        self(req, res).into_future()
    }
}

pub struct HttpServer<S> {
    addr: SocketAddr,
    spawner: S,
}
impl<S> HttpServer<S>
    where S: Spawn + Clone + Send + 'static
{
    pub fn new(addr: SocketAddr, spawner: S) -> Self {
        HttpServer {
            addr: addr,
            spawner: spawner,
        }
    }
    pub fn start_fn<F, G>(self, f: F) -> HttpServerHandle
        where F: Fn(Request, Response) -> G + Sync + Send + 'static,
              G: IntoFuture<Item = Response, Error = Error>,
              G::Future: Send
    {
        self.start(Arc::new(Box::new(f)))
    }

    pub fn start<H>(self, handler: H) -> HttpServerHandle
        where H: HandleRequest
    {
        let HttpServer { addr, spawner } = self;
        let monitor = spawner.clone().spawn_monitor(TcpListener::bind(addr).and_then(|listener| {
            // TODO: Handle requests from HttpServerHandle
            // TODO: support keep alive
            listener.incoming().for_each(move |(client, _)| {
                let handler = handler.clone();
                spawner.spawn(client.and_then(|socket| ReadHeader(socket).map_err(|(_, e)| e))
                    .and_then(|(socket, header, buf)| {
                        let req = Request {
                            socket: socket.clone(),
                            header: header,
                            buf: buf,
                        };
                        let res = Response { socket: socket };
                        handler.handle_request(req, res)
                    })
                    .then(|_r| {
                        // TODO: invoke handle_error if needed
                        Ok(())
                    }));
                Ok(())
            })
        }));
        HttpServerHandle { monitor: monitor }
    }
}

pub struct ReadHeader<R>(R);
impl<R: Read> Future for ReadHeader<R> {
    type Item = (R, Header, Vec<u8>);
    type Error = (R, Error);
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        panic!()
    }
}
pub struct Header;

pub struct Request {
    socket: TcpStream,
    header: Header,
    buf: Vec<u8>,
}
pub struct Response {
    socket: TcpStream,
}
