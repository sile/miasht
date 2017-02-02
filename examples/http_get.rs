extern crate clap;
extern crate fibers;
extern crate futures;
extern crate miasht;
extern crate handy_async;

use clap::{App, Arg};
use fibers::{Executor, InPlaceExecutor};
use futures::Future;
use miasht::{Client, Method};
use handy_async::io::ReadFrom;
use handy_async::pattern::read::{All, Utf8};

fn main() {
    let matches = App::new("http_get")
        .arg(Arg::with_name("HOST").index(1).required(true))
        .arg(Arg::with_name("PATH").index(2).required(true))
        .arg(Arg::with_name("PORT").short("p").takes_value(true).default_value("80"))
        .get_matches();
    let host = matches.value_of("HOST").unwrap();
    let path = matches.value_of("PATH").unwrap().to_string();
    let port = matches.value_of("PORT").unwrap();
    let addr = format!("{}:{}", host, port).parse().expect("Invalid address");

    let mut executor = InPlaceExecutor::new().unwrap();
    let monitor = executor.spawn_monitor(Client::new()
        .connect(addr)
        .and_then(move |connection| connection.build_request(Method::Get, &path).finish())
        .and_then(|req| req.read_response())
        .and_then(|res| {
            Utf8(All)
                .read_from(res)
                .map(|(_, body)| body)
                .map_err(|e| e.into_error().into())
        }));
    match executor.run_fiber(monitor).unwrap() {
        Ok(s) => println!("{}", s),
        Err(e) => println!("[ERROR] {:?}", e),
    }
}
