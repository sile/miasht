extern crate clap;
extern crate fibers;
extern crate futures;
extern crate miasht;

use fibers::{Executor, ThreadPoolExecutor};
use futures::{Future, BoxFuture};
use miasht::{Server, Status};
use miasht::builtin::servers::{SimpleHttpServer, RawConnection};
use miasht::builtin::headers::ContentLength;
use miasht::builtin::io::BodyReader;
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
    connection.read_request()
        .and_then(|request| {
            futures::done(BodyReader::new(request)).and_then(|r| r.read_all_bytes())
        })
        .and_then(|(request, buf)| {
            let connection = request.into_inner().finish();

            let mut builder = connection.build_response(Status::Ok);
            builder.add_header(&ContentLength(buf.len() as u64));
            let response = builder.finish();
            response.write_all_bytes(buf).then(|_| Ok(()))
        })
        .map_err(|e| {
            println!("Error: {:?}", e);
            ()
        })
        .boxed()
}
