use super::param;

use spin::Mutex;

struct PipeContent {
    data: [u8; param::PIPESIZE],
    nread: usize,    // number of bytes read
    nwrite: usize,   // number of bytes written
    readopen: bool,  // read fd is still open
    writeopen: bool, // write fd is still open
}

pub struct Pipe {
    content: Mutex<PipeContent>,
}
