extern crate clap;
extern crate fibers;
extern crate futures;
extern crate handy_async;
extern crate miasht;

use fibers::{Executor, Spawn, ThreadPoolExecutor};
use fibers::net::TcpStream;
use futures::{Async, Future, Poll};
use handy_async::future::Phase;
use miasht::{Server, Status};
use miasht::server::{ReadRequest, Response};
use miasht::builtin::servers::{RawConnection, SimpleHttpServer};
use miasht::builtin::futures::WriteAllBytes;
use miasht::builtin::headers::ContentLength;
use miasht::builtin::FutureExt;

fn main() {
    let mut executor = ThreadPoolExecutor::new().unwrap();
    let addr = "0.0.0.0:3000".parse().unwrap();
    let server = SimpleHttpServer::new((), hello);
    let server = server.start(addr, executor.handle());
    let monitor = executor.spawn_monitor(server.join());
    let result = executor.run_fiber(monitor).unwrap();
    println!("HTTP Server shutdown: {:?}", result);
}

fn hello(_: (), connection: RawConnection) -> Box<Future<Item = (), Error = ()> + Send + 'static> {
    let phase = Phase::A(connection.read_request());
    Box::new(Hello { phase })
}

struct Hello {
    phase: Phase<
        ReadRequest<TcpStream>,
        WriteAllBytes<Response<TcpStream>, &'static [u8; 12]>,
        Response<TcpStream>,
    >,
}
impl Future for Hello {
    type Item = ();
    type Error = ();
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Async::Ready(phase) = self.phase.poll().map_err(|_| ())? {
            let next = match phase {
                Phase::A(request) => {
                    let bytes = b"Hello, World";
                    let connection = request.finish();
                    let mut response = connection.build_response(Status::Ok);
                    response.add_header(&ContentLength(bytes.len() as u64));
                    Phase::B(response.finish().write_all_bytes(bytes))
                }
                Phase::B(response) => Phase::C(response),
                Phase::C(connection) => Phase::A(connection.read_request()),
                _ => unreachable!(),
            };
            self.phase = next;
        }
        Ok(Async::NotReady)
    }
}
