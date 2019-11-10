use core::num::Wrapping;

use super::kalloc;
use super::mmu;
use super::mp;
use super::proc;
use super::utils;
use super::utils::address::{
    p2v, p2v_raw, paddr, paddr_pg, paddr_raw, v2p, v2p_raw, vaddr, vaddr_pg, vaddr_raw,
};
use super::utils::pointer::Ptr;
use super::x86;

pub type PageDirEntry = u32;
pub type PageTableEntry = u32;

const EXTMEM: usize = 0x100000; // Start of extended memory
const PHYSTOP: usize = 0xE000000; // Top physical memory
const DEVSPACE: usize = 0xFE000000; // Other devices are at high addresses
const KERNBASE: usize = 0x80000000; // First kernel virtual address
const KERNLINK: usize = KERNBASE + EXTMEM; // Address where kernel is linked

extern "C" {
    static data: [u8; 0];
}

lazy_static! {
    static ref kpgdir: Option<&'static [PageDirEntry; mmu::NPDENTRIES]> = setupkvm();
}

//------------------------------------------------------------------------------

// Set up CPU's kernel segment descriptors.
// Run once on entry on each CPU.
pub fn seg_init() {
    let idx = proc::mycpu().cpuid();
    let c = unsafe { mp::CPU_ARRAY.borrow_mut(idx) };

    // Map "logical" addresses to virtual addresses using identity map.
    // Cannot share a CODE descriptor for both kernel and user
    // because it would have to have DPL_USR, but the CPU forbids
    // an interrupt from CPL=0 to DPL=3.
    {
        use mmu::seg::*;
        c.gdt[KCODE] = mmu::segdesc::new(STA_X | STA_R, 0, 0xffffffff, 0);
        c.gdt[KDATA] = mmu::segdesc::new(STA_W, 0, 0xffffffff, 0);
        c.gdt[UCODE] = mmu::segdesc::new(STA_X | STA_R, 0, 0xffffffff, DPL_USER);
        c.gdt[UDATA] = mmu::segdesc::new(STA_W, 0, 0xffffffff, DPL_USER);
        x86::lgdt(
            &mut c.gdt[0] as *mut mmu::segdesc,
            core::mem::size_of_val(&c.gdt) as u16,
        );
    }

    println!("mycpu: {:?}", c);
}

// Return the address of the PTE in page table pgdir
// that corresponds to virtual address va.  If alloc!=0,
// create any required page table pages.
fn walkpgdir(
    pgdir: &mut [PageDirEntry; mmu::NPDENTRIES],
    va: vaddr_pg,
    alloc: bool,
) -> Option<&mut PageTableEntry> {
    let pde = &mut pgdir[mmu::pdx(va)];
    let pgtab: &mut [PageTableEntry; mmu::NPTENTRIES];
    if *pde & mmu::PteFlags::PRESENT.bits() != 0 {
        pgtab = unsafe {
            let tmp: *mut [PageTableEntry; mmu::NPTENTRIES] = p2v(mmu::pte_addr(*pde)).as_mut_ptr();
            tmp.as_mut().unwrap()
        };
    } else {
        if !alloc {
            return None;
        }
        if let Some(slice) = kalloc::kalloc() {
            pgtab = unsafe {
                let tmp = slice.as_mut_ptr() as *mut [PageTableEntry; mmu::NPTENTRIES];
                tmp.as_mut().unwrap()
            };
        } else {
            println!("walkpgdir: fail to kalloc");
            return None;
        }

        // Make sure all those PTE_P bits are zero.
        utils::fill(pgtab, 0x00000000);

        // The permissions here are overly generous, but they can
        // be further restricted by the permissions in
        // the page table entries, if necessary.
        *pde = v2p(vaddr::from_ptr(pgtab.as_ptr()).unwrap()).as_raw() as u32
            | (mmu::PteFlags::PRESENT | mmu::PteFlags::WRITABLE | mmu::PteFlags::USER).bits();
        assert!(*pde & mmu::PteFlags::PRESENT.bits() != 0);
    }

    Some(&mut pgtab[mmu::ptx(va)])
}

fn walkpgdir_lookup(
    pgdir: &[PageDirEntry; mmu::NPDENTRIES],
    va: vaddr_pg,
) -> Option<&PageTableEntry> {
    let pde = pgdir[mmu::pdx(va)];
    if pde & mmu::PteFlags::PRESENT.bits() != 0 {
        let pgtab = unsafe {
            let tmp: *const [PageTableEntry; mmu::NPTENTRIES] = p2v(mmu::pte_addr(pde)).as_ptr();
            tmp.as_ref().unwrap()
        };
        Some(&pgtab[mmu::ptx(va)])
    } else {
        None
    }
}

// Create PTEs for virtual addresses starting at va that refer to
// physical addresses starting at pa. va and size might not
// be page-aligned.
fn mappages(
    pgdir: &mut [PageDirEntry; mmu::NPDENTRIES],
    va: vaddr,
    size: usize,
    mut pa: paddr_pg,
    perm: mmu::PteFlags,
) -> Option<()> {
    println!(
        "mappages: virt: {}, start: {}, size: 0x{:08X}",
        va, pa, size
    );

    let mut a = mmu::page_rounddown(va);
    let last = mmu::page_rounddown(va.next(size - 1));
    loop {
        let pte = walkpgdir(pgdir, a, true)?;
        if *pte & mmu::PteFlags::PRESENT.bits() != 0 {
            panic!("remap");
        }
        *pte = (pa.as_raw() as u32) | (perm | mmu::PteFlags::PRESENT).bits();
        if a == last {
            break;
        }
        a.increase(1);
        pa.increase(1);
    }
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
            end: v2p(vaddr_pg::from_ptr(unsafe{data.as_ptr()}).unwrap()),
            perm: mmu::PteFlags::empty(),
        },
        Kmap { // kern data+memory
            virt: vaddr_raw(unsafe{data.as_ptr()} as usize),
            start: v2p(vaddr_pg::from_ptr(unsafe{data.as_ptr()}).unwrap()),
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
fn setupkvm() -> Option<&'static [PageDirEntry; mmu::NPDENTRIES]> {
    let pgdir = unsafe {
        let page = kalloc::kalloc().unwrap();
        let tmp = page.as_mut_ptr() as *mut [PageTableEntry; mmu::NPTENTRIES];
        tmp.as_mut().unwrap()
    };
    utils::fill(pgdir, 0x00000000);

    println!("setupkvm: pgdir = {:p}", pgdir.as_ptr());

    if p2v_raw(PHYSTOP) > DEVSPACE {
        panic!("PHYSTOP too hight");
    }

    for k in kmap.iter() {
        if mappages(
            pgdir,
            k.virt,
            (Wrapping(k.end.as_raw()) - Wrapping(k.start.as_raw())).0,
            k.start,
            k.perm,
        )
        .is_none()
        {
            freevm(Some(pgdir));
            return None;
        }
    }
    Some(pgdir)
}

// Allocate one page table for the machine for the kernel address
// space for scheduler processes.
pub fn kvmalloc() {
    lazy_static::initialize(&kpgdir);

    // check mappings
    unsafe {
        {
            let va = vaddr_pg::from_raw(KERNBASE).unwrap();
            let tmp = walkpgdir_lookup(kpgdir.unwrap(), va);
            assert!(!tmp.is_none());
            let tmp = tmp.unwrap();
            assert_eq!(mmu::pte_addr(*tmp), paddr_pg::from_raw(0).unwrap());
        }

        {
            let va = vaddr_pg::from_raw(KERNLINK).unwrap();
            let tmp = walkpgdir_lookup(kpgdir.unwrap(), va);
            assert!(!tmp.is_none());
            let tmp = tmp.unwrap();
            assert_eq!(
                mmu::pte_addr(*tmp),
                v2p(vaddr_pg::from_raw(KERNLINK).unwrap())
            );
        }

        {
            let va = vaddr_pg::from_raw(data.as_ptr() as usize).unwrap();
            let tmp = walkpgdir_lookup(kpgdir.unwrap(), va);
            assert!(!tmp.is_none());
            let tmp = tmp.unwrap();
            assert_eq!(
                mmu::pte_addr(*tmp),
                v2p(vaddr_pg::from_raw(data.as_ptr() as usize).unwrap())
            );
        }

        {
            let va = vaddr_pg::from_raw(DEVSPACE).unwrap();
            let tmp = walkpgdir_lookup(kpgdir.unwrap(), va);
            assert!(!tmp.is_none());
            let tmp = tmp.unwrap();
            assert_eq!(mmu::pte_addr(*tmp), paddr_pg::from_raw(DEVSPACE).unwrap());
        }

        {
            let va = vaddr_pg::from_raw(0x80101000).unwrap();
            let tmp = walkpgdir_lookup(kpgdir.unwrap(), va);
            assert!(!tmp.is_none());
            let tmp = tmp.unwrap();
            assert_eq!(mmu::pte_addr(*tmp), paddr_pg::from_raw(0x101000).unwrap());
        }
    }

    switchkvm();
}

// Switch h/w page table register to the kernel-only page table,
// for when no process is running.
fn switchkvm() {
    let p = v2p(vaddr::from_ptr(kpgdir.unwrap().as_ptr()).unwrap());
    x86::lcr3(p.as_raw()); // switch to the kernel page table
}

// Deallocate user pages to bring the process size from old_sz to
// new_sz.  old_sz and new_sz need not be page-aligned, nor does new_sz
// need to be less than old_sz.  old_sz can be larger than the actual
// process size.  Returns the new process size.
fn deallocuvm(pgdir: &mut [PageDirEntry; mmu::NPDENTRIES], old_sz: usize, new_sz: usize) -> usize {
    if new_sz >= old_sz {
        return old_sz;
    }

    let mut a = mmu::page_roundup(vaddr_raw(new_sz));
    while a < old_sz {
        let pte = walkpgdir(pgdir, a, false);
        if pte.is_none() {
            a = mmu::pgaddr(mmu::pdx(a) + 1, 0, 0).prev(1);
        } else {
            let pte = pte.unwrap();
            if *pte & mmu::PteFlags::PRESENT.bits() != 0 {
                let pa = mmu::pte_addr(*pte);
                if pa.is_null() {
                    println!("a = {}, pte = {}, pa = {}", a, pte, pa);
                    panic!("deallocuvm: kfree");
                }
                let ptr: *mut mmu::Page = p2v(pa).as_mut_ptr();
                kalloc::kfree(unsafe { ptr.as_mut().unwrap() });
                *pte = 0;
            }
        }
        a.increase(1);
    }
    new_sz
}

// Free a page table and all the physical memory pages
// in the user part.
fn freevm(pgdir: Option<&mut [PageDirEntry; mmu::NPDENTRIES]>) {
    if pgdir.is_none() {
        panic!("freevm: no pgdir");
    }
    let pgdir = pgdir.unwrap();
    deallocuvm(pgdir, KERNBASE, 0);
    for dent in pgdir.into_iter() {
        if dent & mmu::PteFlags::PRESENT.bits() != 0 {
            let table_ptr: *mut mmu::Page = p2v(mmu::pte_addr(*dent)).as_mut_ptr();
            kalloc::kfree(unsafe { table_ptr.as_mut().unwrap() });
        }
    }
    let ptr = pgdir.as_mut_ptr() as *mut mmu::Page;
    kalloc::kfree(unsafe { ptr.as_mut().unwrap() });
}
