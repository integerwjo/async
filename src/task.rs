use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex;

type BoxFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

pub struct Task {
    pub future: Mutex<BoxFuture>,
}

impl Task {
    pub fn new(future: BoxFuture) -> Task {
        Task {
            future: Mutex::new(future),
        }
    }
}