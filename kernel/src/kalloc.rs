use crate::spin::Mutex;
use crate::utils;
use crate::utils::address::{p2v, paddr, paddr_pg, v2p, vaddr, vaddr_pg};

#[repr(C)]
struct Run {
    next: vaddr_pg,
}

lazy_static! {
    static ref freelist: Mutex<vaddr_pg> = Mutex::new(vaddr_pg::new());
}

const PGSIZE: usize = 4096;
const PHYSTOP: usize = 0xE000000;
type Page = [u8; PGSIZE];

fn page_roundup(addr: vaddr) -> vaddr_pg {
    let a = addr.as_raw();
    vaddr_pg::from_raw((a + PGSIZE - 1) & !(PGSIZE - 1)).unwrap()
}

// Initialization happens in two phases.
// 1. main() calls kinit1() while still using entrypgdir to place just
// the pages mapped by entrypgdir on free list.
// 2. main() calls kinit2() with the rest of the physical pages
// after installing a full page table that maps them on all cores.
pub fn kinit1(start: vaddr, end: vaddr) {
    assert!(core::mem::size_of::<Page>() == PGSIZE);
    freerange(page_roundup(start), end.check_aligned().unwrap());

    // check after condition
    unsafe {
        assert_eq!(core::mem::size_of::<vaddr>(), core::mem::size_of::<usize>());

        let mut ptr = page_roundup(start);
        assert_eq!(ptr.as_raw(), 0x80106000);

        // the first page (end of freelist)
        let r = ptr.as_ptr::<Run>();
        assert!((*r).next.is_null());
        ptr.increase(PGSIZE);

        // check other pages
        while ptr.as_raw() + PGSIZE <= 0x80400000 {
            let r = ptr.as_ptr::<Run>();
            let page_begin = ptr.as_raw();
            let page_end = ptr.as_raw() + PGSIZE;

            // head of this page is pointer that point prev page
            assert_eq!((*r).next.as_raw(), page_begin - PGSIZE);

            // the rest of the page is filled with 1
            for i in (page_begin + 4)..page_end {
                assert_eq!(*(i as *const u8), 1);
            }
            ptr.increase(PGSIZE);
        }
    }
}

fn freerange(start: vaddr_pg, end: vaddr_pg) {
    println!(
        "freerange: start=0x{:X}, end=0x{:X}",
        start.as_raw(),
        end.as_raw()
    );
    let mut p = start;
    while p.as_raw() + PGSIZE <= end.as_raw() {
        kfree(p);
        p.increase(PGSIZE);
    }
}

extern "C" {
    static kernel_end: usize;
}
lazy_static! {
    static ref kernel_end_addr: usize = unsafe { &kernel_end as *const usize as usize };
}

pub fn kfree(v: vaddr_pg) {
    if v.as_raw() < *kernel_end_addr || v2p(v).as_raw() >= PHYSTOP {
        panic!("kfree");
    }

    utils::fill(
        unsafe { core::slice::from_raw_parts_mut(v.as_mut_ptr(), PGSIZE) },
        1u8,
    );

    let r = v.as_mut_ptr::<Run>();
    unsafe {
        let mut tmp = freelist.lock();
        (*r).next = *tmp;
        *tmp = v.check_aligned().unwrap();
    }
}

// return Some(address) if there is an available page, otherwise None
pub fn kalloc() -> Option<vaddr_pg> {
    let r: *const Run;
    {
        let mut tmp = freelist.lock();
        if tmp.is_null() {
            return None;
        }
        r = (*tmp).as_ptr();
        unsafe {
            *tmp = (*r).next;
        }
    }
    Some(vaddr_pg::from_raw(r as usize).unwrap())
}
