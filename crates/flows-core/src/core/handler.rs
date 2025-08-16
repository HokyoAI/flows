use core::pin::Pin;
use core::task::{Context, Poll};
pub trait Handler<ST, E>: Default {
    fn transition(&self, current: &ST, event: &E) -> ST;
    fn transient_exec(&self, state: &ST) -> ();
    fn exec<F: Future>(
        &self,
        state: &FlowState,
        future: Pin<&mut F>,
        cx: &mut Context,
    ) -> Poll<F::Output>;
}

pub enum UserControlEvent<U> {
    Pause,
    Resume,
    Invoke(U),
    Cancel,
}

pub enum FnControlEvent {
    Block,
}

pub enum FlowEvent<U> {
    User(UserControlEvent<U>),
    Fn(FnControlEvent),
}

#[derive(Debug, Clone, PartialEq)]
pub enum FlowState {
    Running,
    Paused,
    Blocked,
    Cancelled,
    Completed,
    Error,
}

impl Default for FlowState {
    fn default() -> Self {
        FlowState::Running
    }
}

pub struct FlowEventHandler {}

impl Default for FlowEventHandler {
    fn default() -> Self {
        Self {}
    }
}

impl<U> Handler<FlowState, FlowEvent<U>> for FlowEventHandler {
    fn transition(&self, current: &FlowState, event: &FlowEvent<U>) -> FlowState {
        match (current, event) {
            // terminal states (maybe we should error instead)
            (FlowState::Completed, _) => FlowState::Completed,
            (FlowState::Error, _) => FlowState::Error,
            (FlowState::Cancelled, _) => FlowState::Cancelled,

            // always allow cancellation if not terminal
            (_, FlowEvent::User(UserControlEvent::Cancel)) => FlowState::Cancelled,

            // pause running
            (FlowState::Running, FlowEvent::User(UserControlEvent::Pause)) => FlowState::Paused,
            // block running
            (FlowState::Running, FlowEvent::Fn(FnControlEvent::Block)) => FlowState::Blocked,

            // resume
            (FlowState::Paused, FlowEvent::User(UserControlEvent::Resume)) => FlowState::Running,
            // unblock
            (FlowState::Blocked, FlowEvent::User(UserControlEvent::Invoke(_))) => {
                FlowState::Running
            }

            // no change - clone the current state
            _ => current.clone(),
        }
    }

    fn transient_exec(&self, state: &FlowState) -> () {
        // Execute behavior for a state we are passing through while transitioning through events
        // right now this is a noop, but eventually we may want to add some logic here for blocking/invoking
        // so that the invocation data is not lost
        match state {
            _ => {}
        }
    }

    fn exec<F: Future>(
        &self,
        state: &FlowState,
        future: Pin<&mut F>,
        cx: &mut Context,
    ) -> Poll<F::Output> {
        // Execute behavior for the ending state after transitioning through all events
        match state {
            FlowState::Running => future.poll(cx),
            _ => Poll::Pending,
        }
    }
}
