use super::mmu;
use super::spin::Mutex;

use super::utils;
use super::utils::address::{p2v, paddr, paddr_pg, v2p, vaddr, vaddr_pg};
use super::utils::pointer::Pointer;

//------------------------------------------------------------------------------

extern "C" {
    static kernel_end: [u8; 0];
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
struct Run {
    next: Pointer<Run>,
}

lazy_static! {
    static ref freelist: Mutex<Pointer<Run>> = Mutex::new(Pointer::null());
}

const PHYSTOP: usize = 0xE000000;
pub type Page = [u8; mmu::PGSIZE];

//------------------------------------------------------------------------------

// Initialization happens in two phases.
// 1. main() calls kinit1() while still using entrypgdir to place just
// the pages mapped by entrypgdir on free list.
// 2. main() calls kinit2() with the rest of the physical pages
// after installing a full page table that maps them on all cores.
pub fn kinit1(start: vaddr, end: vaddr) {
    freerange(mmu::page_roundup(start), end.check_aligned().unwrap());

    // check after condition
    unsafe {
        assert!(core::mem::size_of::<Page>() == mmu::PGSIZE);
        assert_eq!(core::mem::size_of::<vaddr>(), core::mem::size_of::<usize>());

        let mut ptr = mmu::page_roundup(start);

        // the first page (end of freelist)
        let r = ptr.as_ptr::<Run>();
        assert!((*r).next.is_null());
        ptr.increase(1);

        // check other pages
        while ptr.as_raw() + mmu::PGSIZE <= 0x80400000 {
            let r = ptr.as_ptr::<Run>();
            let page_begin = ptr.as_raw();
            let page_end = ptr.as_raw() + mmu::PGSIZE;

            // head of this page is pointer that point prev page
            assert_eq!((*r).next.address().as_raw(), page_begin - mmu::PGSIZE);

            // the rest of the page is filled with 1
            for i in (page_begin + 4)..page_end {
                assert_eq!(*(i as *const u8), 1);
            }
            ptr.increase(1);
        }
    }
}

fn freerange(start: vaddr_pg, end: vaddr_pg) {
    println!("freerange: start={}, end={}", start, end);
    let mut p = Pointer::<Page>::from(start);
    let mut cnt = 0;
    while p.address().as_raw() + mmu::PGSIZE <= end.as_raw() {
        kfree(p);
        p.increase(1);
        cnt += 1;
    }
    println!("{} pages available", cnt);
}

pub fn kfree(page: Pointer<Page>) {
    // TODO: give pointer alignment
    if page
        .address()
        .check_aligned::<utils::address::PageAligned>()
        .is_none()
    {
        panic!("kfree: FreeAligned");
    }

    if page.address().as_raw() < unsafe { kernel_end.as_ptr() } as usize
        || v2p(page.address()).as_raw() >= PHYSTOP
    {
        panic!("kfree");
    }

    // Fill with junk to catch dangling refs.
    unsafe {
        mmu::fill_page(page.address().check_aligned().unwrap(), 1u8);
    }

    let r = page.cast::<Run>();
    unsafe {
        let mut tmp = freelist.lock();
        (*r.get_mut()).next = *tmp;
        *tmp = r;
    }
}

// return Some(address) if there is an available page, otherwise None
pub fn kalloc() -> Option<Pointer<Page>> {
    let r: *const Run;
    {
        let mut tmp = freelist.lock();
        if tmp.is_null() {
            return None;
        }
        r = (*tmp).get();
        unsafe {
            *tmp = (*r).next;
        }
    }
    Some(Pointer::from(vaddr_pg::from_raw(r as usize).unwrap()))
}
