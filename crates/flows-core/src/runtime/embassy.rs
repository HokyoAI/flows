use super::{FlowRuntime, Spawner, Timer, WakerBuilder};
use core::future::Future;
use core::task::Waker;

#[derive(Debug, Clone)]
pub struct EmbassyRuntime;

impl EmbassyRuntime {
    pub fn new() -> Self {
        Self
    }
}

impl Timer for EmbassyRuntime {
    type DelayFuture = embassy_time::Timer;

    fn delay_ms(&self, millis: u32) -> Self::DelayFuture {
        embassy_time::Timer::after(embassy_time::Duration::from_millis(millis as u64))
    }

    fn delay_us(&self, micros: u64) -> Self::DelayFuture {
        embassy_time::Timer::after(embassy_time::Duration::from_micros(micros))
    }
}


impl Spawner for EmbassyRuntime {
    type Handle = ();
    type Error = ();

    fn spawn<F>(&self, _future: F) -> Result<Self::Handle, Self::Error>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        // Embassy spawning would be runtime-specific and typically handled
        // by the executor. This is a placeholder implementation.
        Ok(())
    }
}

impl WakerBuilder for EmbassyRuntime {
    fn build_waker(&self) -> Waker {
        // For embassy, we create a simple waker
        // In practice, this would be integrated with the embassy executor
        use core::task::{RawWaker, RawWakerVTable};
        
        const VTABLE: RawWakerVTable =
            RawWakerVTable::new(|_| RAW_WAKER, |_| {}, |_| {}, |_| {});
        const RAW_WAKER: RawWaker = RawWaker::new(core::ptr::null(), &VTABLE);
        
        unsafe { Waker::from_raw(RAW_WAKER) }
    }
}

impl FlowRuntime for EmbassyRuntime {
    async fn yield_now(&self) {
        // Embassy equivalent of yielding
        // embassy_executor::yield_now().await;
        core::future::ready(()).await;
    }
}


impl Default for EmbassyRuntime {
    fn default() -> Self {
        Self::new()
    }
}
