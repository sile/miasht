use std::net::SocketAddr;
use fibers::net::{self, TcpStream};
use futures::{Future, Poll};

use connection2::{Connection, ByteBuffer, HeaderBuffer};
use error::Error;

//
pub struct Client {}
impl Client {
    pub fn connect(server: SocketAddr) -> Connect {
        Connect(TcpStream::connect(server))
    }
}

pub struct Connect(net::futures::Connect);
impl Future for Connect {
    type Item = Connection<TcpStream>;
    type Error = Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok(self.0.poll().map_err(Error::Io)?.map(|socket| {
            Connection::new(socket, ByteBuffer::new(1024, 1024), HeaderBuffer::new(32))
        }))
    }
}
