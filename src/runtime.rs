use crate::future::{Future, PollState};
use mio::{Events, Poll, Registry};
use std::{future, sync::OnceLock};

mod executor;
mod reactor;

pub use executor::{Executor, Executor as Runtime, Waker};
pub use reactor::reactor;

pub fn init() -> Executor {
    reactor::start();
    Executor::new()
}
