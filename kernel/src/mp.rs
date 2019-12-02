use super::lapic;
use super::param;
use super::proc::CPU;
use super::utils;
use super::utils::address::{p2v, paddr, paddr_raw, v2p, vaddr, vaddr_raw};
use super::utils::pointer::Ptr;
use super::x86;

// floating pointer
#[repr(C)]
struct mp {
    signature: [u8; 4], // "_MP_"
    physaddr: usize,    // phys addr of MP config table
    length: u8,         // 1
    specrev: u8,        // [14]
    checksum: u8,       // all bytes must add up to 0
    mp_type: u8,        // MP system config type
    imcrp: u8,
    reserved: [u8; 3],
}

// configuration table header
#[repr(C)]
struct mpconf {
    signature: [u8; 4],    // "PCMP"
    length: u16,           // total table length
    version: u8,           // [14]
    checksum: u8,          // all bytes must add up to 0
    product: [u8; 20],     // product id
    oem_table: *const u32, // OEM table pointer
    oem_length: u16,       // OEM table length
    entry: u16,            // entry count
    lapicaddr: *mut u32,   // address of local APIC
    xlenght: u16,          // extended table length
    xchecksum: u8,         // extended table checksum
    reserved: u8,
}

// processor table entry
#[repr(C)]
struct mpproc {
    entry_type: u8,     // entry type (0)
    apicid: u8,         // local APIC id
    version: u8,        // local APIC verison
    flags: u8,          // CPU flags
    signature: [u8; 4], // CPU signature
    feature: u32,       // feature flags from CPUID instruction
    reserved: [u8; 8],
}
const MPBOOT: u8 = 0x02;

// I/O APIC table entry
#[repr(C)]
struct mpioapic {
    entry_type: u8,   // entry type (2)
    apicno: u8,       // I/O APIC id
    version: u8,      // I/O APIC version
    flags: u8,        // I/O APIC flags
    addr: *const u32, // I/O APIC address
}

// be careful to use !
pub static mut CPU_ARRAY: CPUArray = CPUArray::new();
pub static mut ioapicid: u8 = 0;

#[derive(Debug)]
pub struct CPUArray {
    ncpu: usize,
    cpus: [Option<CPU>; param::NCPU],
}
impl CPUArray {
    pub const fn new() -> Self {
        CPUArray {
            ncpu: 0,
            cpus: [None, None, None, None, None, None, None, None],
        }
    }
    pub fn slice(&self) -> &[Option<CPU>] {
        &self.cpus
    }
    pub fn len(&self) -> usize {
        self.ncpu
    }
    pub fn add(&mut self, id: usize, apicid: u8) {
        self.cpus[self.ncpu] = Some(CPU::new(id, apicid));
        self.ncpu += 1;
    }
    pub fn borrow(&self, idx: usize) -> &CPU {
        self.cpus[idx].as_ref().unwrap()
    }
    pub fn borrow_mut(&mut self, idx: usize) -> &mut CPU {
        self.cpus[idx].as_mut().unwrap()
    }
}

// Table entry types
const MPPROC: u8 = 0x00; // One per processor
const MPBUS: u8 = 0x01; // One per bus
const MPIOAPIC: u8 = 0x02; // One per I/O APIC
const MPIOINTR: u8 = 0x03; // One per bus interrupt source
const MPLINTR: u8 = 0x04; // One per system interrupt source

fn sum(range: &[u8]) -> u8 {
    use core::num::Wrapping;
    range
        .iter()
        .map(|x: &u8| Wrapping(*x))
        .sum::<Wrapping<u8>>()
        .0
}

// Look for an MP structure in the given range.
fn mpsearch1<'a>(range: &'a [mp]) -> Option<&'a mp> {
    for x in range.iter() {
        let range = utils::bytes_from_ref(x);
        if &x.signature == b"_MP_" && sum(range) == 0 {
            return Some(x);
        }
    }
    None
}

// Search for the MP Floating Ptr Structure, which according to the
// spec is in one of the following three locations:
// 1) in the first KB of the EBDA;
// 2) in the last KB of system base memory;
// 3) in the BIOS ROM between 0xE0000 and 0xFFFFF.
fn mpsearch() -> Option<&'static mp> {
    fn mps_from_paddr<'a>(p: usize, len: usize) -> &'a [mp] {
        unsafe { core::slice::from_raw_parts(p2v(paddr_raw(p)).as_ptr::<mp>(), len) }
    }

    let bda = p2v(paddr::from_raw(0x400).unwrap()).as_ptr::<u8>();
    let mut p: usize = unsafe { (*bda.add(0x0F) as usize) << 8 | (*bda.add(0x0E) as usize) << 4 };
    if p != 0 {
        let range = mps_from_paddr(p, 1024 / 16);
        if let Some(r) = mpsearch1(range) {
            return Some(r);
        }
    } else {
        p = unsafe { ((*bda.add(0x14) as usize) << 8 | (*bda.add(0x13) as usize)) * 1024 };
        let range = mps_from_paddr(p, 1024 / 16);
        if let Some(r) = mpsearch1(range) {
            return Some(r);
        }
    }
    mpsearch1(mps_from_paddr(0xF0000, 0x10000 / 16))
}

fn mpconfig() -> Result<(&'static mp, &'static mpconf), &'static str> {
    let mp = mpsearch().ok_or("mpconfig: MP not found")?;
    if mp.physaddr == 0 {
        return Err("mpconfig: physaddr is 0");
    }
    let conf = unsafe {
        p2v(paddr_raw(mp.physaddr))
            .as_ptr::<mpconf>()
            .as_ref()
            .unwrap()
    };
    if &conf.signature != b"PCMP" {
        return Err("mpconfig: signature is not \"PCMP\"");
    }
    if conf.version != 1 && conf.version != 4 {
        return Err("mpconfig: version is not 1 nor 4");
    }
    let range = unsafe {
        core::slice::from_raw_parts(conf as *const mpconf as *const u8, conf.length as usize)
    };
    if sum(range) != 0 {
        return Err("mpconfig: invalid sum");
    }
    Ok((mp, conf))
}

pub fn mp_init() {
    {
        use core::mem::size_of;
        assert_eq!(size_of::<mp>(), 16);
    }

    let (mp, conf) = mpconfig().expect("Expect to run on an SMP");
    let mut is_mp = true;

    unsafe {
        // it's safe because now only 'this' processor is running.
        lapic::lapic = conf.lapicaddr;

        let mut p: Ptr<u8> = {
            let conf_addr = vaddr::from_raw(conf as *const mpconf as usize).unwrap();
            let mut p: Ptr<mpconf> = Ptr::from(conf_addr);
            p.increase(1);
            p.cast()
        };
        let e = (conf as *const mpconf as *const u8).add(conf.length as usize);
        assert!(!p.is_null());

        use core::mem::size_of;
        let mut ncpu = 0;
        while p.get() < e {
            match *p.get() {
                MPPROC => {
                    let proc: Ptr<mpproc> = p.cast();
                    if ncpu < param::NCPU {
                        let apicid = (*proc.get()).apicid;
                        CPU_ARRAY.add(ncpu, apicid);
                        ncpu += 1;
                    }
                    p.increase_bytes(size_of::<mpproc>()).unwrap();
                }
                MPIOAPIC => {
                    let ioapic: Ptr<mpioapic> = p.cast();
                    ioapicid = (*ioapic.get()).apicno;
                    p.increase_bytes(size_of::<mpioapic>()).unwrap();
                }
                MPBUS | MPIOINTR | MPLINTR => {
                    // skip 8 bytes
                    p.increase_bytes(8).unwrap();
                }
                _ => {
                    is_mp = false;
                    break;
                }
            }
        }
    }
    if !is_mp {
        panic!("Didn't find a suitable machine");
    }

    if mp.imcrp != 0 {
        // Bochs doesn't support IMCR, so this doesn't run on Bochs.
        // But it would on real hardware.
        x86::outb(0x22, 0x70); // Select IMCR
        x86::outb(0x23, x86::inb(0x23) | 1); // Mask external interrupts.
    }

    println!("ncpu = {}", unsafe { CPU_ARRAY.len() });
    unsafe {
        for c in CPU_ARRAY.slice().iter() {
            match c.as_ref() {
                Some(c) => {
                    println!("cpuid = {}, apicid = {}", c.id, c.apicid);
                }
                None => {}
            }
        }
    }
    println!("lapic = {:?}", unsafe { lapic::lapic });
}
