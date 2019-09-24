bitflags! {
    pub struct EFlags: u32 {
/* 0 */ const CF = 0b00000001u32;
/* 2 */ const PF = 0b00000100u32;
/* 4 */ const AF = 0b00010000u32;
/* 6 */ const ZF = 0b01000000u32;
/* 7 */ const SF = 0b10000000u32;
/* 8 */ const TF = 0b00000001u32 << 8;
/* 9 */ const IF = 0b00000010u32 << 8;
/* 10 */ const DF = 0b00000100u32 << 8;
/* 11 */ const OF = 0b00001000u32 << 8;
/* 12-13 */ const IOPL = 0b00110000u32 << 8;
/* 14 */ const NT = 0b01000000u32 << 8;
/* 16 */ const RF = 0b00000001u32 << 16;
/* 17 */ const VM = 0b00000010u32 << 16;
/* 18 */ const AC = 0b00000100u32 << 16;
/* 19 */ const VIF = 0b00001000u32 << 16;
/* 20 */ const VIP = 0b00010000u32 << 16;
/* 21 */ const ID = 0b00100000u32 << 16;
    }
}

#[inline]
pub fn readflags() -> EFlags {
    let mut tmp: u32 = 0;
    unsafe {
        asm!("pushfl; popl %0"
                : "=r"(tmp)
                ::: "volatile");
    }
    EFlags::from_bits(tmp).unwrap()
}

#[inline]
pub fn cli() {
    unsafe {
        asm!("cli"::::"volatile");
    }
}

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
pub fn insl(port: u16, addr: *mut u32, cnt: usize) {
    let mut _addr = addr;
    let mut _cnt = cnt;
    unsafe {
        asm!("cld; rep insl"
            : "+{edi}"(_addr), "+{ecx}"(_cnt)
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
pub fn outsl(port: u16, addr: *const u32, cnt: usize) {
    let mut _addr = addr;
    let mut _cnt = cnt;
    unsafe {
        asm!("cld; rep outsl"
            : "+{esi}"(_addr), "+{ecx}"(_cnt)
            : "{dx}"(port)
            : "cc"
            : "volatile");
    }
}

// write the byte (data) to the address (cnt times repeatedly)
#[inline]
pub fn stosb(addr: *const u8, data: u8, cnt: usize) {
    let mut _addr = addr;
    let mut _cnt = cnt;
    unsafe {
        asm!("cld; rep stosb"
            : "+{edi}"(_addr), "+{ecx}"(_cnt)
            : "{al}"(data)
            : "memory", "cc"
            : "volatile");
    }
}

// write the double word (data) to the address (cnt times repeatedly)
#[inline]
pub fn stosl(addr: *const u8, data: u32, cnt: usize) {
    let mut _addr = addr;
    let mut _cnt = cnt;
    unsafe {
        asm!("cld; rep stosl"
            : "+{edi}"(_addr), "+{ecx}"(_cnt)
            : "{eax}"(data)
            : "memory", "cc"
            : "volatile");
    }
}

// do nothing
#[inline]
pub fn nop() {
    unsafe {
        asm!("nop" :::: "volatile");
    }
}

#[inline]
pub fn xchg(addr: &mut usize, new_val: usize) -> usize {
    let mut result;
    unsafe {
        asm!("lock; xchgl $0, $1"
                : "+m" (addr as *mut usize), "={eax}" (result)
                : "1" (new_val)
                : "cc"
                : "volatile");
    }
    result
}

#[inline]
pub fn rcr2() -> usize {
    let mut val;
    unsafe {
        asm!("movl %cr2, $0"
                : "=r" (val)
                :
                :
                : "volatile");
    }
    val
}

#[inline]
pub fn lcr3(val: usize) {
    unsafe {
        asm!("movl $0, %cr3"
                :
                : "r" (val)
                :
                : "volatile");
    }
}
