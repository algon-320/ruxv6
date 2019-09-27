use super::spin::Mutex;

use super::mmu;
use super::mmu::Page;
use super::utils;
use super::utils::address::{p2v, paddr, paddr_pg, v2p, vaddr, vaddr_pg};
use super::utils::pointer::Ptr;
//------------------------------------------------------------------------------

extern "C" {
    static kernel_end: [u8; 0];
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
struct Run {
    next: Ptr<Run>,
}

lazy_static! {
    static ref freelist: Mutex<Ptr<Run>> = Mutex::new(Ptr::null());
}

const PHYSTOP: usize = 0xE000000;

//------------------------------------------------------------------------------

// Initialization happens in two phases.
// 1. main() calls kinit1() while still using entrypgdir to place just
// the pages mapped by entrypgdir on free list.
// 2. main() calls kinit2() with the rest of the physical pages
// after installing a full page table that maps them on all cores.
pub fn kinit1(start: vaddr, end: vaddr) {
    freerange(mmu::page_roundup(start), end.check_aligned().unwrap());

    // check some conditions
    unsafe {
        use core::mem::size_of;
        assert!(size_of::<Page>() == mmu::PGSIZE);
        assert_eq!(size_of::<vaddr>(), size_of::<usize>());

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
    let mut p = Ptr::<Page>::from(start);
    let mut num_pages = 0;
    while p.address().next(mmu::PGSIZE) <= end {
        kfree(&mut *p);
        p.increase(1);
        num_pages += 1;
    }
    println!("{} pages available", num_pages);
}

pub fn kfree<'a>(page: &'a mut Page) {
    if page.as_ptr() < unsafe { kernel_end.as_ptr() }
        || v2p(vaddr::from_ptr(page.as_ptr()).unwrap()) >= PHYSTOP
    {
        panic!("kfree");
    }

    let page: Ptr<Page> = Ptr::from(page.as_ptr() as *const Page);

    // Fill with junk to catch dangling refs.
    unsafe {
        mmu::fill_page(page.address().check_aligned().unwrap(), 1u8);
    }

    let mut r = page.cast::<Run>();
    {
        let mut tmp = freelist.lock();
        (*r).next = *tmp;
        *tmp = r;
    }
}

// return Some(address) if there is an available page, otherwise None
pub fn kalloc<'a>() -> Option<&'a mut Page> {
    let r: Ptr<Run>;
    {
        let mut tmp = freelist.lock();
        if tmp.is_null() {
            return None;
        }
        r = *tmp;
        *tmp = (*r).next;
    }
    unsafe { r.cast::<Page>().get_mut().as_mut() }
}
