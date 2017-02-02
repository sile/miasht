extern crate clap;
extern crate fibers;
extern crate futures;
extern crate handy_async;
extern crate miasht;

use fibers::{Executor, ThreadPoolExecutor};
use futures::{Future, BoxFuture};
use handy_async::io::{AsyncRead, AsyncWrite};
use miasht::{Server, Status};
use miasht::builtin::servers::{SimpleHttpServer, RawConnection};
use miasht::builtin::headers::ContentLength;

fn main() {
    let mut executor = ThreadPoolExecutor::new().unwrap();
    let addr = "0.0.0.0:3000".parse().unwrap();
    let server = SimpleHttpServer::new((), echo);
    let server = server.start(addr, executor.handle());
    let monitor = executor.spawn_monitor(server.join());
    let result = executor.run_fiber(monitor).unwrap();
    println!("HTTP Server shutdown: {:?}", result);
}

fn echo(_: (), connection: RawConnection) -> BoxFuture<(), ()> {
    connection.read_request()
        .and_then(|request| {
            let body = request.into_body_reader();
            let buf = vec![0; 1024];
            body.async_read(buf).map_err(|e| e.into_error().into())
        })
        .and_then(|(body, mut buf, size)| {
            buf.truncate(size);
            let connection = body.finish();

            let mut resp = connection.response(Status::Ok);
            resp.add_header(&ContentLength(buf.len() as u64));
            resp.into_body_writer()
                .async_write_all(buf)
                .map_err(|e| e.into_error().into())
                .and_then(|(s, _)| s.finish())
                .then(|_| Ok(()))
        })
        .map_err(|e| {
            println!("Error: {:?}", e);
            ()
        })
        .boxed()
}
