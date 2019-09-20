#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(global_asm)]
#![feature(asm)]
#![feature(start)]

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate lazy_static;

extern crate spin;

#[macro_use]
mod utils;
use utils::address::{paddr, vaddr};

#[macro_use]
mod vga_buffer;

// mod console;
mod kalloc;
// mod proc;
// mod spinlock;
mod x86;

//--------------------------------------------------------

global_asm!(include_str!("entry.S"));

type pde_t = u32;

const NPDENTRIES: usize = 1024;
const PGSIZE: usize = 4096;

#[used]
#[no_mangle]
#[link_section = ".data.entrypgdir"]
pub static entrypgdir: [pde_t; NPDENTRIES] = asigned_array![0; NPDENTRIES;
    // Map VA's [0, 4MB) to PA's [0, 4MB)
    [0] = 0x000 | 0x001 | 0x002 | 0x080,
    // Map VA's [KERNBASE, KERNBASE+4MB) to PA's [0, 4MB)
    [0x80000000 >> 22] = 0x000 | 0x001 | 0x002 | 0x080
];

extern "C" {
    #[no_mangle]
    static kernel_end: usize;
}

#[no_mangle]
pub unsafe extern "C" fn main() {
    vga_buffer::VGA_WRITER.lock().clear_screen();
    println!("main function called !");
    println!("kernel_end = {}", &kernel_end as *const usize as usize);

    let end_phys = paddr::from_raw(4 * 1024 * 1024).unwrap();
    let end_virt: Option<vaddr> = end_phys.into();
    kalloc::kinit1(
        vaddr::from_raw(&kernel_end as *const usize as usize).unwrap(),
        end_virt.unwrap(),
    ); // phys page allocator

    unimplemented!();

    loop {}
}

use core::panic::PanicInfo;

#[panic_handler]
#[no_mangle]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[lang = "eh_personality"]
#[no_mangle]
fn eh_personality() -> ! {
    loop {}
}
