extern crate clap;
extern crate env_logger;
extern crate fibers;
extern crate futures;
extern crate handy_async;
extern crate miasht;

use std::io::Read;
use fibers::{Executor, InPlaceExecutor};
use miasht::server::HttpServer;
use handy_async::io::AsyncWrite;
use futures::Future;

fn main() {
    let executor = InPlaceExecutor::new().unwrap();
    let addr = "0.0.0.0:3000".parse().unwrap();
    let server = HttpServer::new(addr, executor.handle());
    let _ = server.start_fn(|mut req| {
        println!("Hello World!: {} {:?}", req.version(), req.headers());
        let mut body = Vec::new();
        req.read_to_end(&mut body).unwrap();
        println!("Body: {:?}", body);
        let res = req.response(200, "OK");
        let mut body = res.into_body();
        body.async_write_all(b"Hello").map(|(b, _)| b).map_err(|_| ())
    });
    executor.run().unwrap();
}
