use super::traps;
use super::utils::address::{paddr, paddr_raw, vaddr};

// IO APIC MMIO structure write reg, then read or write data.
#[repr(C)]
struct ioapic {
    reg: u32,
    pad: [u32; 3],
    data: u32,
}

lazy_static! {
     // Default physical address of IO APIC
    static ref IOAPIC: paddr = paddr_raw(0xFEC00000);
}

const REG_ID: u32 = 0x00; // Register index: ID
const REG_VER: u32 = 0x01; // Register index: version
const REG_TABLE: u32 = 0x10; // Redirection table base

// The redirection table starts at REG_TABLE and uses
// two registers to configure each interrupt.
// The first (low) register in a pair contains configuration bits.
// The second (high) register contains a bitmask telling which
// CPUs can serve that interrupt.
const INT_DISABLED: u32 = 0x00010000; // Interrupt disabled
const INT_LEVEL: u32 = 0x00008000; // Level-triggered (vs edge-)
const INT_ACTIVELOW: u32 = 0x00002000; // Active low (vs high)
const INT_LOGICAL: u32 = 0x00000800; // Destination is CPU id (vs APIC ID)

static mut ioapic: *mut ioapic = core::ptr::null_mut();

fn ioapic_read(reg: u32) -> u32 {
    unsafe {
        let ioapic_reg: *mut u32 = &mut (*ioapic).reg;
        core::ptr::write_volatile(ioapic_reg, reg);
    }
    unsafe {
        let ioapic_data: *const u32 = &(*ioapic).data;
        core::ptr::read_volatile(ioapic_data)
    }
}

fn ioapic_write(reg: u32, data: u32) {
    unsafe {
        let ioapic_reg: *mut u32 = &mut (*ioapic).reg;
        core::ptr::write_volatile(ioapic_reg, reg);
    }
    unsafe {
        let ioapic_data: *mut u32 = &mut (*ioapic).data;
        core::ptr::write_volatile(ioapic_data, data);
    }
}

pub fn ioapic_enable(irq: u32, cpunum: u32) {
    // Mark interrupt edge-triggered, active high,
    // enabled, and routed to the given cpunum,
    // which happens to be that cpu's APIC ID.
    ioapic_write(REG_TABLE + 2 * irq, traps::T_IRQ0 + irq);
    ioapic_write(REG_TABLE + 2 * irq + 1, cpunum << 24);
}
