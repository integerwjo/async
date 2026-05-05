// adding new futures to the executor
use std::sync::mpsc::{Receiver,  SyncSender};
use crate::task::Task;
use std::sync::Arc;
use std::future::Future;

/// spawner sends tasks to the executor
pub struct Spawner {
    pub task_sender: SyncSender<Arc<Task>>,
}

impl Spawner {
    /// creating a new instance of spawner
    /// A s
    pub fn new(&self, future: impl Future<Output = ()> + Send + 'static) {
        let task = Arc::new(Task::new(Box::pin(future)));
        self.task_sender.send(task).expect("Failed to send task");
    }
}