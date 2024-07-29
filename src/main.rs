use std::{thread, time::{Duration, Instant}};

mod http;
mod future;

use crate::http::Http;
use future::{Future, PollState};

fn main() {
    let mut future = async_main();
    loop {
        match future.poll() {
            PollState::NotReady => {
                println!("Schedule other tasks");
            },
            PollState::Ready(_) => break,
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn async_main() -> impl Future<Output = ()> {
    Coroutine::new()
}

struct Coroutine {
    state: State,
}

enum State {
    // Created but hasn't been polled yet
    Start,
    // HttpGetFuture returned is stored
    // At this point we return control back to callin fn
    // Generic over all Future functions that output a String
    Wait1(Box<dyn Future<Output = String>>),
    // The second Http::get is the second place where we'll pass
    // control back to the calling function
    Wait2(Box<dyn Future<Output = String>>),
    // The future is resolved and there is no more work to do
    Resolved,
}

impl Coroutine {
    fn new() -> Self {
        Self {
            state: State::Start,
        }
    }
}

impl Future for Coroutine {
    type Output = ();

    fn poll(&mut self) -> PollState<Self::Output> {
        loop {
            match self.state {
                State::Start => {
                    println!("Program starting");
                    let fut = Box::new(Http::get("/600/HelloWorld1"));
                    self.state = State::Wait1(fut);
                }
                State::Wait1(ref mut fut) => match fut.poll() {
                    PollState::Ready(txt) => {
                        println!("{txt}");
                        let fut2 = Box::new(Http::get("/400/HelloWorld2"));
                        self.state = State::Wait2(fut2);
                    }
                    PollState::NotReady => break PollState::NotReady,
                }
                State::Wait2(ref mut fut2) => match fut2.poll() {
                    PollState::Ready(txt2) => {
                        println!("{txt2}");
                        self.state = State::Resolved;
                        break PollState::Ready(());
                    }
                    PollState::NotReady => break PollState::NotReady,
                }
                State::Resolved => panic!("Polled a resolved future")
            }
        }
    }
}