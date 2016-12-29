extern crate clap;
extern crate env_logger;
extern crate fibers;
extern crate miasht;

use fibers::{Executor, InPlaceExecutor};
use miasht::server::HttpServer;

fn main() {
    let executor = InPlaceExecutor::new().unwrap();
    let addr = "0.0.0.0:3000".parse().unwrap();
    let server = HttpServer::new(addr, executor.handle());
    let _ = server.start_fn(|req, res| {
        println!("Hello World!: {} {:?}", req.version(), req.headers());
        Ok(res)
    });
    executor.run().unwrap();
}
