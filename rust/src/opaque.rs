use std::ops::{Deref, DerefMut};

pub struct OpaquePtr<T> {
    ptr: *mut T,
}

/// Marker trait to allow the opaque wrapper ZSTs to know which `T` they
/// represent when being unwrapped in an `OpaquePtr<T>`.
pub trait OpaqueTarget<'a> {
    type Target;
}

impl<T> OpaquePtr<T> {
    pub fn from_ptr(ptr: *mut T) -> Self {
        Self { ptr }
    }

    pub fn from_box(boxed_value: Box<T>) -> Self {
        Self::from_ptr(Box::into_raw(boxed_value))
    }

    pub fn new(value: T) -> Self {
        Self::from_box(Box::new(value))
    }

    pub unsafe fn free(&self) {
        Box::from_raw(self.ptr);
    }

    pub fn as_ptr(&self) -> *mut T {
        self.ptr
    }

    pub fn as_ref(&self) -> &mut T {
        unsafe { &mut *self.ptr }
    }

    // Only allow the single type declared as the wrapped "opaque pointee"
    // type to be converted to its opaque wrapper
    pub fn from_opaque<'a, O>(opaque: *mut O) -> Self
    where
        O: OpaqueTarget<'a, Target = T>,
    {
        Self::from_ptr(opaque as *mut T)
    }

    pub fn opaque<O>(&self) -> *mut O {
        self.ptr as *mut O
    }
}

impl<T> Deref for OpaquePtr<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.as_ref()
    }
}

impl<T> DerefMut for OpaquePtr<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.as_ref()
    }
}
