use {Version, Method};
use header::Headers;
use status::RawStatus;

pub trait Metadata {
    fn version(&self) -> Version;
    fn headers(&self) -> &Headers;
    fn status(&self) -> Option<&RawStatus>;
    fn method(&self) -> Option<Method>;
    fn is_request(&self) -> bool {
        self.method().is_some()
    }
    fn is_response(&self) -> bool {
        self.status().is_some()
    }
}
