use super::{AtomicWaker, FlowEvent, FlowEventHandler, FlowState, Handler};
use anyhow::Result;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use heapless::mpmc::MpMcQueue;

pub struct Slot<U, const N: usize> {
    channel: MpMcQueue<FlowEvent<U>, N>,
    handler: FlowEventHandler,
    waker: AtomicWaker,
}

impl<U, const N: usize> Default for Slot<U, N> {
    fn default() -> Self {
        Slot {
            channel: MpMcQueue::new(),
            handler: FlowEventHandler::default(),
            waker: AtomicWaker::new(),
        }
    }
}

impl<U, const N: usize> Slot<U, N> {
    // fn with_waker_lock<F, R>(&self, f: F) -> R
    // where
    //     F: FnOnce() -> R,
    // {
    //     // Simple spinlock
    //     while self
    //         .waker_lock
    //         .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
    //         .is_err()
    //     {
    //         core::hint::spin_loop();
    //     }

    //     let result = f();

    //     self.waker_lock.store(false, Ordering::Release);
    //     result
    // }

    /// used by the user and function to send events and wake the flow future
    pub fn send(&self, item: FlowEvent<U>) -> Result<(), FlowEvent<U>> {
        // maybe should check if waker exists before enqueue, or use a ready bit
        match self.channel.enqueue(item) {
            Ok(()) => {
                self.waker.wake();
                Ok(())
            }
            Err(item) => Err(item),
        }
    }

    /// used by the flow future
    /// consumes all events currently in the queue, possibly executing some code for each state transitioned to
    /// updates the waker to be the one the future was polled with
    pub fn consume<F: Future>(
        &self,
        current: &FlowState,
        future: Pin<&mut F>,
        waker: &Waker,
    ) -> (FlowState, Poll<F::Output>) {
        let mut state = current.clone();

        while let Some(event) = self.channel.dequeue() {
            state = self.handler.transition(&state, &event);
            // do not know why Rust wants the government name here
            <FlowEventHandler as Handler<FlowState, FlowEvent<U>>>::transient_exec(
                &self.handler,
                &state,
            );

            use std::println;
            println!("Transitioning from {:?} to {:?}", current, state);
        }

        self.waker.register(waker);

        let mut cx = Context::from_waker(waker);
        let output = <FlowEventHandler as Handler<FlowState, FlowEvent<U>>>::exec(
            &self.handler,
            &state,
            future,
            &mut cx,
        );
        (state, output)
    }

    /// resets the slot for use with a different flow
    pub fn reset(&self) {
        while let Some(_) = self.channel.dequeue() {}

        // should find some way to invalidate the waker at this point, maybe.
    }
}
