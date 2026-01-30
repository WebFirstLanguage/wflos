/// PS/2 Keyboard driver
/// Handles scan codes from PS/2 keyboard controller

use crate::arch::x86_64::pic;
use crate::sync::spinlock::Spinlock;
use shared::data_structures::ring_buffer::RingBuffer;

const PS2_DATA_PORT: u16 = 0x60;
const PS2_STATUS_PORT: u16 = 0x64;
const PS2_COMMAND_PORT: u16 = 0x64;

const BUFFER_SIZE: usize = 256;

static KEYBOARD_BUFFER: Spinlock<RingBuffer<u8, BUFFER_SIZE>> =
    Spinlock::new(RingBuffer::new());

/// Initialize PS/2 keyboard
pub fn init() {
    // Enable keyboard IRQ (IRQ1)
    pic::enable_irq(1);

    // Flush keyboard buffer
    unsafe {
        while (inb(PS2_STATUS_PORT) & 1) != 0 {
            inb(PS2_DATA_PORT);
        }
    }
}

/// Handle keyboard interrupt (called from IRQ handler)
pub fn handle_interrupt() {
    unsafe {
        let scan_code = inb(PS2_DATA_PORT);

        // Add to buffer
        KEYBOARD_BUFFER.lock().push(scan_code);

        // Send EOI
        pic::send_eoi(1);
    }
}

/// Read a scan code from the buffer
pub fn read_scancode() -> Option<u8> {
    KEYBOARD_BUFFER.lock().pop()
}

/// Read a key (blocking)
pub fn read_key() -> Option<char> {
    while let Some(scan_code) = read_scancode() {
        if let Some(key) = scancode_to_ascii(scan_code) {
            return Some(key);
        }
    }
    None
}

/// Convert scan code to ASCII (US keyboard layout, Set 1)
/// Only handles key press events (not release)
fn scancode_to_ascii(scan_code: u8) -> Option<char> {
    // Ignore key release events (bit 7 set)
    if scan_code & 0x80 != 0 {
        return None;
    }

    match scan_code {
        0x01 => Some('\x1B'), // ESC
        0x02 => Some('1'),
        0x03 => Some('2'),
        0x04 => Some('3'),
        0x05 => Some('4'),
        0x06 => Some('5'),
        0x07 => Some('6'),
        0x08 => Some('7'),
        0x09 => Some('8'),
        0x0A => Some('9'),
        0x0B => Some('0'),
        0x0C => Some('-'),
        0x0D => Some('='),
        0x0E => Some('\x08'), // Backspace
        0x0F => Some('\t'),   // Tab
        0x10 => Some('q'),
        0x11 => Some('w'),
        0x12 => Some('e'),
        0x13 => Some('r'),
        0x14 => Some('t'),
        0x15 => Some('y'),
        0x16 => Some('u'),
        0x17 => Some('i'),
        0x18 => Some('o'),
        0x19 => Some('p'),
        0x1A => Some('['),
        0x1B => Some(']'),
        0x1C => Some('\n'), // Enter
        0x1E => Some('a'),
        0x1F => Some('s'),
        0x20 => Some('d'),
        0x21 => Some('f'),
        0x22 => Some('g'),
        0x23 => Some('h'),
        0x24 => Some('j'),
        0x25 => Some('k'),
        0x26 => Some('l'),
        0x27 => Some(';'),
        0x28 => Some('\''),
        0x29 => Some('`'),
        0x2B => Some('\\'),
        0x2C => Some('z'),
        0x2D => Some('x'),
        0x2E => Some('c'),
        0x2F => Some('v'),
        0x30 => Some('b'),
        0x31 => Some('n'),
        0x32 => Some('m'),
        0x33 => Some(','),
        0x34 => Some('.'),
        0x35 => Some('/'),
        0x39 => Some(' '), // Space
        _ => None,          // Unsupported key
    }
}

#[inline]
unsafe fn outb(port: u16, value: u8) {
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
        options(nomem, nostack, preserves_flags)
    );
}

#[inline]
unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!(
        "in al, dx",
        out("al") value,
        in("dx") port,
        options(nomem, nostack, preserves_flags)
    );
    value
}
