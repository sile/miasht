extern crate clap;
extern crate fibers;
extern crate futures;
extern crate miasht;
extern crate sha1;
extern crate base64;
extern crate handy_async;
#[macro_use]
extern crate trackable;

use fibers::{Executor, ThreadPoolExecutor, Spawn};
use futures::{Future, BoxFuture, IntoFuture};
use miasht::{Server, Status, Method, Error};
use miasht::builtin::servers::{SimpleHttpServer, RawConnection};
use miasht::builtin::headers::ContentLength;
use miasht::builtin::{IoExt, FutureExt};
use miasht::builtin::router::{Router, RouteBuilder};
use handy_async::io::AsyncRead;

type TcpRequest = miasht::server::Request<fibers::net::TcpStream>;

fn main() {
    let mut builder = RouteBuilder::new();
    builder.add_callback((), handle_get_file);
    builder.add_callback((), handle_put);
    builder.add_callback((), handle_upgrade);
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

fn handle_get_file(_: (), request: TcpRequest) -> Result<BoxFuture<(), ()>, TcpRequest> {
    if request.method() == Method::Get && request.path().starts_with("/files/") {
        use std::io::Read;
        let mut buf = Vec::new();
        {
            let path = &request.path()[7..];
            println!("# GET: file={}", path);
            let mut f = std::fs::File::open(path).expect("Cannot open file");
            f.read_to_end(&mut buf).expect("Cannot read file");
            println!(" => {} bytes", buf.len());
        }
        let connection = request.finish();
        let mut response = connection.build_response(Status::Ok);
        response.add_header(&ContentLength(buf.len() as u64));
        Ok(
            response
                .finish()
                .write_all_bytes(buf)
                .then(|_| Ok(()))
                .boxed(),
        )
    } else {
        Err(request)
    }
}

fn handle_upgrade(_: (), request: TcpRequest) -> Result<BoxFuture<(), ()>, TcpRequest> {
    if request.method() != Method::Get {
        return Err(request);
    }
    if request.headers().get("Upgrade") != Some(b"websocket") {
        return Err(request);
    }
    let key = std::str::from_utf8(request.headers().get("Sec-WebSocket-Key").expect(
        "No 'Sec-WebSocket-Key'",
    )).unwrap()
        .to_string();
    println!("# WebSocket: key={}", key);

    let connection = request.finish();
    let mut response = connection.build_response(Status::SwitchingProtocols);
    response.add_raw_header("Upgrade", b"websocket");
    response.add_raw_header("Connection", b"upgrade");

    let accept_key = format!("{}258EAFA5-E914-47DA-95CA-C5AB0DC85B11", key);
    let mut m = sha1::Sha1::new();
    m.update(accept_key.as_bytes());
    let accept_key = base64::encode(&m.digest().bytes()[..]);
    response.add_raw_header("Sec-WebSocket-Accept", accept_key.as_bytes());
    //response.add_header(&ContentLength(buf.len() as u64));
    Ok(
        response
            .finish()
            .and_then(|conn| {
                let stream = conn.into_raw_stream();
                let buf = vec![0; 128];
                let future = stream.async_read(buf).map(|(_, buf, size)| {
                    println!("# BUF: {:?}", &buf[..size])
                });
                future.map_err(|e| track!(Error::from(e)))
            })
            .then(|_| Ok(()))
            .boxed(),
    )
}
fn handle_default(_: (), request: TcpRequest) -> Result<BoxFuture<(), ()>, TcpRequest> {
    println!(
        "# headers: {:?}",
        request
            .headers()
            .iter()
            .map(|(k, v)| (k, std::str::from_utf8(v)))
            .collect::<Vec<_>>()
    );
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
