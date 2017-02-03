extern crate clap;
extern crate fibers;
extern crate futures;
extern crate miasht;

use clap::{App, Arg};
use fibers::{Executor, InPlaceExecutor};
use futures::Future;
use miasht::{Client, Method};
use miasht::builtin::io::BodyReader;
use miasht::builtin::FutureExt;

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
            futures::done(BodyReader::new(res))
                .and_then(|r| r.read_all_str())
                .map(|(_, body)| body)
        }));
    match executor.run_fiber(monitor).unwrap() {
        Ok(s) => println!("{}", s),
        Err(e) => println!("[ERROR] {:?}", e),
    }
}
