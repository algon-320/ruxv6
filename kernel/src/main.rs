#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(global_asm)]
#![feature(asm)]
#![feature(start)]

use core::panic::PanicInfo;

global_asm!(include_str!("entry.S"));

#[allow(dead_code)]
fn print(s: &[u8], offset: usize) {
    let vga_buffer = (0xb8000 + offset * 2) as *mut u8;
    for (i, &b) in s.iter().enumerate() {
        unsafe {
            *vga_buffer.offset((i * 2 + 0) as isize) = b;
            *vga_buffer.offset((i * 2 + 1) as isize) = 0b00001010;
        }
    }
}

type pde_t = u32;

const NPDENTRIES: usize = 1024;
const PGSIZE: usize = 4096;

#[repr(C, align(4096))]
pub struct Entrypgdir {
    buf: [pde_t; NPDENTRIES],
}

#[no_mangle]
pub static entrypgdir: Entrypgdir = entrypgdir_init();

const fn entrypgdir_init() -> Entrypgdir {
    let mut buf: [pde_t; NPDENTRIES] = [0; NPDENTRIES];
    buf[0] = 0x000 | 0x001 | 0x002 | 0x080;
    buf[0x80000000 >> 22] = 0x000 | 0x001 | 0x002 | 0x080;
    Entrypgdir { buf }
}

#[no_mangle]
pub unsafe extern "C" fn main() {
    print(b"main function called !", 160);
    loop {}
}

#[panic_handler]
#[no_mangle]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[lang = "eh_personality"]
#[no_mangle]
fn eh_personality() -> ! {
    loop {}
}
