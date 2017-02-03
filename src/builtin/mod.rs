use std::fmt;
use std::error;

pub use self::io::IoExt;
pub use self::futures::FutureExt;
pub use self::servers::SimpleHttpServer;

pub mod io;
pub mod router;
pub mod headers;
pub mod servers;
pub mod futures;

#[derive(Debug)]
pub struct NoError(());
impl fmt::Display for NoError {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        unreachable!()
    }
}
impl error::Error for NoError {
    fn description(&self) -> &str {
        unreachable!()
    }
    fn cause(&self) -> Option<&error::Error> {
        unreachable!()
    }
}
