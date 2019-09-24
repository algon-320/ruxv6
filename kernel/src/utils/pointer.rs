use super::address::*;
use core::marker::PhantomData;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Pointer<T> {
    addr: vaddr,
    phantom: PhantomData<T>,
}

impl<T> Pointer<T> {
    pub fn from<A: Align>(v: Address<Virtual, A>) -> Self {
        Pointer {
            addr: Address::from_raw(v.as_raw()).unwrap(),
            phantom: PhantomData,
        }
    }
    pub fn null() -> Self {
        Self::from(vaddr::null())
    }
    pub fn is_null(&self) -> bool {
        self.addr.is_null()
    }

    pub fn cast<U>(&self) -> Pointer<U> {
        Pointer {
            addr: self.addr,
            phantom: PhantomData,
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

    // pub fn shift(&mut self, offset: isize) {
    //     self.addr.shift(offset * core::mem::size_of::<T>() as isize);
    // }
    pub fn increase(&mut self, units: usize) {
        self.addr.increase(units * core::mem::size_of::<T>());
    }
    pub fn decrease(&mut self, units: usize) {
        self.addr.decrease(units * core::mem::size_of::<T>());
    }
}

use core::fmt;
impl<T> fmt::Display for Pointer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.address())
    }
}
