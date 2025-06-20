pub mod listener;
pub mod stream;
pub mod task;
pub mod utils;

#[derive(Clone, Copy)]
pub enum Connection {
    Close,
    Continue,
    StopServer,
}
