pub mod listener;
pub mod stream;
pub mod task;
pub mod utils;

#[derive(Clone, Copy)]
pub enum FlowControl {
    Close,
    Continue,
    StopServer,
}
