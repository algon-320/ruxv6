#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(global_asm)]
#![feature(asm)]
#![feature(start)]

//------------------------------------------------------------------------------

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate lazy_static;

extern crate spin;

//------------------------------------------------------------------------------

#[macro_use]
mod utils;
#[macro_use]
mod vga_buffer;

mod console;
mod file;
mod fs;
mod kalloc;
mod lapic;
mod mmu;
mod mp;
mod proc;
mod traps;
mod vm;
mod x86;

use utils::address::{p2v, paddr, v2p, vaddr};

//------------------------------------------------------------------------------

global_asm!(include_str!("entry.S"));

type PageDirEntry = u32;

extern "C" {
    #[no_mangle]
    static kernel_end: [u8; 0];
}

#[used]
#[no_mangle]
#[link_section = ".data.entrypgdir"]
pub static entrypgdir: [PageDirEntry; mmu::NPDENTRIES] = assigned_array![
    0; mmu::NPDENTRIES;
    // Map VA's [0, 4MB) to PA's [0, 4MB)
    [0] = 0x000 | 0x001 | 0x002 | 0x080,
    // Map VA's [KERNBASE, KERNBASE+4MB) to PA's [0, 4MB)
    [0x80000000 >> 22] = 0x000 | 0x001 | 0x002 | 0x080
    // 0x80 means the size of the page is 4MiB
];

#[no_mangle]
pub extern "C" fn main() {
    vga_buffer::VGA_WRITER.lock().clear_screen();
    println!(vga_buffer::INFO_COLOR; "main function called !");
    println!("kernel_end = {:p}", unsafe { kernel_end.as_ptr() });

    // phys page allocator
    kalloc::kinit1(
        vaddr::from_raw(unsafe { kernel_end.as_ptr() } as usize).unwrap(),
        p2v(paddr::from_raw(4 * 1024 * 1024).unwrap()),
    );
    // kernel page table
    vm::kvmalloc();

    // detect other processors
    mp::mp_init();

    // interrupt controller
    lapic::lapic_init();

    // console hardware
    console::console_init();

    unimplemented!();

    loop {}
}

use core::panic::PanicInfo;

#[panic_handler]
#[no_mangle]
fn panic(info: &PanicInfo) -> ! {
    println!(vga_buffer::ERROR_COLOR; "{}", info);
    loop {}
}

#[lang = "eh_personality"]
#[no_mangle]
fn eh_personality() -> ! {
    loop {}
}
