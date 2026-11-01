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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::future::{Future, PollState};
    use std::sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    };

    struct ReadyFuture {
        done: Arc<AtomicBool>,
    }

    impl Future for ReadyFuture {
        type Output = String;

        fn poll(&mut self, _waker: &Waker) -> PollState<Self::Output> {
            self.done.store(true, Ordering::SeqCst);
            PollState::Ready("done".to_string())
        }
    }

    struct WakeFuture {
        remaining: usize,
        polls: Arc<AtomicUsize>,
        done: Arc<AtomicBool>,
    }

    impl Future for WakeFuture {
        type Output = String;

        fn poll(&mut self, waker: &Waker) -> PollState<Self::Output> {
            self.polls.fetch_add(1, Ordering::SeqCst);

            if self.remaining == 0 {
                self.done.store(true, Ordering::SeqCst);
                PollState::Ready("done".to_string())
            } else {
                self.remaining -= 1;
                waker.wake();
                PollState::NotReady
            }
        }
    }

    fn reset_executor_core() {
        CURRENT_EXEC.with(|e| {
            e.tasks.borrow_mut().clear();
            e.ready_queue.lock().unwrap().clear();
            e.next_id.set(0);
        });
    }

    #[test]
    fn block_on_polls_a_ready_future() {
        reset_executor_core();

        let done = Arc::new(AtomicBool::new(false));
        let mut executor = Executor::new();

        executor.block_on(ReadyFuture { done: done.clone() });

        assert!(done.load(Ordering::SeqCst));
    }

    #[test]
    fn block_on_repolls_a_task_after_it_is_woken() {
        reset_executor_core();

        let done = Arc::new(AtomicBool::new(false));
        let polls = Arc::new(AtomicUsize::new(0));
        let mut executor = Executor::new();

        executor.block_on(WakeFuture {
            remaining: 2,
            polls: polls.clone(),
            done: done.clone(),
        });

        assert!(done.load(Ordering::SeqCst));
        assert_eq!(polls.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn spawn_queues_a_task_with_a_unique_id() {
        reset_executor_core();

        spawn(ReadyFuture {
            done: Arc::new(AtomicBool::new(false)),
        });

        CURRENT_EXEC.with(|e| {
            assert_eq!(e.tasks.borrow().len(), 1);
            assert_eq!(e.ready_queue.lock().unwrap().as_slice(), &[0]);
            assert_eq!(e.next_id.get(), 1);
        });
    }

    #[test]
    fn waker_pushes_its_task_id_to_the_ready_queue() {
        let ready_queue = Arc::new(Mutex::new(Vec::new()));
        let waker = Waker {
            thread: thread::current(),
            id: 42,
            ready_queue: ready_queue.clone(),
        };

        waker.wake();

        assert_eq!(*ready_queue.lock().unwrap(), vec![42]);
    }
}
