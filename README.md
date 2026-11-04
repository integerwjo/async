# Asynchronous Runtime

A minimal asynchronous runtime implemented in Rust, featuring a reactor-executor pattern using `mio` for non-blocking I/O.

## Overview

This project implements a simple async runtime from scratch.
THe core components are:

- The `Future` trait and `PollState` enum
- A reactor pattern using `mio` for event demultiplexing
- An executor that drives futures to completion
- Non-blocking HTTP client operations

## Project Structure

| File | Purpose |
|------|---------|
| `src/future.rs` | Defines the `Future` trait and `PollState` enum |
| `src/runtime.rs` | Runtime initialization, executor, and reactor |
| `src/reactor.rs` | Reactor using `mio` for event handling |
| `src/executor.rs` | Executor that drives futures to completion |
| `src/http.rs` | Async HTTP GET client using the runtime |
| `src/main.rs` | Entry point running two async HTTP requests |

## Building and Running

```bash
cargo build
cargo run
```

## Usage

```rust
use async::runtime::Runtime;
use async::future::Future;

fn main() {
    let mut executor = async::runtime::init();
    executor.block_on(async_main());
}
```

