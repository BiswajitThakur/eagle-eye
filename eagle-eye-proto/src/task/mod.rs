mod r#async;
mod sync;

pub use sync::{ExecuteResult, TaskRegisterySync, TaskSync};

pub trait GetId {
    fn id() -> &'static str;
}
