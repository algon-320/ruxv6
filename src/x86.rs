// read a byte from the port
#[inline]
pub fn inb(port: u16) -> u8 {
    let data: u8;
    unsafe {
        asm!("inb $1, $0"
            : "={al}"(data)
            : "{dx}"(port)
            : 
            : "volatile");
    }
    data
}

// read cnt double-words from the port
#[inline]
pub fn insl(port: u16, mut addr: *mut u32, mut cnt: u32) {
    unsafe {
        asm!("cld; rep insl"
            : "+{edi}"(addr), "+{ecx}"(cnt)
            : "{dx}"(port)
            : "memory", "cc"
            : "volatile");
    }
}

// write the byte (data) to the port
#[inline]
pub fn outb(port: u16, data:u8) {
    unsafe {
        asm!("outb $0, $1"
            :
            : "{al}"(data), "{dx}"(port)
            :
            : "volatile");
    }
}

// write the word (data) to the port
#[inline]
pub fn outw(port: u16, data: u16) {
    unsafe {
        asm!("outw $0, $1"
            :
            : "{ax}"(data), "{dx}"(port)
            :
            : "volatile");
    }
}

// write cnt double-words from the addr to the port
#[inline]
pub fn outsl(port: u16, mut addr: *const u32, mut cnt: u32) {
    unsafe {
        asm!("cld; rep outsl"
            : "+{esi}"(addr), "+{ecx}"(cnt)
            : "{dx}"(port)
            : "cc"
            : "volatile");
    }
}

// write the byte (data) to the address (cnt times repeatedly)
#[inline]
pub fn stosb(mut addr: *const u8, data: u8, mut cnt: u32) {
    unsafe {
        asm!("cld; rep stosb"
            : "+{edi}"(addr), "+{ecx}"(cnt)
            : "{al}"(data)
            : "memory", "cc"
            : "volatile");
    }
}

// write the double word (data) to the address (cnt times repeatedly)
#[inline]
pub fn stosl(mut addr: *const u8, data: u32, mut cnt: u32) {
    unsafe {
        asm!("cld; rep stosl"
            : "+{edi}"(addr), "+{ecx}"(cnt)
            : "{eax}"(data)
            : "memory", "cc"
            : "volatile");
    }
}

// do nothing
#[inline]
pub fn nop() {
    unsafe {
        asm!("nop");
    }
}
