extern crate clap;
extern crate fibers;
extern crate futures;
extern crate miasht;

use fibers::{Executor, ThreadPoolExecutor, Spawn};
use futures::{Future, BoxFuture, IntoFuture};
use miasht::{Server, Status};
use miasht::builtin::servers::{SimpleHttpServer, RawConnection};
use miasht::builtin::headers::ContentLength;
use miasht::builtin::{IoExt, FutureExt};
use miasht::builtin::router::{Router, RouteBuilder};

type TcpRequest = miasht::server::Request<fibers::net::TcpStream>;

fn main() {
    let mut builder = RouteBuilder::new();
    builder.add_callback((), handle_put);
    builder.add_callback((), handle_default);
    let router = builder.finish();

    let mut executor = ThreadPoolExecutor::new().unwrap();
    let addr = "0.0.0.0:3000".parse().unwrap();
    let server = SimpleHttpServer::new(router, echo);
    let server = server.start(addr, executor.handle());
    let monitor = executor.spawn_monitor(server.join());
    let result = executor.run_fiber(monitor).unwrap();
    println!("HTTP Server shutdown: {:?}", result);
}

fn echo(router: Router<fibers::net::TcpStream>, connection: RawConnection) -> BoxFuture<(), ()> {
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
            .build_response(Status::MethodNotAllowed)
            .finish()
            .write_all_bytes("Please use PUT method\n")
            .then(|_| Ok(()))
            .boxed(),
    )
}

fn handle_put(_: (), request: TcpRequest) -> Result<BoxFuture<(), ()>, TcpRequest> {
    if miasht::Method::Put != request.method() {
        return Err(request);
    }
    Ok(
        request
            .into_body_reader()
            .into_future()
            .and_then(|r| r.read_all_bytes())
            .map_err(|e| {
                println!("Error: {:?}", e);
                ()
            })
            .and_then(|(request, buf)| {
                let connection = request.into_inner().finish();

                let mut response = connection.build_response(Status::Ok);
                response.add_header(&ContentLength(buf.len() as u64));
                response.finish().write_all_bytes(buf).then(|_| Ok(()))
            })
            .boxed(),
    )
}
