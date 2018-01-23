extern crate clap;
extern crate fibers;
extern crate futures;
extern crate miasht;

use fibers::{Executor, Spawn, ThreadPoolExecutor};
use futures::{BoxFuture, Future};
use miasht::{Server, Status};
use miasht::builtin::servers::{RawConnection, SimpleHttpServer};
use miasht::builtin::headers::ContentLength;
use miasht::builtin::FutureExt;

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
    connection
        .read_request()
        .and_then(|request| {
            let bytes = b"Hello, World";
            let connection = request.finish();
            let mut response = connection.build_response(Status::Ok);
            response.add_header(&ContentLength(bytes.len() as u64));
            response.finish().write_all_bytes(bytes).then(|_| Ok(()))
        })
        .map_err(|e| {
            println!("Error: {:?}", e);
            ()
        })
        .boxed()
}
