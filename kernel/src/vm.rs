use super::kalloc;
use super::mmu;
use super::x86;
use core::num::Wrapping;

use super::utils::address::{p2v, p2v_raw, paddr, paddr_pg, v2p, v2p_raw, vaddr, vaddr_pg};
use super::utils::pointer::Pointer;

type PageDirEntry = u32;
type PageTableEntry = u32;

const EXTMEM: usize = 0x100000; // Start of extended memory
const PHYSTOP: usize = 0xE000000; // Top physical memory
const DEVSPACE: usize = 0xFE000000; // Other devices are at high addresses
const KERNBASE: usize = 0x80000000; // First kernel virtual address
const KERNLINK: usize = KERNBASE + EXTMEM; // Address where kernel is linked

extern "C" {
    static data: [u8; 0];
}

lazy_static! {
    static ref kpgdir: Pointer<PageDirEntry> = setupkvm();
}

#[inline]
fn vaddr_raw(a: usize) -> vaddr {
    vaddr::from_raw(a).unwrap()
}
#[inline]
fn paddr_raw(a: usize) -> paddr {
    paddr::from_raw(a).unwrap()
}

//------------------------------------------------------------------------------

// Return the address of the PTE in page table pgdir
// that corresponds to virtual address va.  If alloc!=0,
// create any required page table pages.
fn walkpgdir(pgdir: Pointer<PageDirEntry>, va: vaddr_pg, alloc: bool) -> Pointer<PageTableEntry> {
    let pde = {
        let mut tmp = pgdir;
        tmp.increase(mmu::pdx(va));
        tmp
    };
    let mut pgtab: Pointer<PageTableEntry>;
    let ent = unsafe { *pde.get() };
    if ent & mmu::PteFlags::PRESENT.bits() != 0 {
        pgtab = Pointer::from(p2v(mmu::pte_addr(ent)));
    } else {
        if !alloc {
            return Pointer::null();
        }
        if let Some(ptr) = kalloc::kalloc() {
            pgtab = ptr.cast();
        } else {
            println!("walkpgdir: fail to kalloc");
            return Pointer::null();
        }

        // Make sure all those PTE_P bits are zero.
        unsafe {
            mmu::fill_page(pgtab.address().check_aligned().unwrap(), 0);
        }

        // The permissions here are overly generous, but they can
        // be further restricted by the permissions in
        // the page table entries, if necessary.
        unsafe {
            *pde.get_mut() = v2p(pgtab.address()).as_raw() as u32
                | (mmu::PteFlags::PRESENT | mmu::PteFlags::WRITABLE | mmu::PteFlags::USER).bits();
        }
        assert!(unsafe { *pde.get() } & mmu::PteFlags::PRESENT.bits() != 0);
    }

    pgtab.increase(mmu::ptx(va));
    pgtab
}

// Create PTEs for virtual addresses starting at va that refer to
// physical addresses starting at pa. va and size might not
// be page-aligned.
fn mappages(
    pgdir: Pointer<PageDirEntry>,
    va: vaddr,
    size: usize,
    mut pa: paddr_pg,
    perm: mmu::PteFlags,
) -> Option<()> {
    let mut a = mmu::page_rounddown(va);
    let last = {
        let mut tmp = va;
        tmp.increase(size - 1);
        mmu::page_rounddown(tmp)
    };
    let mut cnt = 0;
    loop {
        let pte = walkpgdir(pgdir, a, true);
        if pte.is_null() {
            return None;
        }
        if unsafe { *pte.get() } & mmu::PteFlags::PRESENT.bits() != 0 {
            panic!("remap");
        }
        unsafe {
            *pte.get_mut() = (pa.as_raw() as u32) | (perm | mmu::PteFlags::PRESENT).bits();
        }
        if a.as_raw() == last.as_raw() {
            break;
        }
        a.increase(1);
        pa.increase(1);
        cnt += 1;
    }
    println!("mappages: cnt = {}", cnt);
    Some(())
}

struct Kmap {
    virt: vaddr,
    start: paddr_pg,
    end: paddr_pg,
    perm: mmu::PteFlags,
}
lazy_static! {
    static ref kmap: [Kmap; 4] = [
        Kmap { // I/O space
            virt: vaddr_raw(KERNBASE),
            start: paddr_pg::from_raw(0).unwrap(),
            end: paddr_pg::from_raw(EXTMEM).unwrap(),
            perm: mmu::PteFlags::WRITABLE,
        },
        Kmap { // kern text+rodata
            virt: vaddr_raw(KERNLINK),
            start: v2p(vaddr_pg::from_raw(KERNLINK).unwrap()),
            end: v2p(vaddr_pg::from_raw(unsafe{data.as_ptr()} as usize).unwrap()),
            perm: mmu::PteFlags::empty(),
        },
        Kmap { // kern data+memory
            virt: vaddr_raw(unsafe{data.as_ptr()} as usize),
            start: v2p(vaddr_pg::from_raw(unsafe{data.as_ptr()} as usize).unwrap()),
            end: paddr_pg::from_raw(PHYSTOP).unwrap(),
            perm: mmu::PteFlags::WRITABLE,
        },
        Kmap { // more devices
            virt: vaddr_raw(DEVSPACE),
            start: paddr_pg::from_raw(DEVSPACE).unwrap(),
            end: paddr_pg::from_raw(0).unwrap(),  // end of address
            perm: mmu::PteFlags::WRITABLE,
        },
    ];
}

// Set up kernel part of a page table.
fn setupkvm() -> Pointer<PageDirEntry> {
    let pgdir: Pointer<PageDirEntry> = kalloc::kalloc().unwrap().cast();
    println!("setupkvm: pgdir = {}", pgdir);
    unsafe {
        mmu::fill_page(pgdir.address().check_aligned().unwrap(), 0);
    }

    if p2v_raw(PHYSTOP) > DEVSPACE {
        panic!("PHYSTOP too hight");
    }

    for k in kmap.iter() {
        println!(
            "mappages: virt: {}, start: {}, end: {}, size: 0x{:X}",
            k.virt,
            k.start,
            k.end,
            (Wrapping(k.end.as_raw()) - Wrapping(k.start.as_raw())).0,
        );
        if mappages(
            pgdir,
            k.virt,
            (Wrapping(k.end.as_raw()) - Wrapping(k.start.as_raw())).0,
            k.start,
            k.perm,
        )
        .is_none()
        {
            freevm(pgdir);
            return Pointer::null();
        }
    }
    pgdir
}

// Allocate one page table for the machine for the kernel address
// space for scheduler processes.
pub fn kvmalloc() {
    lazy_static::initialize(&kpgdir);

    // assertions
    unsafe {
        let va = vaddr_pg::from_raw(KERNBASE).unwrap();
        let tmp = walkpgdir(*kpgdir, va, false);
        assert!(!tmp.is_null());
        assert_eq!(mmu::pte_addr(*tmp.get()), paddr_pg::from_raw(0).unwrap());

        let va = vaddr_pg::from_raw(KERNLINK).unwrap();
        let tmp = walkpgdir(*kpgdir, va, false);
        assert!(!tmp.is_null());
        assert_eq!(
            mmu::pte_addr(*tmp.get()),
            v2p(vaddr_pg::from_raw(KERNLINK).unwrap())
        );

        let va = vaddr_pg::from_raw(unsafe { data.as_ptr() } as usize).unwrap();
        let tmp = walkpgdir(*kpgdir, va, false);
        assert!(!tmp.is_null());
        assert_eq!(
            mmu::pte_addr(*tmp.get()),
            v2p(vaddr_pg::from_raw(unsafe { data.as_ptr() } as usize).unwrap())
        );

        let va = vaddr_pg::from_raw(DEVSPACE).unwrap();
        let tmp = walkpgdir(*kpgdir, va, false);
        assert!(!tmp.is_null());
        assert_eq!(
            mmu::pte_addr(*tmp.get()),
            paddr_pg::from_raw(DEVSPACE).unwrap()
        );

        let va = vaddr_pg::from_raw(0x80101000).unwrap();
        let tmp = walkpgdir(*kpgdir, va, false);
        assert!(!tmp.is_null());
        assert_eq!(
            mmu::pte_addr(*tmp.get()),
            paddr_pg::from_raw(0x101000).unwrap()
        );
    }

    switchkvm();
}

// Switch h/w page table register to the kernel-only page table,
// for when no process is running.
fn switchkvm() {
    x86::lcr3(v2p(kpgdir.address()).as_raw()); // switch to the kernel page table
}

// Deallocate user pages to bring the process size from old_sz to
// new_sz.  old_sz and new_sz need not be page-aligned, nor does new_sz
// need to be less than old_sz.  old_sz can be larger than the actual
// process size.  Returns the new process size.
fn deallocuvm(pgdir: Pointer<PageDirEntry>, old_sz: usize, new_sz: usize) -> usize {
    if new_sz >= old_sz {
        return old_sz;
    }

    let mut a = mmu::page_roundup(vaddr_raw(new_sz));
    while a.as_raw() < old_sz {
        let pte = walkpgdir(pgdir, a, false);
        if pte.is_null() {
            a = {
                let mut tmp = mmu::pgaddr(mmu::pdx(a) + 1, 0, 0);
                tmp.decrease(1);
                tmp
            };
        } else {
            let ent = unsafe { *pte.get() };
            if ent & mmu::PteFlags::PRESENT.bits() != 0 {
                let pa = mmu::pte_addr(ent);
                if pa.is_null() {
                    println!("a = {}, pte = {}, pa = {}", a, pte, pa);
                    panic!("deallocuvm: kfree");
                }
                kalloc::kfree(Pointer::from(p2v(pa)));
                unsafe {
                    *pte.get_mut() = 0;
                }
            }
        }
        a.increase(1);
    }
    new_sz
}

// Free a page table and all the physical memory pages
// in the user part.
fn freevm(pgdir: Pointer<PageDirEntry>) {
    if pgdir.is_null() {
        panic!("freevm: no pgdir");
    }
    deallocuvm(pgdir, KERNBASE, 0);
    let mut p = pgdir;
    for _ in 0..mmu::NPDENTRIES {
        let tmp = unsafe { *p.get() };
        if tmp & mmu::PteFlags::PRESENT.bits() != 0 {
            let v = p2v(mmu::pte_addr(tmp));
            kalloc::kfree(Pointer::from(v));
        }
        p.increase(1);
    }
    kalloc::kfree(pgdir.cast());
}
