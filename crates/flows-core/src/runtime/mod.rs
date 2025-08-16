use core::future::Future;
use core::task::Waker;

#[cfg(feature = "tokio")]
pub mod tokio;

#[cfg(feature = "embassy")]
pub mod embassy;

pub trait Timer {
    type DelayFuture: Future<Output = ()>;
    fn delay_ms(&self, millis: u32) -> Self::DelayFuture;
    fn delay_us(&self, micros: u64) -> Self::DelayFuture;
}

pub trait Spawner {
    type Handle;
    type Error;

    fn spawn<F>(&self, future: F) -> Result<Self::Handle, Self::Error>
    where
        F: Future<Output = ()> + Send + 'static;
}

pub trait WakerBuilder {
    fn build_waker(&self) -> Waker;
}

pub trait FlowRuntime: Timer + Spawner + WakerBuilder + Clone + Send + Sync + 'static {
    fn yield_now(&self) -> impl Future<Output = ()>;
}
