// adding new futures to the executor
use std::sync::mpsc::{Receiver,  SyncSender};
use crate::task::Task;
use std::sync::Arc;
use std::future::Future;
pub struct Spawner {
    pub task_sender: SyncSender<Arc<Task>>,
}

pub struct Executor {
    pub tasks: Receiver<Arc<Task>>
}

impl Executor {
    pub fn run(&self) {
        while let Ok(task) = self.tasks.recv() {
            println!("Got task executing...");
            
        }
    }
}

impl Spawner {
    
    pub fn new(&self, future: impl Future<Output = ()> + Send + 'static) {
        let task = Arc::new(Task::new(Box::pin(future)));
        self.task_sender.send(task).expect("Failed to send task");
    }
}

