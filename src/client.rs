use std::io::{Write, Error, Result};
use std::net::SocketAddr;
use futures::{Future, BoxFuture};
use fibers::net::TcpStream;

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
            socket: self.socket,
            pre_body_buf: self.buf,
            pre_body_offset: 0,
        }
    }
}

pub struct RequestBody {
    socket: TcpStream,
    pre_body_buf: Vec<u8>,
    pre_body_offset: usize,
}
impl RequestBody {
    pub fn finish(self) -> Response {
        panic!()
    }
}
impl Write for RequestBody {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        panic!()
    }
    fn flush(&mut self) -> Result<()> {
        panic!()
    }
}

pub struct Response {
}
