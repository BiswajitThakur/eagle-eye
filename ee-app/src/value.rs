use std::any::TypeId;

pub struct Value {
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
    pub fn new<T: 'static>(v: T) -> Self {
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
    pub fn get<T: 'static>(&self) -> Option<&T> {
        if self.id != TypeId::of::<T>() {
            return None;
        }
        let val = unsafe { &*(self.ptr as *const T) };
        Some(val)
    }
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        if self.id != TypeId::of::<T>() {
            return None;
        }
        let val = unsafe { &mut *(self.ptr as *mut T) };
        Some(val)
    }
    pub unsafe fn get_unchecked<T>(&self) -> &T {
        unsafe { &*(self.ptr as *const T) }
    }

    pub unsafe fn get_mut_unchecked<T>(&self) -> &mut T {
        unsafe { &mut *(self.ptr as *mut T) }
    }
    pub fn as_ptr(&self) -> *const u8 {
        self.ptr
    }
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.ptr
    }
}
