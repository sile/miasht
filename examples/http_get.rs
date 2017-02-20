extern crate clap;
extern crate fibers;
extern crate futures;
extern crate miasht;

use clap::{App, Arg};
use fibers::{Executor, InPlaceExecutor, Spawn};
use futures::{Future, IntoFuture};
use miasht::{Client, Method};
use miasht::builtin::{IoExt, FutureExt};

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
            res.into_body_reader()
                .into_future()
                .and_then(|r| r.read_all_str())
                .map(|(_, body)| body)
        }));
    match executor.run_fiber(monitor).unwrap() {
        Ok(s) => println!("{}", s),
        Err(e) => println!("[ERROR] {:?}", e),
    }
}
