#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Virtual;
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Physical;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct UnAligned;
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PageAligned;

pub trait Align {
    fn check(addr: &usize) -> bool;
}

impl Align for UnAligned {
    #[inline]
    fn check(addr: &usize) -> bool {
        true
    }
}
impl Align for PageAligned {
    #[inline]
    fn check(addr: &usize) -> bool {
        addr % 4096 == 0
    }
}

use core::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Address<T, A: Align> {
    addr: usize,
    phantom: PhantomData<(A, T)>,
}

impl<T, A: Align> Address<T, A> {
    pub fn new() -> Self {
        Address {
            addr: 0,
            phantom: PhantomData,
        }
    }
    pub fn is_null(&self) -> bool {
        self.addr == 0
    }
    pub fn from_raw(addr: usize) -> Option<Self> {
        Some(Address {
            addr: Some(addr).filter(A::check)?,
            phantom: PhantomData,
        })
    }

    // modifications
    pub fn shift(&mut self, offset: isize) {
        if offset < 0 {
            self.decrease((-offset) as usize);
        } else {
            self.increase(offset as usize);
        }
    }
    pub fn increase(&mut self, bytes: usize) {
        self.addr += bytes;
    }
    pub fn decrease(&mut self, bytes: usize) {
        self.addr -= bytes;
    }

    // convert other alignment type
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
}

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

pub type vaddr = Address<Virtual, UnAligned>;
pub type paddr = Address<Physical, UnAligned>;

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
