
//! The executor is responsible for managing and executing tasks. 
//! It maintains a queue of tasks that are ready to run and executes them in a loop until there are no more tasks to run.

use std::sync::mpsc::{Receiver,  SyncSender};
use super::task::Task;
use std::sync::Arc;

pub struct Executor {
    pub tasks: Receiver<Arc<Task>>

}




impl Executor{
    pub fn run(&self) {
        while let Ok(task) = self.tasks.recv() {
            println!("Executor: Running task");
            // Here we would poll the task's future and execute it if it's ready.
            // If the task is not ready, we would put it back in the queue to be polled again later.
        }
    }   
}