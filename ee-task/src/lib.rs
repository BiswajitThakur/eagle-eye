use std::io;

use ee_http::HttpRequest;
use ee_stream::EStreamSync;

pub mod file;
pub mod ping_pong;
pub mod prelude;

pub trait GetId {
    fn id() -> &'static str;
}

pub trait ExeSenderSync<T: io::Read + io::Write, W: io::Write>: GetId {
    fn execute_on_sender(
        &self,
        stream: T,
        req: &mut HttpRequest,
        http: W,
    ) -> io::Result<ExecuteResult>;
}

pub trait ExeReceiverSync: GetId {
    fn execute_on_receiver(stream: EStreamSync) -> io::Result<EStreamSync>;
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecuteResult {
    Ok = 0,
    InvalidRequest = 1,
    UnknownTask = 2,
    InvalidPath = 3,
    Faild = 4,
}

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

#[macro_export]
macro_rules! create_task_registery {
    { name : $v:vis $name:ident, handler: $t:ty } => {
        /// A synchronous task registry that maps string IDs to handler functions.
        #[allow(clippy::type_complexity)]
        $v struct $name {
            inner: Vec<(&'static str, $t)>,
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $name {
            /// Create a new empty [`TaskRegisterySync`].
            $v fn new() -> Self {
                Self { inner: Vec::new() }
            }

            /// Registers a new task handler by ID.
            ///
            /// # Panics
            ///
            /// Panics if:
            /// - the ID starts and ends with `:`
            /// - a task with the same ID already exists
            $v fn register(&mut self, id: &'static str, f: $t) {
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
            $v fn len(&self) -> usize {
                self.inner.len()
            }
            $v fn is_empty(&self) -> bool {
                self.inner.is_empty()
            }
            $v fn capacity(&self) -> usize {
                self.inner.capacity()
            }
            /// Retrieves a registered task handler by ID.
            $v fn get<S: AsRef<str>>(&self, id: S) -> Option<$t> {
                let v = id.as_ref();
                self.inner
                    .binary_search_by_key(&v, |&(a, _)| a)
                    .ok()
                    .map(|v| unsafe { self.inner.get_unchecked(v).1 })
            }
        }
    };
}

/*
/// A synchronous task registry that maps string IDs to handler functions.
#[allow(clippy::type_complexity)]
pub struct ReceiverTaskRegisterySync<const N: usize> {
    inner: Vec<(
        &'static str,
        for<'a> fn(
            EStreamSync<N, &'a TcpStream, &'a TcpStream>,
        ) -> io::Result<EStreamSync<N, &'a TcpStream, &'a TcpStream>>,
    )>,
}

impl<const N: usize> Default for ReceiverTaskRegisterySync<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> ReceiverTaskRegisterySync<N> {
    /// Create a new empty [`TaskRegisterySync`].
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    /// Registers a new task handler by ID.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - the ID starts and ends with `:`
    /// - a task with the same ID already exists
    pub fn register(
        &mut self,
        id: &'static str,
        f: for<'a> fn(
            EStreamSync<N, &'a TcpStream, &'a TcpStream>,
        ) -> io::Result<EStreamSync<N, &'a TcpStream, &'a TcpStream>>,
    ) {
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
    pub fn get<S: AsRef<str>>(
        &self,
        id: S,
    ) -> Option<
        for<'a> fn(
            EStreamSync<N, &'a TcpStream, &'a TcpStream>,
        ) -> io::Result<EStreamSync<N, &'a TcpStream, &'a TcpStream>>,
    > {
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

    use FlowControl;

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
*/
