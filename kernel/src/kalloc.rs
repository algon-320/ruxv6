use crate::spin::Mutex;
use crate::utils::address::{paddr, paddr_pg, vaddr, vaddr_pg};

#[repr(C)]
struct Run {
    next: vaddr,
}

lazy_static! {
    static ref freelist: Mutex<vaddr> = Mutex::new(vaddr::new());
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
        assert_eq!((*r).next.as_raw(), 0);
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

pub fn kfree(v: vaddr_pg) {
    let end_ptr = unsafe { &kernel_end as *const usize as usize };
    let p: Option<paddr_pg> = v.into();
    let p = p.unwrap();
    if v.as_raw() < end_ptr || p.as_raw() >= PHYSTOP {
        panic!("kfree");
    }

    let mut p = v;
    for _ in 0..PGSIZE {
        let tmp = p.as_mut_ptr::<u8>();
        unsafe {
            *tmp = 1u8;
        }
        p.increase(1);
    }

    let r = v.as_mut_ptr::<Run>();
    unsafe {
        let mut tmp = freelist.lock();
        (*r).next = *tmp;
        *tmp = v.check_aligned().unwrap();
    }
}

pub fn kalloc() -> vaddr {
    let r: *const Run;
    {
        let mut tmp = freelist.lock();
        r = tmp.as_ptr();
        unsafe {
            *tmp = (*r).next;
        }
    }
    vaddr::from_raw(r as usize).unwrap()
}
