/// PIC (Programmable Interrupt Controller) configuration
/// Remaps IRQs to avoid conflicts with CPU exceptions

use core::arch::asm;

const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

const ICW1_INIT: u8 = 0x11;
const ICW4_8086: u8 = 0x01;

const PIC_EOI: u8 = 0x20;

/// Remap PIC interrupts to avoid conflicts with CPU exceptions
/// CPU exceptions use vectors 0-31, so we remap PIC to 32-47
pub fn init() {
    unsafe {
        // Save masks
        let mask1 = inb(PIC1_DATA);
        let mask2 = inb(PIC2_DATA);

        // Start initialization sequence
        outb(PIC1_COMMAND, ICW1_INIT);
        io_wait();
        outb(PIC2_COMMAND, ICW1_INIT);
        io_wait();

        // Set vector offsets
        outb(PIC1_DATA, 32); // Master PIC starts at 32
        io_wait();
        outb(PIC2_DATA, 40); // Slave PIC starts at 40
        io_wait();

        // Configure cascade
        outb(PIC1_DATA, 4); // Tell Master PIC that there is a slave PIC at IRQ2
        io_wait();
        outb(PIC2_DATA, 2); // Tell Slave PIC its cascade identity
        io_wait();

        // Set mode
        outb(PIC1_DATA, ICW4_8086);
        io_wait();
        outb(PIC2_DATA, ICW4_8086);
        io_wait();

        // Restore masks
        outb(PIC1_DATA, mask1);
        outb(PIC2_DATA, mask2);
    }
}

/// Enable a specific IRQ line
pub fn enable_irq(irq: u8) {
    let port = if irq < 8 { PIC1_DATA } else { PIC2_DATA };
    let value = unsafe { inb(port) } & !(1 << (irq % 8));
    unsafe { outb(port, value) };
}

/// Disable a specific IRQ line
pub fn disable_irq(irq: u8) {
    let port = if irq < 8 { PIC1_DATA } else { PIC2_DATA };
    let value = unsafe { inb(port) } | (1 << (irq % 8));
    unsafe { outb(port, value) };
}

/// Send End of Interrupt signal
pub fn send_eoi(irq: u8) {
    unsafe {
        if irq >= 8 {
            outb(PIC2_COMMAND, PIC_EOI);
        }
        outb(PIC1_COMMAND, PIC_EOI);
    }
}

/// Disable all IRQs
pub fn disable_all() {
    unsafe {
        outb(PIC1_DATA, 0xFF);
        outb(PIC2_DATA, 0xFF);
    }
}

#[inline]
unsafe fn outb(port: u16, value: u8) {
    asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
        options(nomem, nostack, preserves_flags)
    );
}

#[inline]
unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    asm!(
        "in al, dx",
        out("al") value,
        in("dx") port,
        options(nomem, nostack, preserves_flags)
    );
    value
}

#[inline]
fn io_wait() {
    unsafe {
        outb(0x80, 0); // Write to unused port for I/O delay
    }
}
