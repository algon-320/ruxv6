use super::fs;

use spin::Mutex;

// Copy of disk inode
#[repr(C)]
pub struct InodeContent {
    file_type: i16,
    major: i16,
    minor: i16,
    nlink: i16,
    size: usize,
    addrs: [usize; fs::NDIRECT + 1],
}

// in-memory copy of an inode
pub struct Inode {
    dev: u32,
    inum: u32,
    refcnt: i32,
    valid: bool,
    content: Mutex<InodeContent>,
}

// table mapping major device number to
// device functions
#[derive(Debug, Clone, Copy)]
pub struct devsw {
    pub read: Option<fn(*const Inode) -> *const [u8]>,
    pub write: Option<fn(*const Inode, *const [u8])>,
}

impl devsw {
    pub fn new() -> Self {
        devsw {
            read: None,
            write: None,
        }
    }
}

pub const CONSOLE: usize = 1;
