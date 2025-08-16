use crate::core::{FlowFutureController, FlowState};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

/// A controllable future that can be paused, resumed, and cancelled
pub struct Flow<F, U, const N: usize>
where
    F: Future,
    U: 'static,
{
    inner: F,
    ctrl: FlowFutureController<U, N>,
    state: FlowState,
}

impl<F, U, const N: usize> Flow<F, U, N>
where
    F: Future,
{
    /// Create a new Flow wrapping the given future
    pub fn new(future: F, ctrl: FlowFutureController<U, N>) -> Self {
        Self {
            inner: future,
            ctrl,
            state: FlowState::default(),
        }
    }
}

impl<F, U, const N: usize> Future for Flow<F, U, N>
where
    F: Future,
    U: 'static,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let waker = cx.waker().clone();

        let this = unsafe { self.get_unchecked_mut() };
        let inner_future = unsafe { Pin::new_unchecked(&mut this.inner) };
        let current = this.state.clone();
        let (next, output) = this.ctrl.consume(&current, inner_future, &waker);
        this.state = next;
        output
    }
}
