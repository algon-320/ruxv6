bitflags! {
    pub struct EFlags: u32 {
/* 0 */ const CF = 1 << 0;
/* 1 */ const _reserved_1 = 1 << 1;
/* 2 */ const PF = 1 << 2;
/* 3 */ const _reserved_2 = 1 << 3;
/* 4 */ const AF = 1 << 4;
/* 5 */ const _reserved_3 = 1 << 5;
/* 6 */ const ZF = 1 << 6;
/* 7 */ const SF = 1 << 7;
/* 8 */ const TF = 1 << 8;
/* 9 */ const IF = 1 << 9;
/* 10 */ const DF = 1 << 10;
/* 11 */ const OF = 1 << 11;
/* 12-13 */ const IOPL = 3 << 12;
/* 14 */ const NT = 1 << 14;
/* 15 */ const _reserved_4 = 1 << 15;
/* 16 */ const RF = 1 << 16;
/* 17 */ const VM = 1 << 17;
/* 18 */ const AC = 1 << 18;
/* 19 */ const VIF = 1 << 19;
/* 20 */ const VIP = 1 << 20;
/* 21 */ const ID = 1 << 21;
    }
}

// Layout of the trap frame built on the stack by the
// hardware and by trapasm.S, and passed to trap().
pub struct trapframe {
    // registers as pushed by pusha
    edi: u32,
    esi: u32,
    ebp: u32,
    oesp: u32,
    ebx: u32,
    edx: u32,
    ecx: u32,
    eax: u32,

    // rest of trap frame
    gs: u16,
    padding1: u16,
    fs: u16,
    padding2: u16,
    es: u16,
    padding3: u16,
    ds: u16,
    padding4: u16,
    trapno: u32,

    // bellow here defined by x86 hardware
    err: u32,
    epi: u32,
    cs: u16,
    padding5: u16,
    eflags: u32,

    // below here only when crossing rings, such as from user to kernel
    esp: u32,
    ss: u16,
    padding6: u16,
}

#[inline]
pub fn readflags() -> EFlags {
    let mut tmp: u32 = 0;
    unsafe {
        asm!("pushfl; popl $0"
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

#[inline]
pub fn lgdt(p: *mut crate::mmu::SegDesc, size: u16) {
    let pd = [
        size - 1,
        ((p as usize) & 0xffff) as u16,
        (((p as usize) >> 16) & 0xffff) as u16,
    ];
    let ptr = &pd as *const u16;
    unsafe {
        asm!("lgdt ($0)"
                :
                : "r" (ptr)
                :
                : "volatile");
    }
}
