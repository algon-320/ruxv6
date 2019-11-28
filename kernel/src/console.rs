use super::file;
use super::ioapic;
use super::traps;
use super::uart;
use super::utils::address::{p2v, paddr, vaddr};
use super::x86;

use spin::Mutex;

const NDEV: usize = 10; // maximum major device number
const INPUT_BUF: usize = 128;

struct InputBuffer {
    buf: [u8; INPUT_BUF],
    r: usize, // read index
    w: usize, // write index
    e: usize, // edit index
}
impl InputBuffer {
    fn new() -> Self {
        InputBuffer {
            buf: [0; INPUT_BUF],
            r: 0,
            w: 0,
            e: 0,
        }
    }
}

// Control-x
fn C(x: u8) -> u8 {
    x - b'@'
}

lazy_static! {
    static ref devsw: Mutex<[file::devsw; NDEV]> = Mutex::new([file::devsw::new(); NDEV]);
    static ref cons: Mutex<InputBuffer> = Mutex::new(InputBuffer::new());
}
static mut panicked: bool = false;

fn cgaputc(c: u16) {
    print!("{}", core::char::from_u32(c as u32).unwrap());
}

fn putc(c: u16) {
    if unsafe { panicked } {
        loop {}
    }
    // BACKSPACE
    if c == 0x100 {
        uart::putc(0x08); // BS
        uart::putc(0x20); // SPC
        uart::putc(0x08); // BS
    } else {
        uart::putc(c as u8);
    }
    cgaputc(c);
}

fn panic(s: &str) {
    panic!("{}", s)
}

fn console_read(inode: *const file::Inode, n: usize) -> *const [u8] {
    let _inode_content = unsafe { (*inode).content.lock() };
    let target = n;
    {
        let _cons_lock = cons.lock();
        while n > 0 {
            unimplemented!()
        }
    }
    unimplemented!()
}
fn console_write(inode: *const file::Inode, bytes: *const [u8]) {
    let _inode_content = unsafe { (*inode).content.lock() };
    {
        let _cons_lock = cons.lock();
        for c in unsafe { (*bytes).iter() } {
            putc(*c as u16);
        }
    }
}

pub fn console_init() {
    let mut tmp = devsw.lock();
    tmp[file::CONSOLE].write = Some(console_write);
    tmp[file::CONSOLE].read = Some(console_read);
    ioapic::ioapic_enable(traps::IRQ_KBD, 0);
    // unimplemented!(); // TODO implement console_read
}
