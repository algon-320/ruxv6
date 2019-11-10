use super::utils;
use super::utils::address::{paddr, paddr_pg, vaddr, vaddr_pg};
use super::utils::pointer::Ptr;
use core::slice;

//------------------------------------------------------------------------------

pub mod seg {
    pub const KCODE: usize = 1; // kernel code
    pub const KDATA: usize = 2; // kernel data+stack
    pub const UCODE: usize = 3; // user code
    pub const UDATA: usize = 4; // user data+stack
    pub const TSS: usize = 5; // this process's task state
    pub const NUM: usize = 6;

    pub const STA_X: u8 = 0x8; // Executable
    pub const STA_W: u8 = 0x2; // Writable (non-executable segments)
    pub const STA_R: u8 = 0x2; // Readable     (executable segments)

    pub const DPL_USER: u8 = 0x3; // User DPL
}

pub const NPDENTRIES: usize = 1024; // # directory entries per page directory
pub const NPTENTRIES: usize = 1024; // # PTEs per page table
pub const PGSIZE: usize = 4096; // bytes mapped by a page

const PTXSHIFT: usize = 12; // offset of PTX in a linear address
const PDXSHIFT: usize = 22; // offset of PDX in a linear address

pub type Page = [u8; PGSIZE];

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct segdesc(u64);

impl segdesc {
    pub fn zero() -> Self {
        segdesc(0)
    }
    pub fn new(ty: u8, base: u32, lim: u32, dpl: u8) -> Self {
        let ty = ty as u64;
        let base = base as u64;
        let lim = lim as u64;
        let dpl = dpl as u64;
        segdesc(
            (((lim >> 12) & 0xffff) << 48)
                | ((base & 0xffff) << 32)
                | (((base >> 16) & 0xff) << 24)
                | (ty << 20)
                | (1 << 19)
                | (dpl << 17)
                | (1 << 16)
                | ((lim >> 28) << 12)
                | (0 << 11)
                | (0 << 10)
                | (1 << 9)
                | (1 << 8)
                | (base >> 24) << 0,
        )
    }
    // functions to read / write segment descriptor
}

// Task state segment format
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct taskstate {
    link: u32,        // Old ts selector
    esp0: u32,        // Stack pointers and segment selectors after an increase in privilege level
    ss0: u16,         //
    padding1: u16,    //
    esp1: *const u32, //
    ss1: u16,         //
    padding2: u16,    //
    esp2: *const u32, //
    ss2: u16,         //
    padding3: u16,    //
    cr3: usize,       // Page directory base
    epi: *const u32,  // Saved state from last task switch
    eflags: u32,      //
    eax: u32,         // More saved state (registers)
    ecx: u32,         //
    edx: u32,         //
    ebx: u32,         //
    esp: *const u32,  //
    ebp: *const u32,  //
    esi: u32,         //
    edi: u32,         //
    es: u16,          // Even more saved state (segment selectors)
    padding4: u16,    //
    cs: u16,          //
    padding5: u16,    //
    ss: u16,          //
    padding6: u16,    //
    ds: u16,          //
    padding7: u16,    //
    fs: u16,          //
    padding8: u16,    //
    gs: u16,          //
    padding9: u16,    //
    ldt: u16,         //
    padding10: u16,   //
    t: u16,           // Trap on task switch
    iomb: u16,        // I/O map base address
}
impl taskstate {
    pub fn new() -> Self {
        taskstate {
            link: 0,
            esp0: 0,
            ss0: 0,
            padding1: 0,
            esp1: core::ptr::null(),
            ss1: 0,
            padding2: 0,
            esp2: core::ptr::null(),
            ss2: 0,
            padding3: 0,
            cr3: 0,
            epi: core::ptr::null(),
            eflags: 0,
            eax: 0,
            ecx: 0,
            edx: 0,
            ebx: 0,
            esp: core::ptr::null(),
            ebp: core::ptr::null(),
            esi: 0,
            edi: 0,
            es: 0,
            padding4: 0,
            cs: 0,
            padding5: 0,
            ss: 0,
            padding6: 0,
            ds: 0,
            padding7: 0,
            fs: 0,
            padding8: 0,
            gs: 0,
            padding9: 0,
            ldt: 0,
            padding10: 0,
            t: 0,
            iomb: 0,
        }
    }
}

#[inline]
pub fn page_roundup(addr: vaddr) -> vaddr_pg {
    vaddr_pg::from_raw((addr.as_raw() + PGSIZE - 1) & !(PGSIZE - 1)).unwrap()
}
#[inline]
pub fn page_rounddown(addr: vaddr) -> vaddr_pg {
    vaddr_pg::from_raw(addr.as_raw() & !(PGSIZE - 1)).unwrap()
}

// Page table/directory entry flags.
bitflags! {
    pub struct PteFlags: u32 {
        const PRESENT = 0x001; // Present
        const WRITABLE = 0x002; // Writeable
        const USER = 0x004; // User
        const PAGE_SIZE = 0x080; // Page Size
    }
}

#[inline]
pub fn pte_addr(pte: u32) -> paddr_pg {
    paddr_pg::from_raw((pte as usize) & !(0xFFF)).unwrap()
}
#[inline]
pub fn pte_flags(pte: u32) -> PteFlags {
    PteFlags::from_bits(pte & 0xFFF).unwrap()
}

// page directory index
pub fn pdx(va: vaddr_pg) -> usize {
    (va.as_raw() >> PDXSHIFT) & 0x3FF
}
// page table index
pub fn ptx(va: vaddr_pg) -> usize {
    (va.as_raw() >> PTXSHIFT) & 0x3FF
}

// construct virtual address from indexes and offset
pub fn pgaddr(dir: usize, table: usize, offset: usize) -> vaddr_pg {
    let va = (dir << PDXSHIFT) | (table << PTXSHIFT) | offset;
    vaddr_pg::from_raw(va).unwrap()
}

pub unsafe fn fill_page(addr: vaddr_pg, byte: u8) {
    utils::fill(
        slice::from_raw_parts_mut(addr.as_mut_ptr::<u8>(), PGSIZE),
        byte,
    )
}
