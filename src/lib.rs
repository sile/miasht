#[macro_use]
extern crate log;
extern crate fibers;
extern crate futures;
extern crate httparse;

pub mod server;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
