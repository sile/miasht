use futures::Future;

use {Status, TransportStream};
use server::Request;

pub type Callback<A, T> = fn(A, Request<T>) -> Result<BoxFuture<(), ()>, Request<T>>;

pub type BoxFuture<T, E> = Box<Future<Item = T, Error = E> + Send + 'static>;

pub struct RouteBuilder<T> {
    handlers: Vec<Box<HandleRequest<T> + Sync>>,
}
impl<T> RouteBuilder<T>
where
    T: TransportStream + 'static,
{
    pub fn new() -> Self {
        RouteBuilder {
            handlers: Vec::new(),
        }
    }
    pub fn add_handler<H>(&mut self, handler: H)
    where
        H: HandleRequest<T> + Sync + 'static,
    {
        self.handlers.push(Box::new(handler));
    }
    pub fn add_callback<A>(&mut self, argument: A, callback: Callback<A, T>)
    where
        A: Clone + Sync + 'static,
    {
        self.add_handler(RequestHandleCallback(argument, callback))
    }
    pub fn finish(self) -> Router<T> {
        Router {
            handlers: Arc::new(self.handlers),
        }
    }
}

use std::sync::Arc;
pub struct Router<T> {
    handlers: Arc<Vec<Box<HandleRequest<T> + Sync>>>,
}
unsafe impl<T> Send for Router<T> {}
impl<T> Clone for Router<T> {
    fn clone(&self) -> Self {
        Router {
            handlers: self.handlers.clone(),
        }
    }
}
impl<T> Router<T>
where
    T: TransportStream + Send + 'static,
{
    pub fn handle_request(&self, mut request: Request<T>) -> BoxFuture<(), ()> {
        for handler in self.handlers.iter() {
            match handler.try_handle(request) {
                Ok(future) => return future,
                Err(req) => request = req,
            }
        }
        let future = request
            .finish()
            .build_response(Status::NotFound)
            .finish()
            .then(|_| Ok(()));
        Box::new(future)
    }
}

pub trait HandleRequest<T> {
    fn try_handle(&self, request: Request<T>) -> Result<BoxFuture<(), ()>, Request<T>>;
}

pub struct RequestHandleCallback<A, T>(A, Callback<A, T>);
impl<A, T> HandleRequest<T> for RequestHandleCallback<A, T>
where
    A: Clone + Sync + 'static,
{
    fn try_handle(&self, request: Request<T>) -> Result<BoxFuture<(), ()>, Request<T>> {
        let argument = self.0.clone();
        (self.1)(argument, request)
    }
}
