use std::future::Future;
use std::pin::Pin;
type BoxFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

pub struct Task {
    id: u32,
    future: BoxFuture
}

impl Task {
    /// creating a new task
    fn new(id: u32, future: BoxFuture) -> Task {
        Task { id, future }
    }
}