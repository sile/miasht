extern crate clap;
extern crate env_logger;
extern crate fibers;
extern crate futures;
extern crate handy_async;
extern crate miasht;

use std::io::Read;
use fibers::{Executor, ThreadPoolExecutor};
use handy_async::io::AsyncWrite;
use futures::Future;
use miasht::server::HttpServer;
use miasht::headers::ContentLength;

fn main() {
    let executor = ThreadPoolExecutor::new().unwrap();
    let addr = "0.0.0.0:3000".parse().unwrap();
    let server = HttpServer::new(addr, executor.handle());
    let _ = server.start_fn(|mut req| {
        let mut req_body = Vec::new();
        req.read_to_end(&mut req_body).unwrap();

        let mut res = req.into_response(200, "OK");
        res.add_header(&ContentLength(req_body.len() as u64));
        let res_body = res.into_body();
        res_body.async_write_all(req_body).map(|(b, _)| b).map_err(|_| ())
    });
    executor.run().unwrap();
}
