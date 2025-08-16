use super::{FlowEvent, FlowState, FnControlEvent, Slot, UserControlEvent};
use crate::runtime::FlowRuntime;
use anyhow::Result;
use core::future::Future;
use core::pin::Pin;
use core::task::{Poll, Waker};

/// Controller for the async function being controlled
/// Can only send Block events and use runtime methods
pub struct FnController<R: FlowRuntime, U: 'static, const N: usize> {
    slot: &'static Slot<U, N>,
    runtime: &'static R,
}

impl<R: FlowRuntime, U, const N: usize> FnController<R, U, N> {
    pub fn new(slot: &'static Slot<U, N>, runtime: &'static R) -> Self {
        Self { slot, runtime }
    }

    /// Send a block event to indicate the function needs to wait for user input
    /// TODO this should be an async function that resolves to a U when the user resolves the block
    pub fn block(&self) -> Result<()> {
        self.slot.send(FlowEvent::Fn(FnControlEvent::Block));
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
pub struct UserController<U: 'static, const N: usize> {
    slot: &'static Slot<U, N>,
}

impl<U: 'static, const N: usize> UserController<U, N> {
    pub fn new(slot: &'static Slot<U, N>) -> Self {
        Self { slot }
    }

    /// Pause the flow execution
    pub fn pause(&self) -> Result<()> {
        self.slot.send(FlowEvent::User(UserControlEvent::Pause));
        Ok(())
    }

    /// Resume the flow execution
    pub fn resume(&self) -> Result<()> {
        self.slot.send(FlowEvent::User(UserControlEvent::Resume));
        Ok(())
    }

    /// Cancel the flow execution
    pub fn cancel(&self) -> Result<()> {
        self.slot.send(FlowEvent::User(UserControlEvent::Cancel));
        Ok(())
    }

    /// Send user input to unblock the function
    pub fn invoke(&self, input: U) -> Result<()> {
        self.slot
            .send(FlowEvent::User(UserControlEvent::Invoke(input)));
        Ok(())
    }
}

/// Controller for user the flow future itself
/// Can only read events
pub struct FlowFutureController<U: 'static, const N: usize> {
    slot: &'static Slot<U, N>,
}

impl<U: 'static, const N: usize> FlowFutureController<U, N> {
    pub fn new(slot: &'static Slot<U, N>) -> Self {
        Self { slot }
    }

    pub fn consume<F: Future>(
        &self,
        current: &FlowState,
        future: Pin<&mut F>,
        waker: &Waker,
    ) -> (FlowState, Poll<F::Output>) {
        self.slot.consume(current, future, waker)
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
