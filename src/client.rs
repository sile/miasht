use std::io::{Write, Error, Result, ErrorKind, Read};
use std::net::SocketAddr;
use futures::{Future, BoxFuture, Poll};
use fibers::net::TcpStream;
use handy_async::io::AsyncWrite;
use handy_async::io::futures::Flush;

use {Method, Version, Header};

pub struct HttpClient {
    socket: TcpStream,
}
impl HttpClient {
    pub fn connect(server_addr: SocketAddr) -> BoxFuture<Self, Error> {
        TcpStream::connect(server_addr)
            .map(|socket| HttpClient { socket: socket })
            .boxed()
    }

    pub fn request(self, method: Method, path: &str, version: Version) -> Request {
        let mut buf = Vec::with_capacity(1024);
        write!(buf, "{} {} {}\r\n", method, path, version).unwrap();
        Request {
            socket: self.socket,
            buf: buf,
        }
    }
}

pub struct Request {
    socket: TcpStream,
    buf: Vec<u8>,
}
impl Request {
    pub fn add_header<H: Header>(&mut self, header: H) -> &mut Self {
        header.write(&mut self.buf);
        self.buf.extend_from_slice(b"\r\n");
        self
    }
    pub fn into_body(mut self) -> RequestBody {
        self.buf.extend_from_slice(b"\r\n");
        RequestBody {
            stream: self.socket,
            pre_body_buf: self.buf,
            pre_body_offset: 0,
        }
    }
    pub fn finish(self) -> WaitResponse {
        self.into_body().finish()
    }
}

// TODO: 共通化
pub struct RequestBody {
    stream: TcpStream,
    pre_body_buf: Vec<u8>,
    pre_body_offset: usize,
}
impl RequestBody {
    pub fn finish(self) -> WaitResponse {
        WaitResponse(self.async_flush())
    }
}
impl Write for RequestBody {
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

pub struct WaitResponse(Flush<RequestBody>);
impl Future for WaitResponse {
    type Item = Response;
    type Error = Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        // TODO: read response header
        Ok(self.0.poll().map_err(|e| e.into_error())?.map(|r| Response { stream: r.stream }))
    }
}

#[derive(Debug)]
pub struct Response {
    stream: TcpStream,
}
impl Read for Response {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.stream.read(buf)
    }
}
