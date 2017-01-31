use connection::Connection;

use Version;
use status::Status;

#[derive(Debug)]
pub struct Response<S> {
    connection: Connection<S>,
    version: Version,
    status: Status,
}
impl<S> Response<S> {
    pub fn new(connection: Connection<S>, version: Version, status: Status) -> Self {
        Response {
            connection: connection,
            version: version,
            status: status,
        }
    }
}

#[derive(Debug)]
pub struct ResponseBody;
