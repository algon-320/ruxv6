use super::utils;
use super::utils::address::{paddr, paddr_pg, vaddr, vaddr_pg};
use super::utils::pointer::Pointer;
use core::slice;

//------------------------------------------------------------------------------

pub const NPDENTRIES: usize = 1024; // # directory entries per page directory
pub const NPTENTRIES: usize = 1024; // # PTEs per page table
pub const PGSIZE: usize = 4096; // bytes mapped by a page

const PTXSHIFT: usize = 12; // offset of PTX in a linear address
const PDXSHIFT: usize = 22; // offset of PDX in a linear address

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
