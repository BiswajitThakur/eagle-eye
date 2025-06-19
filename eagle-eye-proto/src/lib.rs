pub mod listener;
pub mod stream;
pub mod task;
pub(crate) mod utils;

#[derive(Clone, Copy)]
pub enum Connection {
    Close,
    Continue,
}
