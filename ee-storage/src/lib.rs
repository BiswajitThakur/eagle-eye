pub mod value;

use std::{any::TypeId, collections::HashMap};

trait Storage {
    fn get<T: 'static>(&self, key: impl AsRef<str>) -> Option<&T>;
}

struct MyStorage {
    inner: HashMap<String, Value>,
}

struct Value {
    ptr: *mut u8,
    id: TypeId,
    _drop: Box<dyn Fn(*mut u8)>,
}

impl Drop for Value {
    fn drop(&mut self) {
        let f = &self._drop;
        f(self.ptr);
    }
}

impl Value {
    fn new<T: 'static>(v: T) -> Self {
        let v = Box::new(v);
        let raw = Box::into_raw(v);
        let id = TypeId::of::<T>();
        let _drop = |ptr| unsafe {
            let _v = Box::from_raw(ptr as *mut T);
        };
        Self {
            ptr: raw as *mut u8,
            id,
            _drop: Box::new(_drop),
        }
    }
    fn get<T: 'static>(&self) -> Option<&T> {
        if self.id != TypeId::of::<T>() {
            return None;
        }
        let val = unsafe { &*(self.ptr as *const T) };
        Some(val)
    }
}

impl MyStorage {
    fn insert<T: 'static>(&mut self, key: impl Into<String>, value: T) {
        let v = Value::new(value);
        self.inner.insert(key.into(), v);
    }
}

impl Storage for MyStorage {
    fn get<T: 'static>(&self, key: impl AsRef<str>) -> Option<&T> {
        let v = self.inner.get(key.as_ref())?;
        v.get::<T>()
    }
}

