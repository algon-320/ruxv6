#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(global_asm)]
#![feature(asm)]
#![feature(start)]

use core::fmt::{self, Write};

struct Wrapper<'a> {
    buf: &'a mut [u8],
    offset: usize,
}

impl<'a> Wrapper<'a> {
    fn new(buf: &'a mut [u8]) -> Self {
        Wrapper {
            buf: buf,
            offset: 0,
        }
    }
}

impl<'a> fmt::Write for Wrapper<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();

        // Skip over already-copied data
        let remainder = &mut self.buf[self.offset..];
        // Check if there is space remaining (return error instead of panicking)
        if remainder.len() < bytes.len() {
            return Err(core::fmt::Error);
        }
        // Make the two slices the same length
        let remainder = &mut remainder[..bytes.len()];
        // Copy
        remainder.copy_from_slice(bytes);

        // Update offset to avoid overwriting
        self.offset += bytes.len();

        Ok(())
    }
}

#[macro_use]
extern crate bitflags;

bitflags! {
    pub struct BitFlagTest: u32 {
        const hoge = 0x1;
        const fuga = 0x2;
    }
}

use core::panic::PanicInfo;

// mod console;
// mod kalloc;
// mod proc;
// mod spinlock;
// mod x86;

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

// similar to C99's designated initializer
macro_rules! asigned_array {
    ($def:expr; $len:expr; $([$idx:expr] = $val:expr),*) => {{
        let mut tmp = [$def; $len];
        $(tmp[$idx] = $val;)*
        tmp
    }};
}

#[used]
#[no_mangle]
#[link_section = ".data.entrypgdir"]
pub static entrypgdir: [pde_t; NPDENTRIES] = asigned_array![0; NPDENTRIES;
    // Map VA's [0, 4MB) to PA's [0, 4MB)
    [0] = 0x000 | 0x001 | 0x002 | 0x080,
    // Map VA's [KERNBASE, KERNBASE+4MB) to PA's [0, 4MB)
    [0x80000000 >> 22] = 0x000 | 0x001 | 0x002 | 0x080
];

#[no_mangle]
pub unsafe extern "C" fn main() {
    print(b"main function called !", 160);
    let bit = BitFlagTest::hoge | BitFlagTest::fuga;
    let mut buffer = [0u8; 80];
    write!(Wrapper::new(&mut buffer[..]), "{:08b}", bit.bits());
    print(&buffer, 240);
    loop {}
}

#[panic_handler]
#[no_mangle]
fn panic(info: &PanicInfo) -> ! {
    // console::console.get().print("TODO implement panic");
    loop {}
}

#[lang = "eh_personality"]
#[no_mangle]
fn eh_personality() -> ! {
    loop {}
}
