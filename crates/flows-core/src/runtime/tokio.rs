use super::{FlowRuntime, Spawner, Timer, WakerBuilder};
use core::future::Future;
use core::task::Waker;
use core::time::Duration;
use tokio::task::JoinHandle;

#[derive(Debug, Clone)]
pub struct TokioRuntime;

impl TokioRuntime {
    pub fn new() -> Self {
        Self
    }
}

impl Timer for TokioRuntime {
    type DelayFuture = tokio::time::Sleep;

    fn delay_ms(&self, millis: u32) -> Self::DelayFuture {
        tokio::time::sleep(Duration::from_millis(millis as u64))
    }

    fn delay_us(&self, micros: u64) -> Self::DelayFuture {
        tokio::time::sleep(Duration::from_micros(micros))
    }
}

impl Spawner for TokioRuntime {
    type Handle = JoinHandle<()>;
    type Error = ();

    fn spawn<F>(&self, future: F) -> Result<Self::Handle, Self::Error>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        Ok(tokio::spawn(future))
    }
}

impl WakerBuilder for TokioRuntime {
    fn build_waker(&self) -> Waker {
        std::task::Waker::from(std::sync::Arc::new(TokioWaker))
    }
}

impl FlowRuntime for TokioRuntime {
    async fn yield_now(&self) {
        tokio::task::yield_now().await
    }
}

struct TokioWaker;

impl std::task::Wake for TokioWaker {
    fn wake(self: std::sync::Arc<Self>) {
        // For tokio, we don't need to do anything special here
        // The tokio runtime handles waking automatically
    }
}

impl Default for TokioRuntime {
    fn default() -> Self {
        Self::new()
    }
}
