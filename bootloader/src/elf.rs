// elf32 header
#[repr(C)]
pub struct elfhdr {
    pub e_ident: [u8; 16],              // ELF Identification
    pub e_type: u16,                    // object file type
    pub e_machine: u16,                 // machine
    pub e_version: u32,                 // object file version
    pub e_entry: extern "C" fn() -> (), // virtual entry point
    pub e_phoff: usize,                 // program header table offset
    pub e_shoff: usize,                 // section header table offset
    pub e_flags: u32,                   // processor-specific flags
    pub e_ehsize: u16,                  // ELF header size
    pub e_phentsize: u16,               // program header entry size
    pub e_phnum: u16,                   // number of program header entries
    pub e_shent_size: u16,              // section header entry size
    pub e_shnum: u16,                   // number of section header entries
    pub e_shstrndx: u16,                // section header tables's
                                        //   "section header string table" entry offset
}

// elf32 program header
#[repr(C)]
pub struct proghdr {
    pub p_type: u32,      // segment type
    pub p_offset: usize,  // segment offset
    pub p_vaddr: *mut u8, // virtual address of segment
    pub p_paddr: *mut u8, // physical address - ignored ?
    pub p_filesz: usize,  // number of bytes in file for seg.
    pub p_memsz: usize,   // number of bytes in mem. for seg.
    pub p_flags: u32,     // flags
    pub p_align: usize,   // memory alignment
}

pub const ELF_MAGIC: [u8; 4] = [0x7F, 0x45, 0x4C, 0x46]; // 0x7F, 'E', 'L', 'F'
