use crate::value::Value;

pub trait AppData: Default {
    fn get<T: 'static>(&self, key: impl AsRef<str>) -> Option<&T>;
    fn get_mut<T: 'static>(&mut self, key: impl AsRef<str>) -> Option<&mut T>;
    unsafe fn get_unchecked<T: 'static>(&self, key: impl AsRef<str>) -> Option<&T>;
    unsafe fn get_mut_unchecked<T: 'static>(&mut self, key: impl AsRef<str>) -> Option<&mut T>;
}

pub struct MyStorage {
    inner: Vec<(String, Value)>,
}

impl Default for MyStorage {
    fn default() -> Self {
        Self { inner: Vec::new() }
    }
}

impl MyStorage {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            inner: Vec::with_capacity(cap),
        }
    }
    pub fn insert<T: 'static>(&mut self, key: impl Into<String>, value: T) {
        self.inner.push((key.into(), Value::new(value)));
        self.inner.sort_by(|a, b| (&a.0).cmp(&b.0));
    }
    pub fn find_index(&self, key: impl AsRef<str>) -> Option<usize> {
        let new_key = key.as_ref();
        self.inner
            .binary_search_by(|v| (&v.0).as_str().cmp(new_key))
            .ok()
    }
}

impl AppData for MyStorage {
    fn get<T: 'static>(&self, key: impl AsRef<str>) -> Option<&T> {
        let index = self.find_index(key)?;
        let val = unsafe { self.inner.get_unchecked(index) };
        val.1.get()
    }
    fn get_mut<T: 'static>(&mut self, key: impl AsRef<str>) -> Option<&mut T> {
        let index = self.find_index(key)?;
        let val = unsafe { &mut self.inner.get_unchecked_mut(index).1 };
        val.get_mut()
    }
    unsafe fn get_unchecked<T: 'static>(&self, key: impl AsRef<str>) -> Option<&T> {
        let index = self.find_index(key)?;
        let val = unsafe { self.inner.get_unchecked(index) };
        Some(unsafe { val.1.get_unchecked() })
    }
    unsafe fn get_mut_unchecked<T: 'static>(&mut self, key: impl AsRef<str>) -> Option<&mut T> {
        let index = self.find_index(key)?;
        let val = unsafe { &mut self.inner.get_unchecked_mut(index).1 };
        Some(unsafe { val.get_mut_unchecked() })
    }
}

#[cfg(test)]
mod tests {
    use crate::app_data::{AppData, MyStorage};

    #[test]
    fn test_get() {
        let mut storage = MyStorage::new();
        storage.insert("k-123", "hello".to_string());
        storage.insert("my-key", vec![1u8, 3, 5, 7, 11]);
        let got = storage.get::<String>("k-123").map(|v| v.as_str());
        assert_eq!(got, Some("hello"));
        let got = storage.get::<Vec<u8>>("my-key");
        assert_eq!(got, Some(&vec![1, 3, 5, 7, 11]));
    }
}
