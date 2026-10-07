use crate::future::{Future, PollState};
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    future,
    sync::{Arc, Mutex},
    thread::{self, Thread},
};

type Task = Box<dyn Future<Output = String>>;

// defining a thread local static variable
// Each thread will have their own instance
thread_local! {

    // This variable holds the executor thats currently running on this thread
    static  CURRENT_EXEC: ExecutorCore = ExecutorCore::default();
}

#[derive(Clone)]
pub struct Waker {
    // a handle to thread object
    thread: Thread,

    // identifies the task this waker is associated with
    id: usize,

    // usize repr the id of the task in the queue
    ready_queue: Arc<Mutex<Vec<usize>>>,
}

impl Waker {
    pub fn wake(&self) {
        self.ready_queue
            .lock()
            .map(|mut q| q.push(self.id))
            .unwrap();

        self.thread.unpark();
    }
}

// Holds the state of the executor
#[derive(Default)]
struct ExecutorCore {
    // Holds all the top level futures associated with this executor on this thread
    tasks: RefCell<HashMap<usize, Task>>,

    // Stores ids of tasks that should be polled by the executor
    // This will be given to each waker that this executor creates
    ready_queue: Arc<Mutex<Vec<usize>>>,

    // A counter that gives out the next available I, Which means that it should never hand out the
    // same id twice for this executor instance
    // We use this to give each top level future a unique id
    next_id: Cell<usize>,
}

pub fn spawn<F>(future: F)
where
    F: Future<Output = String> + 'static,
{
    CURRENT_EXEC.with(|e| {
        let id = e.next_id.get();
        e.tasks.borrow_mut().insert(id, Box::new(future));
        e.ready_queue.lock().map(|mut qu| qu.push(id)).unwrap();
        e.next_id.set(id + 1);
    });
}

pub struct Executor;

impl Executor {
    pub fn new() -> Self {
        Self {}
    }

    fn pop_ready(&self) -> Option<usize> {
        CURRENT_EXEC.with(|q| q.ready_queue.lock().map(|mut q| q.pop()).unwrap())
    }

    // Takes the id of a top level future, removes the future from the tasks collections and returns
    // it (if the task is found)
    fn get_future(&self, id: usize) -> Option<Task> {
        CURRENT_EXEC.with(|q| q.tasks.borrow_mut().remove(&id))
    }

    // Creates a new waker instance
    fn get_waker(&self, id: usize) -> Waker {
        Waker {
            id,
            thread: thread::current(),
            ready_queue: CURRENT_EXEC.with(|q| q.ready_queue.clone()),
        }
    }

    // Inserts a task into the tasks collections
    fn insert_task(&self, id: usize, task: Task) {
        CURRENT_EXEC.with(|q| q.tasks.borrow_mut().insert(id, task));
    }
    // Returns the count of how many tasks we have in the queue
    fn task_count(&self) -> usize {
        CURRENT_EXEC.with(|q| q.tasks.borrow().len())
    }

    // The entry point to the executor
    pub fn block_on<F>(&mut self, futures: F)
    where
        F: Future<Output = String> + 'static,
    {
        spawn(futures);

        loop {
            // Runs as long as their are tasks in the ready queue
            while let Some(id) = self.pop_ready() {
                // If theres is a task in the ready queue we take ownership of the future object by
                // removing it from the collection
                let mut future = match self.get_future(id) {
                    Some(f) => f,

                    // Guard against false wake ups
                    None => continue,
                };

                let waker = self.get_waker(id);

                match future.poll(&waker) {
                    // we insert the task back into our task collection if the future returns not ready
                    // The future will arrange so that so that Waker::wake is called at a later
                    // point in time
                    PollState::NotReady => self.insert_task(id, future),

                    // We continue to the next item in the queue
                    PollState::Ready(_) => continue,
                }
            }

            let task_count = self.task_count();
            let name = thread::current().name().unwrap_or_default().to_string();

            if task_count > 0 {
                println!("{name}: {task_count} pending tasks. Sleep until notified");
                // yield to the os scheduler
                thread::park();
            } else {
                println!("{name}: All tasks are finished");
                break;
            }
        }
    }
}
