#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(global_asm)]
#![feature(asm)]
#![feature(start)]

mod elf;
use elf::*;

#[allow(dead_code)]
mod x86;
use x86::*;

use core::panic::PanicInfo;

global_asm!(include_str!("bootasm.S"));

#[allow(dead_code)]
fn print(s: &[u8]) {
    let vga_buffer = 0xb8000 as *mut u8;
    for (i, &b) in s.iter().enumerate() {
        unsafe {
            *vga_buffer.offset((i * 2 + 0) as isize) = b;
            *vga_buffer.offset((i * 2 + 1) as isize) = 0b00001010;
        }
    }
}

#[allow(dead_code)]
fn print_byte(mut data: u8) {
    let mut buf = [0x30, 0x30, 0x30];
    let mut idx = 2;
    while data > 0 {
        buf[idx] = (data % 10) + 0x30;
        data /= 10;
        idx -= 1;
    }
    print(&buf);
}

#[no_mangle]
pub unsafe extern "C" fn bootmain() {
    let elf_ptr = 0x10000 as (*mut u8); // scratch space
    let elf = (elf_ptr as *mut elfhdr).as_ref().unwrap();

    // Read 1st page off disk
    readseg(elf_ptr, 4096, 0);

    // Is this an ELF executable ?
    if &elf.e_ident[0..4] != &ELF_MAGIC[..] {
        return;
    }

    // Load each program segment (ignores ph flags).
    let mut ph_ptr = elf_ptr.offset(elf.e_phoff as isize) as *const proghdr;
    let eph_ptr = ph_ptr.offset(elf.e_phnum as isize) as *const proghdr;
    while ph_ptr < eph_ptr {
        let ph = ph_ptr.as_ref().unwrap();
        let pa = ph.p_paddr;
        readseg(pa, ph.p_filesz, ph.p_offset);
        if ph.p_memsz > ph.p_filesz {
            stosb(pa.offset(ph.p_filesz as isize), 0, ph.p_memsz - ph.p_filesz);
        }
        ph_ptr = ph_ptr.offset(1);
    }

    print(b"kernel load ok !");

    // Call the entry point from the ELF header.
    // Does not return !
    (elf.e_entry)();
}

const SECTSIZE: usize = 512; // same as u32 on i386

type Sector = [u8; SECTSIZE];

fn waitdisk() {
    // Wait for disk ready
    while (inb(0x01F7) & 0xC0) != 0x40 {
        nop();
    }
}

// get lower 8bits
macro_rules! trunc8 {
    ($x:expr) => {
        ($x & 0xFF) as u8
    };
}

// Read a single sector at offset into dst.
fn readsect(dst: *mut Sector, offset: usize) {
    // Issue command.
    waitdisk();
    outb(0x01F2, 1); // count = 1
    outb(0x01F3, trunc8!(offset >> 0));
    outb(0x01F4, trunc8!(offset >> 8));
    outb(0x01F5, trunc8!(offset >> 16));
    outb(0x01F6, trunc8!(offset >> 24) | 0xE0);
    outb(0x01F7, 0x20); // cmd 0x20 - read sectors

    // Read data.
    waitdisk();
    insl(0x01F0, dst as *mut u32, SECTSIZE / 4);
}

// Read 'count' bytes at 'offset' from kernel into physical address 'pa'.
// Might copy more than asked.
unsafe fn readseg(pa: *mut u8, count: usize, offset: usize) {
    let end_pa = pa.offset(count as isize);
    let mut pa = pa.offset(-((offset % SECTSIZE) as isize)) as *mut Sector;
    let mut offset = (offset / SECTSIZE) + 1;

    while (pa as *mut u8) < end_pa {
        readsect(pa, offset);
        pa = pa.offset(1);
        offset += 1;
    }
}

#[panic_handler]
#[no_mangle]
fn panic(_info: &PanicInfo) -> ! {
    print(b"panic!");
    loop {}
}

#[lang = "eh_personality"]
#[no_mangle]
fn eh_personality() -> ! {
    loop {}
}
