use crate::runtime::Waker;
use mio::{Events, Interest, Poll, Registry, Token, net::TcpStream};

use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
};

type Wakers = Arc<Mutex<HashMap<usize, Waker>>>;

static REACTOR: OnceLock<Reactor> = OnceLock::new();

pub fn reactor() -> &'static Reactor {
    REACTOR.get().expect("Called outside a runtime context")
}

pub struct Reactor {
    // A hashmap of waker objects each identified by an integer
    wakers: Wakers,

    // Holds a Registry instance so that we can interact with the event queue in mio
    registry: Registry,

    // Stores the next available ID so that we can track which event occurred and which waker should
    // be woken
    next_id: AtomicUsize,
}

impl Reactor {
    pub fn register(&self, stream: &mut TcpStream, interest: Interest, id: usize) {
        // Which pass in the id to identify which event has occured later on when we get the
        // notification
        self.registry.register(stream, Token(id), interest).unwrap();
    }

    pub fn deregister(&self, stream: &mut TcpStream, _id: usize) {
        self.registry.deregister(stream).unwrap();
    }

    // Adds the waker to the hashmap using the id property as key to identify it
    pub fn set_waker(&self, waker: &Waker, id: usize) {
        let _ = self
            .wakers
            .lock()
            .map(|mut w| w.insert(id, waker.clone()).is_none())
            .unwrap();
    }
    pub fn next(&self) -> usize {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }
}

fn event_loop(mut poll: Poll, wakers: Wakers) {
    let mut events = Events::with_capacity(100);

    loop {
        poll.poll(&mut events, None).unwrap();

        for e in events.iter() {
            let Token(id) = e.token();
            let wakers = wakers.lock().unwrap();

            if let Some(waker) = wakers.get(&id) {
                waker.wake();
            }
        }
    }
}

pub fn start() {
    use thread::spawn;

    let wakers = Arc::new(Mutex::new(HashMap::new()));
    let poll = Poll::new().unwrap();
    let registry = poll.registry().try_clone().unwrap();

    let next_id = AtomicUsize::new(1);

    let reactor = Reactor {
        wakers: wakers.clone(),
        registry,
        next_id,
    };

    REACTOR.set(reactor).ok().expect("Reactor already running");

    spawn(move || event_loop(poll, wakers));
}
