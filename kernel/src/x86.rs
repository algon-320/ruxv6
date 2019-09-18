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

pub fn readflags() -> EFlags {
    let mut eflags: u32 = 0;
    unsafe {
        asm!("pushfl; popl %0"
                : "=r"(eflags)
                ::: "volatile");
    }
    EFlags::from_bits(eflags).unwrap()
}

pub fn cli() {
    unsafe {
        asm!("cli"::::"volatile");
    }
}

pub fn nop() {
    unsafe {
        asm!("nop"::::"volatile");
    }
}
