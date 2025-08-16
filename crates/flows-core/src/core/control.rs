use super::{
    AtomicWaker, FlowEvent, FlowEventHandler, FlowState, FnControlEvent, Handler, Reset,
    UserControlEvent,
};
use crate::runtime::FlowRuntime;
use anyhow::Result;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use heapless::mpmc::MpMcQueue;

pub struct BaseController<U: 'static, const CHAN_N: usize> {
    channel: MpMcQueue<FlowEvent<U>, CHAN_N>,
    handler: FlowEventHandler,
    waker: AtomicWaker,
}

impl<U: 'static, const CHAN_N: usize> Default for BaseController<U, CHAN_N> {
    fn default() -> Self {
        BaseController {
            channel: MpMcQueue::new(),
            handler: FlowEventHandler::default(),
            waker: AtomicWaker::new(),
        }
    }
}

impl<U: 'static, const CHAN_N: usize> Reset for BaseController<U, CHAN_N> {
    fn reset(&self) {
        while let Some(_) = self.channel.dequeue() {}
        // should find some way to invalidate the waker at this point, maybe.
    }
}

impl<U: 'static, const CHAN_N: usize> BaseController<U, CHAN_N> {
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
}

/// Controller for the async function being controlled
/// Can only send Block events and use runtime methods
pub struct FnController<R: FlowRuntime, U: 'static, const CHAN_N: usize> {
    inner: &'static BaseController<U, CHAN_N>,
    runtime: &'static R,
}

impl<R: FlowRuntime, U: 'static, const CHAN_N: usize> FnController<R, U, CHAN_N> {
    pub fn new(inner: &'static BaseController<U, CHAN_N>, runtime: &'static R) -> Self {
        Self { inner, runtime }
    }

    /// Send a block event to indicate the function needs to wait for user input
    /// TODO this should be an async function that resolves to a U when the user resolves the block
    pub fn block(&self) -> Result<()> {
        self.inner.send(FlowEvent::Fn(FnControlEvent::Block));
        todo!(
            "This should be an async function that resolves to a U when the user resolves the block"
        );
    }

    /// Yield control to allow other tasks to run
    pub fn yield_now(&self) -> impl Future<Output = ()> + '_ {
        self.runtime.yield_now()
    }

    /// Delay execution for the specified number of milliseconds
    pub fn delay_ms(&self, millis: u32) -> impl Future<Output = ()> + '_ {
        self.runtime.delay_ms(millis)
    }
}

/// Controller for user operations
/// Can send Pause/Resume/Cancel/Invoke events
pub struct UserController<U: 'static, const CHAN_N: usize> {
    inner: &'static BaseController<U, CHAN_N>,
}

impl<U: 'static, const CHAN_N: usize> UserController<U, CHAN_N> {
    pub fn new(inner: &'static BaseController<U, CHAN_N>) -> Self {
        Self { inner }
    }

    /// Pause the flow execution
    pub fn pause(&self) -> Result<()> {
        self.inner.send(FlowEvent::User(UserControlEvent::Pause));
        Ok(())
    }

    /// Resume the flow execution
    pub fn resume(&self) -> Result<()> {
        self.inner.send(FlowEvent::User(UserControlEvent::Resume));
        Ok(())
    }

    /// Cancel the flow execution
    pub fn cancel(&self) -> Result<()> {
        self.inner.send(FlowEvent::User(UserControlEvent::Cancel));
        Ok(())
    }

    /// Send user input to unblock the function
    pub fn invoke(&self, input: U) -> Result<()> {
        self.inner
            .send(FlowEvent::User(UserControlEvent::Invoke(input)));
        Ok(())
    }
}

/// Controller for user the flow future itself
/// Can only read events
pub struct FlowFutureController<U: 'static, const CHAN_N: usize> {
    inner: &'static BaseController<U, CHAN_N>,
}

impl<U: 'static, const CHAN_N: usize> FlowFutureController<U, CHAN_N> {
    pub fn new(inner: &'static BaseController<U, CHAN_N>) -> Self {
        Self { inner }
    }

    pub fn consume<F: Future>(
        &self,
        current: &FlowState,
        future: Pin<&mut F>,
        waker: &Waker,
    ) -> (FlowState, Poll<F::Output>) {
        self.inner.consume(current, future, waker)
    }
}

// pub struct UserQueryFuture<T> {
//     query_id: u64,
//     response: Option<T>,
// }

// impl<T> Future for UserQueryFuture<T>
// where
//     T: Unpin,
// {
//     type Output = T;

//     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//         let this = self.get_mut();
//         if let Some(response) = this.response.take() {
//             Poll::Ready(response)
//         } else {
//             cx.waker().wake_by_ref();
//             Poll::Pending
//         }
//     }
// }
