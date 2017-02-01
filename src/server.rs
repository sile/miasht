use std::fmt;
use std::io::{Error, ErrorKind, Read, Write, Result};
use std::net::SocketAddr;
use std::sync::Arc;
use fibers::Spawn;
use fibers::net::{TcpListener, TcpStream};
use fibers::sync::oneshot::{Monitor, MonitorError};
use futures::{self, Future, IntoFuture, Poll, Stream, Async};
use futures::BoxFuture;
use futures::future::Either;
use handy_async::io::AsyncWrite;
use httparse;

use Method;
use {Header, Headers};
use headers::ContentLength;

pub struct HttpServerHandle {
    pub monitor: Monitor<(), Error>,
}
impl Future for HttpServerHandle {
    type Item = ();
    type Error = Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.monitor.poll().map_err(|e| match e {
            MonitorError::Aborted => {
                Error::new(ErrorKind::ConnectionAborted,
                           "The HTTP server exited unexpectedly")
            }
            MonitorError::Failed(e) => e,
        })
    }
}
// impl stop,get_opts,set_opts,etc

pub trait HandleRequest: Clone + Send + 'static {
    type Future: Future<Item = ResponseBody, Error = ()> + Send;
    fn handle_request(self, req: Request<'static>) -> Self::Future;
    fn handle_error(self, server: SocketAddr, client: SocketAddr, error: Error) {
        error!("HTTP connection between {:?}(server) and {:?}(client) is disconnected: {}",
               client,
               server,
               error);

    }
}
impl<F, G> HandleRequest for Arc<Box<F>>
    where F: Fn(Request<'static>) -> G + Sync + Send + 'static,
          G: IntoFuture<Item = ResponseBody, Error = ()>,
          G::Future: Send
{
    type Future = G::Future;
    fn handle_request(self, req: Request<'static>) -> Self::Future {
        self(req).into_future()
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
        where F: Fn(Request) -> G + Sync + Send + 'static,
              G: IntoFuture<Item = ResponseBody, Error = ()>, // TODO: Error = Error
              G::Future: Send
    {
        self.start(Arc::new(Box::new(f)))
    }

    pub fn start<H>(self, handler: H) -> HttpServerHandle
        where H: HandleRequest
    {
        let HttpServer { addr, spawner } = self;
        let monitor = spawner.clone().spawn_monitor(TcpListener::bind(addr).and_then(|listener| {
            // TODO: support keep alive
            let server_addr = listener.local_addr().unwrap();
            listener.incoming().for_each(move |(client, client_addr)| {
                let handler = handler.clone();
                spawner.spawn(client.and_then(|socket| ReadHeader::new(socket))
                    .then(move |result| match result {
                        Err(e) => {
                            handler.handle_error(server_addr, client_addr, e);
                            Either::A(futures::failed(()))
                        }
                        Ok(req) => {
                                Either::B(handler.handle_request(req)
                                    .and_then(|res| {
                                        res.stream.async_flush().map(|_| ()).map_err(|_| ())
                                    }))
                            }
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
    type Error = Error;
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
                    Err(e)
                }
            }
            Ok(0) => Err(Error::new(ErrorKind::UnexpectedEof, "TODO")),
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
                                return Err(e);
                            }
                            Ok(httparse::Status::Partial) => Err(false),
                            Ok(httparse::Status::Complete(_body_offset)) => {
                                panic!();
                                // let method = req.method.unwrap();
                                // let result =
                                //     (Method::try_from_str(req.method.unwrap()).ok_or_else(|| {
                                //         Error::UnknownMethod(method.to_string())
                                //          })?,
                                //      req.path.unwrap(),
                                //      req.version.unwrap(),
                                //      req.headers.len(),
                                //      body_offset);
                                // Ok(result)
                            }
                        }
                    };
                    match result {
                        Ok((method, path, version, header_count, body_offset)) => {
                            inner.headers.truncate(header_count);
                            let body =
                                RequestBodyStream::new(inner.reader,
                                                       inner.buf,
                                                       body_offset,
                                                       &Headers { headers: &inner.headers })?;
                            let req = Request {
                                method: method,
                                path: path,
                                version: version,
                                headers: inner.headers,
                                body: body,
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
pub struct RequestBodyStream<R: Read, B: AsRef<[u8]>> {
    stream: R,
    buf: B,
    offset: usize,
    remainings: u64,
}
impl<R: Read, B: AsRef<[u8]>> RequestBodyStream<R, B> {
    fn new(stream: R,
           buf: B,
           offset: usize,
           headers: &Headers)
           -> ::std::result::Result<Self, Error> {
        if headers.get_bytes("Transfer-Encoding").is_some() {
            unimplemented!()
        }
        let header = headers.get::<ContentLength>()?;
        Ok(RequestBodyStream {
            stream: stream,
            buf: buf,
            offset: offset,
            remainings: header.map(|h| h.len()).unwrap_or(0),
        })
    }
    fn unread_bytes(&self) -> Vec<u8> {
        Vec::from(&self.buf.as_ref()[self.offset..])
    }
}
impl<R: Read, B: AsRef<[u8]>> Read for RequestBodyStream<R, B> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        use std::cmp;
        let max_size = cmp::min(buf.len() as u64, self.remainings) as usize;
        if self.offset < self.buf.as_ref().len() {
            let size = cmp::min(max_size, self.buf.as_ref().len() - self.offset);
            buf[0..size].copy_from_slice(&self.buf.as_ref()[self.offset..self.offset + size]);
            self.offset += size;
            self.remainings -= size as u64;
            Ok(size)
        } else {
            let size = self.stream.read(&mut buf[0..max_size])?;
            self.remainings -= size as u64;
            Ok(size)
        }
    }
}

pub type ReadBodyBytes<'a> = BoxFuture<(Request<'a>, Vec<u8>), (Request<'a>, Error)>;
// pub struct ReadBodyBytes<'a>(Request<'a>);
// impl<'a> Future for ReadBodyBytes<'a> {
//     type Item = (Request<'a>, Vec<u8>);
//     type Error = (Request<'a>, Error);
//     fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
//         panic!()
//     }
// }

pub struct Request<'a> {
    method: Method,
    path: &'a str,
    version: u8,
    headers: Vec<httparse::Header<'a>>,
    body: RequestBodyStream<TcpStream, Vec<u8>>,
}
impl<'a> fmt::Debug for Request<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "Request {{ method: {:?}, path: {:?}, version: {:?}, headers: {{",
               self.method,
               self.path,
               self.version)?;
        let mut is_first = true;
        for h in self.headers.iter() {
            use std::str;
            if !is_first {
                write!(f, ", ")?;
            }
            is_first = false;
            if let Ok(value) = str::from_utf8(h.value) {
                write!(f, "{:?} => {:?}", h.name, value)?;
            } else {
                write!(f, "{:?} => {:?}", h.name, h.value)?;
            }
        }
        write!(f, "}}, body: _ }}")
    }
}
impl Request<'static> {
    pub fn read_body_bytes(self) -> ReadBodyBytes<'static> {
        use handy_async::io::ReadFrom;
        use handy_async::pattern::read::All;
        // TODO: Handle chunked-stream
        // TODO: Limit max size
        if let Some(length) = self.headers().get::<ContentLength>().unwrap() {
            let buf = vec![0; length.0 as usize];
            buf.read_from(self).map_err(|e| e.unwrap()).boxed()
        } else {
            All.read_from(self).map_err(|e| e.unwrap()).boxed()
        }
    }
}
impl<'a> Request<'a> {
    pub fn method(&self) -> &Method {
        &self.method
    }
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn version(&self) -> u8 {
        self.version
    }
    pub fn headers(&self) -> Headers {
        Headers { headers: &self.headers }
    }
    pub fn into_response(self, status_code: u16, status_reason: &'static str) -> Response {
        let unread = self.body.unread_bytes();
        let mut buf = Vec::with_capacity(1024);
        write!(buf,
               "HTTP/1.{} {} {}\r\n",
               self.version,
               status_code,
               status_reason)
            .unwrap();
        Response {
            pre_body_buf: buf,
            stream: self.body.stream,
            unread: unread,
        }
    }
}
impl<'a> Read for Request<'a> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.body.read(buf)
    }
}

pub struct Response {
    pre_body_buf: Vec<u8>,
    stream: TcpStream,
    unread: Vec<u8>,
}
impl Response {
    pub fn add_header<H: Header>(&mut self, header: &H) -> &mut Self {
        header.write(&mut self.pre_body_buf);
        self.pre_body_buf.extend_from_slice(b"\r\n");
        self
    }
    pub fn add_raw_header(&mut self, key: &str, value: &[u8]) {
        self.pre_body_buf.extend_from_slice(key.as_bytes());
        self.pre_body_buf.extend_from_slice(": ".as_bytes());
        self.pre_body_buf.extend_from_slice(value);
        self.pre_body_buf.extend_from_slice(b"\r\n");
    }
    // TODO: into_body_stream
    pub fn into_body(mut self) -> ResponseBody {
        self.pre_body_buf.extend_from_slice(b"\r\n");
        ResponseBody {
            pre_body_buf: self.pre_body_buf,
            pre_body_offset: 0,
            stream: self.stream,
            unread: self.unread,
        }
    }
    pub fn write_body_bytes<B: AsRef<[u8]>>(self, body: B) -> WriteBodyBytes<B> {
        use handy_async::io::AsyncWrite;
        WriteBodyBytes(self.into_body().async_write_all(body))
    }
}

use handy_async::io::futures::WriteAll;
pub struct WriteBodyBytes<B>(WriteAll<ResponseBody, B>);
impl<B: AsRef<[u8]>> Future for WriteBodyBytes<B> {
    type Item = ResponseBody;
    type Error = ();
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok(self.0.poll().map_err(|_| ())?.map(|(s, _)| s))
    }
}

#[allow(dead_code)]
pub struct ResponseBody {
    pre_body_buf: Vec<u8>,
    pre_body_offset: usize,
    stream: TcpStream,
    unread: Vec<u8>,
}
impl Write for ResponseBody {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        if self.pre_body_offset < self.pre_body_buf.len() {
            let size = self.stream.write(&self.pre_body_buf[self.pre_body_offset..])?;
            if size == 0 {
                Err(Error::new(ErrorKind::UnexpectedEof, "TODO"))
            } else {
                self.pre_body_offset += size;
                self.write(buf)
            }
        } else {
            self.stream.write(buf)
        }
    }
    fn flush(&mut self) -> Result<()> {
        if self.pre_body_offset < self.pre_body_buf.len() {
            let size = self.stream.write(&self.pre_body_buf[self.pre_body_offset..])?;
            if size == 0 {
                Err(Error::new(ErrorKind::UnexpectedEof, "TODO"))
            } else {
                self.pre_body_offset += size;
                self.flush()
            }
        } else {
            self.stream.flush()
        }
    }
}
