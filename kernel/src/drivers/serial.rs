//! Serial port driver for COM1 (0x3F8)
//! Used for debugging output in QEMU

use crate::sync::spinlock::Spinlock;
use core::fmt;

const COM1_PORT: u16 = 0x3F8;

pub struct Serial {
    initialized: bool,
}

impl Serial {
    const fn new() -> Self {
        Serial {
            initialized: false,
        }
    }

    pub fn init(&mut self) {
        unsafe {
            // Disable interrupts
            outb(COM1_PORT + 1, 0x00);

            // Enable DLAB (set baud rate divisor)
            outb(COM1_PORT + 3, 0x80);

            // Set divisor to 3 (38400 baud)
            outb(COM1_PORT, 0x03);
            outb(COM1_PORT + 1, 0x00);

            // 8 bits, no parity, one stop bit
            outb(COM1_PORT + 3, 0x03);

            // Enable FIFO, clear with 14-byte threshold
            outb(COM1_PORT + 2, 0xC7);

            // IRQs enabled, RTS/DSR set
            outb(COM1_PORT + 4, 0x0B);

            // Set in loopback mode, test the serial chip
            outb(COM1_PORT + 4, 0x1E);

            // Test serial chip (send byte 0xAE and check if serial returns same byte)
            outb(COM1_PORT, 0xAE);

            // Check if serial is faulty
            if inb(COM1_PORT) != 0xAE {
                return;
            }

            // Set to normal operation mode
            outb(COM1_PORT + 4, 0x0F);

            self.initialized = true;
        }
    }

    fn is_transmit_empty(&self) -> bool {
        unsafe { (inb(COM1_PORT + 5) & 0x20) != 0 }
    }

    pub fn write_byte(&mut self, byte: u8) {
        if !self.initialized {
            return;
        }

        // Wait for transmit buffer to be empty
        while !self.is_transmit_empty() {
            core::hint::spin_loop();
        }

        unsafe {
            outb(COM1_PORT, byte);
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
    }
}

impl fmt::Write for Serial {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

static SERIAL: Spinlock<Serial> = Spinlock::new(Serial::new());

pub fn init() {
    SERIAL.lock().init();
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => ($crate::drivers::serial::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($($arg:tt)*) => ($crate::serial_print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    SERIAL.lock().write_fmt(args).unwrap();
}

// x86_64 I/O port operations
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
