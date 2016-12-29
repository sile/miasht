use std::io::{Error, ErrorKind, Read};
use std::net::SocketAddr;
use std::sync::Arc;
use fibers::Spawn;
use fibers::net::{TcpListener, TcpStream};
use fibers::sync::oneshot::{Monitor, MonitorError};
use futures::{Future, IntoFuture, Poll, Stream, Async};
use httparse;

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
                spawner.spawn(client.and_then(|socket| ReadHeader::new(socket).map_err(|(_, e)| e))
                    .and_then(|req| {
                        let res = Response { socket: req.stream.clone() };
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

pub struct ReadHeaderInner<'a> {
    reader: TcpStream,
    buf: Vec<u8>,
    offset: usize,
    headers: Vec<httparse::Header<'a>>,
}

pub struct ReadHeader<'a>(Option<ReadHeaderInner<'a>>);

impl<'a> ReadHeader<'a> {
    fn new(reader: TcpStream) -> Self {
        ReadHeader(Some(ReadHeaderInner {
            reader: reader,
            buf: vec![0; 1024],
            offset: 0,
            headers: vec![httparse::EMPTY_HEADER; 8],
        }))
    }
}
impl<'a> Future for ReadHeader<'a> {
    type Item = Request<'a>;
    type Error = (TcpStream, Error);
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut inner = self.0.take().expect("Cannot poll ReadHeader twice");
        if inner.offset == inner.buf.len() {
            let new_len = inner.offset * 2; // TODO: max size
            inner.buf.resize(new_len, 0);
        }
        match inner.reader.read(&mut inner.buf[inner.offset..]) {
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock {
                    self.0 = Some(inner);
                    Ok(Async::NotReady)
                } else {
                    Err((inner.reader, e))
                }
            }
            Ok(read_size) => {
                inner.offset += read_size;
                loop {
                    let result = {
                        let buf = &inner.buf[0..inner.offset];
                        let buf = unsafe { &*(buf as *const _) as &'static _ };
                        let mut req = httparse::Request::new(&mut inner.headers);
                        match req.parse(buf) {
                            Err(httparse::Error::TooManyHeaders) => Err(true),
                            Err(e) => {
                                let e = Error::new(ErrorKind::InvalidData,
                                                   format!("HTTP parse failure: {}", e));
                                return Err((inner.reader, e));
                            }
                            Ok(httparse::Status::Partial) => Err(false),
                            Ok(httparse::Status::Complete(body_offset)) => {
                                let x = (req.method.unwrap().to_string(),
                                         req.path.unwrap().to_string(),
                                         req.version.unwrap());
                                Ok((body_offset, x, req.headers.len()))
                            }
                        }
                    };
                    match result {
                        Ok((body_offset, x, header_count)) => {
                            inner.headers.truncate(header_count);
                            let req = Request {
                                stream: inner.reader,
                                method: x.0,
                                path: x.1,
                                version: x.2,
                                headers: inner.headers,
                                buf: inner.buf,
                                body_offset: body_offset,
                            };
                            return Ok(Async::Ready(req));
                        }
                        Err(false) => {
                            self.0 = Some(inner);
                            return self.poll();
                        }
                        Err(true) => {
                            // retry
                            let new_len = inner.headers.len() * 2; // TODO: size limit
                            inner.headers.resize(new_len, httparse::EMPTY_HEADER);
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Request<'a> {
    stream: TcpStream,
    method: String, // TODO: &'a str or enum
    path: String, // TODO: &'a str
    version: u8,
    headers: Vec<httparse::Header<'a>>,
    buf: Vec<u8>,
    body_offset: usize,
}
impl<'a> Request<'a> {
    pub fn method(&self) -> &str {
        &self.method
    }
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn version(&self) -> u8 {
        self.version
    }
}
pub struct Response {
    socket: TcpStream,
}
