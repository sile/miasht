use std::fmt;
use std::error;

pub mod headers;
pub mod servers;
pub mod io;

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
