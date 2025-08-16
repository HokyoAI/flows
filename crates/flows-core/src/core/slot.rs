use super::{
    BaseController, DataChannel, FlowFutureController, FnController, FnDataHandle, Reset,
    UserController, UserDataHandle,
};
use crate::runtime::FlowRuntime;

pub struct Slot<U: 'static, UD: 'static, FD: 'static, const CHAN_N: usize, const DATA_N: usize> {
    ctrl: BaseController<U, CHAN_N>,
    data: DataChannel<UD, FD, DATA_N>,
}

impl<U: 'static, UD: 'static, FD: 'static, const CHAN_N: usize, const DATA_N: usize> Default
    for Slot<U, UD, FD, CHAN_N, DATA_N>
{
    fn default() -> Self {
        Slot {
            ctrl: BaseController::default(),
            data: DataChannel::default(),
        }
    }
}

impl<U: 'static, UD: 'static, FD: 'static, const CHAN_N: usize, const DATA_N: usize> Reset
    for Slot<U, UD, FD, CHAN_N, DATA_N>
{
    fn reset(&self) {
        self.ctrl.reset();
        self.data.reset();
    }
}

impl<U: 'static, UD: 'static, FD: 'static, const CHAN_N: usize, const DATA_N: usize>
    Slot<U, UD, FD, CHAN_N, DATA_N>
{
    pub fn handles(
        &'static self,
    ) -> (FnDataHandle<UD, FD, DATA_N>, UserDataHandle<UD, FD, DATA_N>) {
        (
            FnDataHandle::new(&self.data.fn_data, &self.data.user_data),
            UserDataHandle::new(&self.data.user_data, &self.data.fn_data),
        )
    }

    pub fn ctrls<R: FlowRuntime>(
        &'static self,
        runtime: &'static R,
    ) -> (
        FnController<R, U, CHAN_N>,
        FlowFutureController<U, CHAN_N>,
        UserController<U, CHAN_N>,
    ) {
        (
            FnController::new(&self.ctrl, runtime),
            FlowFutureController::new(&self.ctrl),
            UserController::new(&self.ctrl),
        )
    }
}
