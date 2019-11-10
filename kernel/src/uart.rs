use super::x86;

const COM1: u16 = 0x3f8;

static mut uart: bool = false; // is there a uart?

pub fn uart_init() {
    // Turn off the FIFO
    x86::outb(COM1 + 2, 0);

    // 9600 baud, 8 data bits, 1 stop bit, parity off.
    x86::outb(COM1 + 3, 0x80); // Unlock divisor
    x86::outb(COM1 + 0, (115200 / 9600) as u8); // ?
    x86::outb(COM1 + 1, 0);
    x86::outb(COM1 + 3, 0x03); // Lock divisor, 8 data bits.
    x86::outb(COM1 + 4, 0);
    x86::outb(COM1 + 1, 0x01); // Enable receive interrupts.

    // If status is 0xFF, no serial port.
    if x86::inb(COM1 + 5) == 0xFF {
        return;
    }
    unsafe {
        uart = true;
    }

    // Acknowledge pre-existing interrupt conditions;
    // enable interrupts.
    x86::inb(COM1 + 2);
    x86::inb(COM1 + 0);

    unimplemented!()
}
