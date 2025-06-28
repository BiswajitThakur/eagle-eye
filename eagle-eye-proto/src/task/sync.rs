use std::io;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecuteResult {
    Ok = 0,
    InvalidRequest = 1,
    UnknownTask = 2,
    InvalidPath = 3,
    Faild = 4,
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
            [3] => Ok(Self::InvalidPath),
            [4] => Ok(Self::Faild),
            _ => Err(()),
        }
    }
}

impl ExecuteResult {
    pub fn is_success(&self) -> bool {
        &Self::Ok == self
    }
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
#[allow(clippy::type_complexity)]
pub struct TaskRegisterySync<T> {
    pre_handler: Option<fn(T) -> io::Result<T>>,
    post_handler: Option<fn(T) -> io::Result<()>>,
    inner: Vec<(&'static str, fn(T) -> io::Result<T>)>,
}

impl<T> Default for TaskRegisterySync<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> TaskRegisterySync<T> {
    /// Create a new empty [`TaskRegisterySync`].
    pub fn new() -> Self {
        Self {
            pre_handler: None,
            post_handler: None,
            inner: Vec::new(),
        }
    }
    fn pre_handler_default(stream: T) -> io::Result<T> {
        Ok(stream)
    }
    pub fn set_pre_handler(&mut self, f: fn(T) -> io::Result<T>) {
        self.pre_handler = Some(f);
    }
    pub fn get_pre_handler(&mut self) -> Option<fn(T) -> io::Result<T>> {
        self.pre_handler
    }
    fn post_handler_default(stream: T) -> io::Result<()> {
        Ok(())
    }
    pub fn set_post_handler(&mut self, f: fn(T) -> io::Result<()>) {
        self.post_handler = Some(f);
    }
    pub fn get_post_handler(&mut self) -> Option<fn(T) -> Result<(), io::Error>> {
        self.post_handler
    }
    /// Registers a new task handler by ID.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - the ID starts and ends with `:`
    /// - a task with the same ID already exists
    pub fn register(&mut self, id: &'static str, f: fn(T) -> io::Result<T>) {
        assert!(
            !(id.starts_with(':') && id.ends_with(':')),
            "ID should not start and end with `:`"
        );
        for &(v, _) in self.inner.iter() {
            assert_ne!(id, v, "already exists with this ID...");
        }
        self.inner.push((id, f));
        self.inner.sort_by(|a, b| a.0.cmp(b.0));
    }
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }
    /// Retrieves a registered task handler by ID.
    pub fn get<S: AsRef<str>>(&self, id: S) -> Option<fn(T) -> io::Result<T>> {
        let v = id.as_ref();
        self.inner
            .binary_search_by_key(&v, |&(a, _)| a)
            .ok()
            .map(|v| unsafe { self.inner.get_unchecked(v).1 })
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use crate::FlowControl;

    use super::TaskRegisterySync;

    #[test]
    fn test_task_registery_sync_new_default() {
        let registery = TaskRegisterySync::<fn(i32) -> io::Result<(FlowControl, i32)>>::new();
        assert!(registery.get("id").is_none());
        assert_eq!(registery.len(), 0);
    }

    #[test]
    fn test_task_registery_sync_register_1() {
        let mut registery = TaskRegisterySync::default();
        registery.register("add-10", |v| Ok(v + 10));
        registery.register("minus-10", |v| Ok(v - 10));
        registery.register("mul-2", |v| Ok(v * 2));
        registery.register("error", |_| Err(io::Error::other("unknown error")));

        assert!(registery.get("not-found").is_none());

        let f = registery.get("add-10").unwrap();
        assert_eq!(f(1).unwrap(), 11);

        let f = registery.get("minus-10").unwrap();
        assert_eq!(f(11).unwrap(), 1);

        let f = registery.get("mul-2").unwrap();
        assert_eq!(f(202).unwrap(), 404);

        let f = registery.get("error").unwrap();
        assert!(f(10).is_err());
    }

    #[test]
    #[should_panic]
    fn test_task_registery_sync_register_2() {
        let mut registery = TaskRegisterySync::new();

        registery.register("add-10", |v: i32| Ok(v + 10));
        // panic, id already exists
        registery.register("add-10", |v| Ok(v + 10));
    }

    #[test]
    #[should_panic]
    fn test_task_registery_sync_register_3() {
        let mut registery = TaskRegisterySync::new();

        // panic, id starts and ends with `:`
        registery.register(":add-10:", |v: usize| Ok(v + 10));
    }
}
