
//! The executor is responsible for managing and executing tasks. 
//! It maintains a queue of tasks that are ready to run and executes them in a loop until there are no more tasks to run.

use std::collections::VecDeque;

use crate::task::Task;

pub struct Executor {
    tasks: VecDeque<Task>,

}

impl Executor{
    fn new() -> Self {
        Executor {
            tasks: VecDeque::new(),
        }
    }
}