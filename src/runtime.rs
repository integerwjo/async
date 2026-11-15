mod executor;
mod reactor;

pub use executor::{Executor, Executor as Runtime, Waker};
pub use reactor::reactor;

pub fn init() -> Runtime {
    reactor::start();
    Executor::new()
}
