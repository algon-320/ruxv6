use crate::mmu;
use core::fmt;
use core::marker::PhantomData;
use core::num::Wrapping;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Virtual;
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Physical;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct FreeAligned;
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct PageAligned;

pub trait Align {
    fn check(addr: &usize) -> bool;
    fn bytes() -> usize;
    fn display() -> &'static str;
}

impl Align for FreeAligned {
    #[inline]
    fn check(_: &usize) -> bool {
        true
    }
    #[inline]
    fn bytes() -> usize {
        1
    }
    #[inline]
    fn display() -> &'static str {
        "free"
    }
}
impl Align for PageAligned {
    #[inline]
    fn check(addr: &usize) -> bool {
        addr % mmu::PGSIZE == 0
    }
    #[inline]
    fn bytes() -> usize {
        mmu::PGSIZE
    }
    #[inline]
    fn display() -> &'static str {
        "page"
    }
}

#[derive(Debug, Clone, Copy, Eq, Ord)]
pub struct Address<T, A: Align> {
    addr: usize,
    _phantom: PhantomData<(T, A)>,
}

impl<T, A: Align> Address<T, A> {
    pub fn new() -> Self {
        Address {
            addr: 0,
            _phantom: PhantomData,
        }
    }
    pub fn null() -> Self {
        Self::from_raw(0).unwrap()
    }
    pub fn is_null(&self) -> bool {
        self.addr == Self::null().addr
    }

    pub fn from_raw(addr: usize) -> Option<Self> {
        Some(Address {
            addr: Some(addr).filter(A::check)?,
            _phantom: PhantomData,
        })
    }
    pub fn from_ptr<U>(ptr: *const U) -> Option<Self> {
        Self::from_raw(ptr as usize)
    }

    // modifications
    pub fn increase(&mut self, units: usize) {
        self.addr = (Wrapping(self.addr) + Wrapping(units) * Wrapping(A::bytes())).0;
    }
    pub fn decrease(&mut self, units: usize) {
        self.addr = (Wrapping(self.addr) - Wrapping(units) * Wrapping(A::bytes())).0;
    }
    pub fn increase_bytes(&mut self, bytes: usize) -> Option<()> {
        if (Wrapping(self.addr) + Wrapping(bytes)).0 % A::bytes() == 0 {
            self.addr = (Wrapping(self.addr) + Wrapping(bytes)).0;
            Some(())
        } else {
            None
        }
    }
    pub fn decrease_bytes(&mut self, bytes: usize) -> Option<()> {
        if (Wrapping(self.addr) - Wrapping(bytes)).0 % A::bytes() == 0 {
            self.addr = (Wrapping(self.addr) - Wrapping(bytes)).0;
            Some(())
        } else {
            None
        }
    }

    // get next/prev address
    pub fn next(&self, units: usize) -> Self {
        let mut ret = Self::from_raw(self.addr).unwrap();
        ret.increase(units);
        ret
    }
    pub fn prev(&self, units: usize) -> Self {
        let mut ret = Self::from_raw(self.addr).unwrap();
        ret.decrease(units);
        ret
    }
    pub fn next_bytes(&self, bytes: usize) -> Option<Self> {
        let mut ret = Self::from_raw(self.addr).unwrap();
        ret.increase_bytes(bytes)?;
        Some(ret)
    }
    pub fn prev_bytes(&self, bytes: usize) -> Option<Self> {
        let mut ret = Self::from_raw(self.addr).unwrap();
        ret.decrease_bytes(bytes)?;
        Some(ret)
    }

    // convert to other alignment type
    pub fn check_aligned<B: Align>(self) -> Option<Address<T, B>> {
        Address::from_raw(self.addr)
    }
    pub fn as_ptr<U>(&self) -> *const U {
        self.addr as *const U
    }
    pub fn as_mut_ptr<U>(&self) -> *mut U {
        self.addr as *mut U
    }
    pub fn as_raw(&self) -> usize {
        self.addr
    }
    pub unsafe fn as_ref<U>(&self) -> Option<&'static U> {
        (self.addr as *const U).as_ref()
    }
}

impl<T, A: Align> PartialEq<usize> for Address<T, A> {
    fn eq(&self, other: &usize) -> bool {
        self.addr == *other
    }
}
impl<T, A: Align> PartialOrd<usize> for Address<T, A> {
    fn partial_cmp(&self, other: &usize) -> Option<core::cmp::Ordering> {
        use core::cmp::Ordering::{Equal, Greater, Less};
        if self.addr == *other {
            Some(Equal)
        } else if self.addr < *other {
            Some(Less)
        } else {
            Some(Greater)
        }
    }
}
impl<T, A: Align, B: Align> PartialEq<Address<T, B>> for Address<T, A> {
    fn eq(&self, other: &Address<T, B>) -> bool {
        self.addr == other.addr
    }
}
impl<T, A: Align, B: Align> PartialOrd<Address<T, B>> for Address<T, A> {
    fn partial_cmp(&self, other: &Address<T, B>) -> Option<core::cmp::Ordering> {
        use core::cmp::Ordering::{Equal, Greater, Less};
        if self.addr == other.addr {
            Some(Equal)
        } else if self.addr < other.addr {
            Some(Less)
        } else {
            Some(Greater)
        }
    }
}

impl<T, A: Align> fmt::Display for Address<T, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:08X}<{}>", self.as_raw(), A::display())
    }
}

// TODO to move elsewhere
const KERNBASE: usize = 0x80000000;

// convert between Virtual and Physical
impl<A: Align> Into<Option<Address<Virtual, A>>> for Address<Physical, A> {
    fn into(self) -> Option<Address<Virtual, A>> {
        Address::from_raw(self.addr + KERNBASE)
    }
}
impl<A: Align> Into<Option<Address<Physical, A>>> for Address<Virtual, A> {
    fn into(self) -> Option<Address<Physical, A>> {
        Address::from_raw(self.addr - KERNBASE)
    }
}

pub type vaddr = Address<Virtual, FreeAligned>;
pub type paddr = Address<Physical, FreeAligned>;

pub type vaddr_pg = Address<Virtual, PageAligned>;
pub type paddr_pg = Address<Physical, PageAligned>;

#[inline]
pub fn v2p<A: Align>(v: Address<Virtual, A>) -> Address<Physical, A> {
    Into::<Option<Address<Physical, A>>>::into(v).unwrap()
}
#[inline]
pub fn p2v<A: Align>(p: Address<Physical, A>) -> Address<Virtual, A> {
    Into::<Option<Address<Virtual, A>>>::into(p).unwrap()
}
#[inline]
pub fn v2p_raw(v: usize) -> usize {
    v2p(vaddr::from_raw(v).unwrap()).as_raw()
}
#[inline]
pub fn p2v_raw(p: usize) -> usize {
    p2v(paddr::from_raw(p).unwrap()).as_raw()
}

#[inline]
pub fn vaddr_raw(a: usize) -> vaddr {
    vaddr::from_raw(a).unwrap()
}
#[inline]
pub fn paddr_raw(a: usize) -> paddr {
    paddr::from_raw(a).unwrap()
}

// into FreeAligned is always successful
impl<T> From<Address<T, PageAligned>> for Address<T, FreeAligned> {
    fn from(a: Address<T, PageAligned>) -> Address<T, FreeAligned> {
        a.check_aligned().unwrap()
    }
}
