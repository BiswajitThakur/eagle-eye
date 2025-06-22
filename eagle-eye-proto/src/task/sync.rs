use std::io;

use crate::FlowControl;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecuteResult {
    Ok = 0,
    InvalidRequest = 1,
    UnknownTask = 2,
}

/*
impl TryFrom<u8> for ExecuteResult {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Ok),
            1 => Ok(Self::InvalidRequest),
            2 => Ok(Self::UnknownTask),
            _ => Err(()),
        }
    }
}
*/

impl TryFrom<[u8; 1]> for ExecuteResult {
    type Error = ();
    fn try_from(value: [u8; 1]) -> Result<Self, Self::Error> {
        match value {
            [0] => Ok(Self::Ok),
            [1] => Ok(Self::InvalidRequest),
            [2] => Ok(Self::UnknownTask),
            _ => Err(()),
        }
    }
}

impl ExecuteResult {
    pub fn to_be_bytes(&self) -> [u8; 1] {
        let v = *self as u8;
        v.to_be_bytes()
    }
}

pub trait TaskSync<T: io::Read + io::Write, W: io::Write, E: io::Write> {
    fn id() -> &'static str;
    fn execute_on_sender(&self, stream: T, ok: W, err: E) -> io::Result<ExecuteResult>;
}

/// A synchronous task registry that maps string IDs to handler functions.
///
/// Each task handler takes a generic input `T` and returns a [`Result`] with a tuple
/// `(FlowControl, T)` where [`FlowControl`] represents how the stream/server should
/// behave after the task, and `T` is the updated context/state.
pub struct TaskRegisterySync<T> {
    _default: fn(T) -> io::Result<(FlowControl, T)>,
    tasks: Vec<(&'static str, fn(T) -> io::Result<(FlowControl, T)>)>,
}

impl<T> TaskRegisterySync<T> {
    /// Create a new [`TaskRegisterySync`] with a given default fallback handler.
    pub fn new_default(default: fn(T) -> io::Result<(FlowControl, T)>) -> Self {
        Self {
            _default: default,
            tasks: Vec::new(),
        }
    }
    /// Registers a new task handler by ID.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - the ID starts and ends with `:`
    /// - a task with the same ID already exists
    pub fn register(&mut self, id: &'static str, f: fn(T) -> io::Result<(FlowControl, T)>) {
        assert!(
            !(id.starts_with(':') && id.ends_with(':')),
            "ID should not start and end with `:`"
        );
        for &(v, _) in self.tasks.iter() {
            assert_ne!(id, v, "already exists with this ID...");
        }
        self.tasks.push((id, f));
        self.tasks.sort_by(|a, b| a.0.cmp(b.0));
    }
    /// Retrieves a registered task handler by ID.
    ///
    /// If the ID is not found in the registry, the default handler is returned.
    pub fn get<S: AsRef<str>>(&self, id: S) -> fn(T) -> io::Result<(FlowControl, T)> {
        let v = id.as_ref();
        match self.tasks.binary_search_by_key(&v, |&(a, _)| a) {
            Ok(index) => self.tasks.get(index).map(|m| m.1).unwrap_or(self._default),
            Err(_) => self._default,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use crate::FlowControl;

    use super::TaskRegisterySync;

    #[test]
    fn test_task_registery_sync_new_default() {
        let registery = TaskRegisterySync::new_default(|v: i32| Ok((FlowControl::Continue, v * v)));
        assert_eq!(registery.get("id")(7).unwrap(), (FlowControl::Continue, 49));

        let registery =
            TaskRegisterySync::new_default(|v: i32| Ok((FlowControl::StopServer, v + 10)));
        assert_eq!(
            registery.get("id")(7).unwrap(),
            (FlowControl::StopServer, 17)
        );

        let registery = TaskRegisterySync::new_default(|v: i32| Ok((FlowControl::Close, v * 2)));
        assert_eq!(registery.get("id")(7).unwrap(), (FlowControl::Close, 14));
    }

    #[test]
    fn test_task_registery_sync_register_1() {
        let mut registery =
            TaskRegisterySync::new_default(|v: i32| Ok((FlowControl::Continue, v * v)));
        registery.register("add-10", |v| Ok((FlowControl::Continue, v + 10)));
        registery.register("minus-10", |v| Ok((FlowControl::Close, v - 10)));
        registery.register("mul-2", |v| Ok((FlowControl::StopServer, v * 2)));
        registery.register("error", |_| Err(io::Error::other("unknown error")));

        assert_eq!(
            registery.get("not-found")(10).unwrap(),
            (FlowControl::Continue, 100)
        );

        assert_eq!(
            registery.get("add-10")(1).unwrap(),
            (FlowControl::Continue, 11)
        );

        assert_eq!(
            registery.get("minus-10")(11).unwrap(),
            (FlowControl::Close, 1)
        );

        assert_eq!(
            registery.get("mul-2")(202).unwrap(),
            (FlowControl::StopServer, 404)
        );

        assert!(registery.get("error")(10).is_err());
    }

    #[test]
    #[should_panic]
    fn test_task_registery_sync_register_2() {
        let mut registery =
            TaskRegisterySync::new_default(|v: i32| Ok((FlowControl::Continue, v * v)));

        registery.register("add-10", |v| Ok((FlowControl::Continue, v + 10)));
        // panic, id already exists
        registery.register("add-10", |v| Ok((FlowControl::Continue, v + 10)));
    }

    #[test]
    #[should_panic]
    fn test_task_registery_sync_register_3() {
        let mut registery =
            TaskRegisterySync::new_default(|v: i32| Ok((FlowControl::Continue, v * v)));

        // panic, id starts and ends with `:`
        registery.register(":add-10:", |v| Ok((FlowControl::Continue, v + 10)));
    }
}
