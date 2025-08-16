use flows::runtime::tokio::TokioRuntime;

const QUEUE_SIZE: usize = 8;

async fn example(_ctrl: flows_core::core::FnController<TokioRuntime, (), QUEUE_SIZE>) -> () {
    println!("Task: Starting interactive workflow");

    for i in 1..6 {
        println!("Task: Step {}/5", i);
        tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
    }

    println!("Workflow completed successfully!");
}

static RUNTIME: std::sync::LazyLock<TokioRuntime> =
    std::sync::LazyLock::new(|| TokioRuntime::new());

static SLOT_1: std::sync::LazyLock<flows_core::core::Slot<(), QUEUE_SIZE>> =
    std::sync::LazyLock::new(|| flows_core::core::Slot::default());

#[tokio::main]
async fn main() {
    println!("Enhanced Pausable Futures Demo (Tokio Runtime)");
    println!("===============================================");

    let slot = &*SLOT_1;
    let runtime = &*RUNTIME;

    let fn_controller = flows_core::core::FnController::new(slot, runtime);
    let flow_func_ctrl = flows_core::core::FlowFutureController::new(slot);
    let user_ctrl = flows_core::core::UserController::new(slot);
    let future = example(fn_controller);
    let flow = flows_core::core::Flow::new(future, flow_func_ctrl);

    let handle = tokio::spawn(flow);
    tokio::time::sleep(std::time::Duration::from_millis(3500)).await;
    user_ctrl.pause();
    tokio::time::sleep(std::time::Duration::from_millis(10000)).await;
    user_ctrl.resume();
    handle.await;
}
