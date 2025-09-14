use std::any::Any;

pub trait AppData: Default {
    fn get<T: 'static>(&self, key: impl AsRef<str>) -> Option<&T>;
    fn get_mut<T: 'static>(&mut self, key: impl AsRef<str>) -> Option<&mut T>;
    // unsafe fn get_unchecked<T: 'static>(&self, key: impl AsRef<str>) -> Option<&T>;
    // unsafe fn get_mut_unchecked<T: 'static>(&mut self, key: impl AsRef<str>) -> Option<&mut T>;
    fn set<T: 'static>(&mut self, key: impl AsRef<str>);
}
