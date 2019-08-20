#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(global_asm)]
#![feature(asm)]
#![feature(start)]

mod elf;
use elf::*;

mod x86;
use x86::*;

use core::panic::PanicInfo;

global_asm!(include_str!("bootasm.S"));

fn print(s: &[u8]) {
    let vga_buffer = 0xb8000 as *mut u8;
    for (i, &b) in s.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2 + 0) = b;
            *vga_buffer.offset(i as isize * 2 + 1) = 0b00001010;
        }
    }
}

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
    let elf = 0x10000 as (*mut elfhdr); // scratch space

    // Read 1st page off disk
    readseg(elf as *mut u8, 4096, 0);

    // Is this an ELF executable ?
    if &(*elf).e_ident[0..4] != &ELF_MAGIC[..] {
        return;
    }

    // Load each program segment (ignores ph flags).
    let elf_byte_ptr = elf as *const u8;

    let mut ph = elf_byte_ptr.offset((*elf).e_phoff as isize) as *const proghdr;
    let eph = ph.offset((*elf).e_phnum as isize) as *const proghdr;

    while ph < eph {
        let pa = (*ph).p_paddr as *mut u8;
        readseg(pa, (*ph).p_filesz, (*ph).p_offset);
        if (*ph).p_memsz > (*ph).p_filesz {
            stosb(
                pa.offset((*ph).p_filesz as isize),
                0,
                (*ph).p_memsz - (*ph).p_filesz,
            );
        }
        ph = ph.offset(1);
    }

    print(b"kernel load ok");

    // Call the entry point from the ELF header.
    // Does not return !
    let entry = core::mem::transmute::<u32, extern "C" fn()>((*elf).e_entry);
    entry();
}

const SECTSIZE: u32 = 512;

fn waitdisk() {
    // Wait for disk ready
    while (inb(0x01F7) & 0xC0) != 0x40 {
        nop();
    }
}

// Read a single sector at offset into dst.
fn readsect(dst: *mut u8, offset: u32) {
    // Issue command.
    waitdisk();
    outb(0x01F2, 1); // count = 1
    outb(0x01F3, ((offset >> 0) & 0xFF) as u8);
    outb(0x01F4, ((offset >> 8) & 0xFF) as u8);
    outb(0x01F5, ((offset >> 16) & 0xFF) as u8);
    outb(0x01F6, (((offset >> 24) & 0xFF) as u8) | 0xE0);
    outb(0x01F7, 0x20); // cmd 0x20 - read sectors

    // Read data.
    waitdisk();
    insl(0x01F0, dst as *mut u32, SECTSIZE / 4);
}

// Read 'count' bytes at 'offset' from kernel into physical address 'pa'.
// Might copy more than asked.
unsafe fn readseg(mut pa: *mut u8, count: u32, mut offset: u32) {
    let end_pa = pa.offset(count as isize);
    pa = pa.offset(-((offset % SECTSIZE) as isize));
    offset = (offset / SECTSIZE) + 1;

    while pa < end_pa {
        readsect(pa, offset);

        pa = pa.offset(SECTSIZE as isize);
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
