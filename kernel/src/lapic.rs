use super::traps;
use super::x86;

pub static mut lapic: *mut u32 = core::ptr::null_mut();

// Local APIC registers, divided by 4 for use as uint[] indices.
const ID: usize = (0x0020 / 4); // ID
const VER: usize = (0x0030 / 4); // Version
const TPR: usize = (0x0080 / 4); // Task Priority
const EOI: usize = (0x00B0 / 4); // EOI
const SVR: usize = (0x00F0 / 4); // Spurious Interrupt Vector
const ENABLE: u32 = 0x00000100; // Unit Enable
const ESR: usize = (0x0280 / 4); // Error Status
const ICRLO: usize = (0x0300 / 4); // Interrupt Command
const INIT: u32 = 0x00000500; // INIT/RESET
const STARTUP: u32 = 0x00000600; // Startup IPI
const DELIVS: u32 = 0x00001000; // Delivery status
const ASSERT: u32 = 0x00004000; // Assert interrupt (vs deassert)
const DEASSERT: u32 = 0x00000000;
const LEVEL: u32 = 0x00008000; // Level triggered
const BCAST: u32 = 0x00080000; // Send to all APICs, including self.
const BUSY: u32 = 0x00001000;
const FIXED: u32 = 0x00000000;
const ICRHI: usize = (0x0310 / 4); // Interrupt Command [63:32]
const TIMER: usize = (0x0320 / 4); // Local Vector Table 0 (TIMER)
const X1: u32 = 0x0000000B; // divide counts by 1
const PERIODIC: u32 = 0x00020000; // Periodic
const PCINT: usize = (0x0340 / 4); // Performance Counter LVT
const LINT0: usize = (0x0350 / 4); // Local Vector Table 1 (LINT0)
const LINT1: usize = (0x0360 / 4); // Local Vector Table 2 (LINT1)
const ERROR: usize = (0x0370 / 4); // Local Vector Table 3 (ERROR)
const MASKED: u32 = 0x00010000; // Interrupt masked
const TICR: usize = (0x0380 / 4); // Timer Initial Count
const TCCR: usize = (0x0390 / 4); // Timer Current Count
const TDCR: usize = (0x03E0 / 4); // Timer Divide Configuration

fn lapic_write(index: usize, value: u32) {
    unsafe {
        core::ptr::write_volatile(lapic.offset(index as isize), value);
        core::ptr::read_volatile(lapic.offset(ID as isize)); // wait for to finish, by reading
    }
}
fn lapic_read(index: usize) -> u32 {
    unsafe { core::ptr::read_volatile(lapic.offset(index as isize)) }
}

pub fn lapic_init() {
    if unsafe { lapic.is_null() } {
        return;
    }

    // Enable local APIC; set spurious interrupt vector.
    lapic_write(SVR, ENABLE | (traps::T_IRQ0 + traps::IRQ_SPURIOUS));

    // The timer repeatedly counts down at bus frequency
    // from lapic[TICR] and then issues an interrupt.
    // If xv6 cared more about precise timekeeping,
    // TICR would be calibrated using an external time source.
    lapic_write(TDCR, X1);
    lapic_write(TIMER, PERIODIC | (traps::T_IRQ0 + traps::IRQ_TIMER));
    lapic_write(TICR, 10000000);

    // Disable logical interrupt lines.
    lapic_write(LINT0, MASKED);
    lapic_write(LINT1, MASKED);

    if lapic_read(VER) >> 16 & 0xFF >= 4 {
        lapic_write(PCINT, MASKED);
    }

    // Map error interrupt to IRQ_ERROR.
    lapic_write(ERROR, traps::T_IRQ0 + traps::IRQ_ERROR);

    // Clear error status register (requires back-to-back writes).
    lapic_write(ESR, 0);
    lapic_write(ESR, 0);

    // Ack any outstanding interrupts.
    lapic_write(EOI, 0);

    // Send an Init Level De-Assert to synchronise arbitration ID's.
    lapic_write(ICRHI, 0);
    lapic_write(ICRLO, BCAST | INIT | LEVEL);
    while lapic_read(ICRLO) & DELIVS != 0 {
        x86::nop();
    }

    // Enable interrupts on the APIC (but not on the processor).
    lapic_write(TPR, 0);
}
