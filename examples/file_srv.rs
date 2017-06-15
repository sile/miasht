extern crate clap;
extern crate fibers;
extern crate futures;
extern crate miasht;
extern crate handy_async;

use std::fs;
use fibers::{Executor, ThreadPoolExecutor, Spawn};
use futures::{Future, BoxFuture};
use miasht::{Server, Status};
use miasht::builtin::servers::{SimpleHttpServer, RawConnection};
use miasht::builtin::headers::ContentLength;
use miasht::builtin::FutureExt;
use miasht::builtin::router::{Router, RouteBuilder};
use handy_async::sync_io::ReadExt;

type TcpRequest = miasht::server::Request<fibers::net::TcpStream>;

fn main() {
    let mut builder = RouteBuilder::new();
    builder.add_callback((), handle_get);
    builder.add_callback((), handle_default);
    let router = builder.finish();

    let mut executor = ThreadPoolExecutor::new().unwrap();
    let addr = "0.0.0.0:3000".parse().unwrap();
    let server = SimpleHttpServer::new(router, route);
    let server = server.start(addr, executor.handle());
    let monitor = executor.spawn_monitor(server.join());
    let result = executor.run_fiber(monitor).unwrap();
    println!("HTTP Server shutdown: {:?}", result);
}

fn route(router: Router<fibers::net::TcpStream>, connection: RawConnection) -> BoxFuture<(), ()> {
    connection
        .read_request()
        .map_err(|e| {
            println!("Error: {:?}", e);
            ()
        })
        .and_then(move |request| router.handle_request(request))
        .boxed()
}

fn handle_default(_: (), request: TcpRequest) -> Result<BoxFuture<(), ()>, TcpRequest> {
    Ok(
        request
            .finish()
            .build_response(Status::NotFound)
            .finish()
            .write_all_bytes("Not Found\n")
            .then(|_| Ok(()))
            .boxed(),
    )
}

fn handle_get(_: (), request: TcpRequest) -> Result<BoxFuture<(), ()>, TcpRequest> {
    if miasht::Method::Get != request.method() {
        return Err(request);
    }
    println!("# GET: {}", &request.path()[1..]);
    Ok(match fs::File::open(&request.path()[1..]).and_then(
        |mut f| {
            ReadExt::read_all_bytes(&mut f)
        },
    ) {
        Err(e) => {
            let reason = e.to_string();
            let mut resp = request.finish().build_response(Status::NotFound);
            resp.add_header(&ContentLength(reason.len() as u64));
            resp.finish()
                .write_all_bytes(reason)
                .then(|_| Ok(()))
                .boxed()
        }
        Ok(bytes) => {
            let mut resp = request.finish().build_response(Status::Ok);
            resp.add_header(&ContentLength(bytes.len() as u64));
            resp.finish()
                .write_all_bytes(bytes)
                .then(|_| Ok(()))
                .boxed()
        }
    })
}
