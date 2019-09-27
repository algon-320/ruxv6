use super::address::*;
use core::marker::PhantomData;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Ptr<T> {
    addr: vaddr,
    _phantom: PhantomData<T>,
}

impl<T> Ptr<T> {
    pub fn null() -> Self {
        Self::from(vaddr::null())
    }
    pub fn is_null(&self) -> bool {
        self.addr.is_null()
    }

    pub fn cast<U>(&self) -> Ptr<U> {
        Ptr {
            addr: self.addr,
            _phantom: PhantomData,
        }
    }

    pub fn address(&self) -> vaddr {
        self.addr.clone()
    }
    pub fn get(&self) -> *const T {
        self.addr.as_ptr()
    }
    pub fn get_mut(&self) -> *mut T {
        self.addr.as_mut_ptr()
    }

    pub fn increase(&mut self, units: usize) {
        self.addr.increase(units * core::mem::size_of::<T>());
    }
    pub fn decrease(&mut self, units: usize) {
        self.addr.decrease(units * core::mem::size_of::<T>());
    }
    pub fn increase_bytes(&mut self, bytes: usize) -> Option<()> {
        self.addr.increase_bytes(bytes)
    }
    pub fn decrease_bytes(&mut self, bytes: usize) -> Option<()> {
        self.addr.decrease_bytes(bytes)
    }
    pub fn next(&self, units: usize) -> Self {
        let mut ret = Self::from(self.addr);
        ret.increase(units);
        ret
    }
    pub fn prev(&self, units: usize) -> Self {
        let mut ret = Self::from(self.addr);
        ret.decrease(units);
        ret
    }
    pub fn next_bytes(&self, bytes: usize) -> Option<Self> {
        let mut ret = Self::from(self.addr);
        ret.increase_bytes(bytes)?;
        Some(ret)
    }
    pub fn prev_bytes(&self, bytes: usize) -> Option<Self> {
        let mut ret = Self::from(self.addr);
        ret.decrease_bytes(bytes)?;
        Some(ret)
    }
}

use core::ops::{Deref, DerefMut};
impl<T> Deref for Ptr<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.get() }
    }
}
impl<T> DerefMut for Ptr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.get_mut() }
    }
}

impl<T, A: Align> From<Address<Virtual, A>> for Ptr<T> {
    fn from(a: Address<Virtual, A>) -> Ptr<T> {
        Ptr {
            addr: a.check_aligned().unwrap(),
            _phantom: PhantomData,
        }
    }
}
impl<T> From<*const T> for Ptr<T> {
    fn from(a: *const T) -> Ptr<T> {
        Ptr {
            addr: vaddr::from_ptr(a).unwrap(),
            _phantom: PhantomData,
        }
    }
}

use core::fmt;
impl<T> fmt::Display for Ptr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ptr({})", self.address())
    }
}
