use super::utils::address::{paddr, vaddr};

#[repr(C)]
struct mp {
    signature: [u8; 4],
    physaddr: paddr,
    length: u8,
    specrev: u8,
    checksum: u8,
    mp_type: u8,
    imcrp: u8,
    reserved: [u8; 3],
}

pub fn mpinit() {
    unimplemented!();
}
