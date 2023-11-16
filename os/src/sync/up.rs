use core::cell::{RefCell, RefMut};

pub(crate) struct UpSafeCell<T> {
    inner: RefCell<T>,
}

unsafe impl<T> Sync for UpSafeCell<T> {}

impl<T> UpSafeCell<T> {
    // unsafe just a marker, telling programmer that you should take care of it, the compiler will not check for you.
    pub(crate) unsafe fn new(value: T) -> Self {
        Self {
            inner: RefCell::new(value),
        }
    }

    // Panic when data **borrowed** (both immutable borrow and mutable borrow)
    pub(crate) fn exclusive_access(&self) -> RefMut<T> {
        self.inner.borrow_mut()
    }
}
