use super::file;
use super::x86;

use spin::Mutex;

const NDEV: usize = 10; // maximum major device number

lazy_static! {
    static ref devsw: Mutex<[file::devsw; NDEV]> = Mutex::new([file::devsw::new(); NDEV]);
}

fn console_read(inode: *const file::Inode) -> *const [u8] {
    println!(crate::vga_buffer::INFO_COLOR; "console_read");
    unimplemented!()
}
fn console_write(inode: *const file::Inode, bytes: *const [u8]) {
    println!(crate::vga_buffer::INFO_COLOR; "console_write");
    unimplemented!()
}

pub fn console_init() {
    let mut tmp = devsw.lock();
    tmp[file::CONSOLE].write = Some(console_write);
    tmp[file::CONSOLE].read = Some(console_read);
}
