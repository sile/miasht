// extern crate clap;
// extern crate env_logger;
// extern crate fibers;
// extern crate futures;
// extern crate handy_async;
// extern crate miasht;

// use fibers::{Executor, ThreadPoolExecutor};
// use futures::{Future, BoxFuture};
// use miasht::server2::{Server, SimpleServer};
// use miasht::connection::TcpConnection;
// use miasht::headers::ContentLength;
// use miasht::Status;
// use handy_async::io::{AsyncRead, AsyncWrite};

fn main() {
    // let mut executor = ThreadPoolExecutor::new().unwrap();
    // let addr = "0.0.0.0:3000".parse().unwrap();
    // let server = SimpleServer::new((), echo);
    // let server = server.start(addr, executor.handle());
    // let monitor = executor.spawn_monitor(server.join());
    // let result = executor.run_fiber(monitor).unwrap();
    // println!("HTTP Server shutdown: {:?}", result);
}

// // See also: https://github.com/rust-lang/rfcs/pull/1558
// fn echo(():(), connection: TcpConnection) -> BoxFuture<(), ()> {
//     connection.read_request()
//         .and_then(|request| {
//             let body = request.into_body();
//             let buf = vec![0; 1024];
//             body.async_read(buf).map_err(|e| miasht::error::Error::Io(e.into_error()))
//         })
//         .and_then(|(body, mut buf, size)| {
//             buf.truncate(size);
//             let mut resp = body.into_response(Status::Ok);
//             resp.add_header(&ContentLength(buf.len() as u64));
//             resp.into_body()
//                 .async_write_all(buf)
//                 .map_err(|e| e.map_state(|(s, _)| s))
//                 .and_then(|(s, _)| s.async_flush())
//                 .then(|_| Ok(()))
//         })
//         .map_err(|e| {
//             println!("Error: {:?}", e);
//             ()
//         })
//         .boxed()
// }
