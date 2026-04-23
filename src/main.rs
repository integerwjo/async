pub mod executor;
pub mod task;
pub mod reactor;
pub mod waker;
mod spawner;

fn main() {
    let (spawner, executor) = new_executor_and_spawner();
    spawner.new(example_task());
    spawner.new(another_task());

    executor.run();
   
}

async fn example_task() {
    println!("This is an example task.");
}

async fn another_task() {
    println!("This is another example task.");
}

fn new_executor_and_spawner() -> (spawner::Spawner, executor::Executor) {
    let (sender, receiver) = std::sync::mpsc::sync_channel(100);

    (
        spawner::Spawner { task_sender: sender },
        executor::Executor { tasks: receiver },
    )
}