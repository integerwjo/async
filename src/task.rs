use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex;
type BoxFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;



pub struct Task {
    future: Option<Mutex<BoxFuture>>
}

impl Task {
    /// creating a new task
    pub fn new(future: BoxFuture) -> Task {
        Task { future: Some(Mutex::new(future)) }
    }
}