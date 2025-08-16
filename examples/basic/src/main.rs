use flows::runtime::tokio::TokioRuntime;

const CHANNEL_SIZE: usize = 8;
const DATA_CHANNEL_SIZE: usize = 16;

async fn example(
    _init: (),
    _ctrl: flows::FnController<TokioRuntime, (), CHANNEL_SIZE>,
    _data: flows::FnDataHandle<(), (), DATA_CHANNEL_SIZE>,
) -> () {
    println!("Task: Starting interactive workflow");

    for i in 1..6 {
        println!("Task: Step {}/5", i);
        tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
    }

    println!("Workflow completed successfully!");
}

static RUNTIME: std::sync::LazyLock<TokioRuntime> =
    std::sync::LazyLock::new(|| TokioRuntime::new());

static SLOT_1: std::sync::LazyLock<flows::Slot<(), (), (), CHANNEL_SIZE, DATA_CHANNEL_SIZE>> =
    std::sync::LazyLock::new(|| flows::Slot::default());

#[tokio::main]
async fn main() {
    println!("Enhanced Pausable Futures Demo (Tokio Runtime)");
    println!("===============================================");

    let slot = &*SLOT_1;
    let runtime = &*RUNTIME;

    let (fn_data_handle, user_data_handle) = slot.handles();
    let (fn_ctrl, flow_func_ctrl, user_ctrl) = slot.ctrls(runtime);

    let future = example((), fn_ctrl, fn_data_handle);
    let flow = flows::Flow::new(future, flow_func_ctrl);

    let handle = tokio::spawn(flow);
    tokio::time::sleep(std::time::Duration::from_millis(3500)).await;
    user_ctrl.pause();
    tokio::time::sleep(std::time::Duration::from_millis(10000)).await;
    user_ctrl.resume();
    handle.await;
}
