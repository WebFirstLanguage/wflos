/// VGA text mode driver
/// Physical address: 0xB8000
/// Access through Limine's Higher-Half Direct Map (HHDM)

use crate::sync::spinlock::Spinlock;
use core::fmt;
use core::ptr;

const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;
const VGA_BUFFER_PHYSICAL: usize = 0xB8000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

impl ScreenChar {
    fn write(&mut self, value: ScreenChar) {
        unsafe {
            ptr::write_volatile(self, value);
        }
    }

    fn read(&self) -> ScreenChar {
        unsafe { ptr::read_volatile(self) }
    }
}

struct Buffer {
    chars: [[ScreenChar; VGA_WIDTH]; VGA_HEIGHT],
}

pub struct VgaBuffer {
    buffer: *mut Buffer,
    column_position: usize,
    row_position: usize,
    color_code: ColorCode,
}

unsafe impl Send for VgaBuffer {}

impl VgaBuffer {
    const fn new_uninit() -> Self {
        VgaBuffer {
            buffer: ptr::null_mut(),
            column_position: 0,
            row_position: 0,
            color_code: ColorCode::new(Color::White, Color::Black),
        }
    }

    pub fn init(&mut self, hhdm_offset: u64) {
        let vga_virtual = hhdm_offset + VGA_BUFFER_PHYSICAL as u64;
        self.buffer = vga_virtual as *mut Buffer;
        self.column_position = 0;
        self.row_position = 0;
        self.color_code = ColorCode::new(Color::White, Color::Black);
    }

    pub fn write_byte(&mut self, byte: u8) {
        if self.buffer.is_null() {
            return;
        }

        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= VGA_WIDTH {
                    self.new_line();
                }

                let row = self.row_position;
                let col = self.column_position;

                let color_code = self.color_code;
                let buffer = unsafe { &mut *self.buffer };

                buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });

                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe), // Replacement character
            }
        }
    }

    fn new_line(&mut self) {
        if self.row_position < VGA_HEIGHT - 1 {
            self.row_position += 1;
        } else {
            self.scroll_up();
        }
        self.column_position = 0;
    }

    fn scroll_up(&mut self) {
        if self.buffer.is_null() {
            return;
        }

        let buffer = unsafe { &mut *self.buffer };

        for row in 1..VGA_HEIGHT {
            for col in 0..VGA_WIDTH {
                let character = buffer.chars[row][col].read();
                buffer.chars[row - 1][col].write(character);
            }
        }

        self.clear_row(VGA_HEIGHT - 1);
    }

    fn clear_row(&mut self, row: usize) {
        if self.buffer.is_null() {
            return;
        }

        let buffer = unsafe { &mut *self.buffer };
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };

        for col in 0..VGA_WIDTH {
            buffer.chars[row][col].write(blank);
        }
    }

    pub fn clear(&mut self) {
        for row in 0..VGA_HEIGHT {
            self.clear_row(row);
        }
        self.column_position = 0;
        self.row_position = 0;
    }
}

impl fmt::Write for VgaBuffer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

static VGA_WRITER: Spinlock<VgaBuffer> = Spinlock::new(VgaBuffer::new_uninit());

pub fn init(hhdm_offset: u64) {
    VGA_WRITER.lock().init(hhdm_offset);
}

pub fn clear_screen() {
    VGA_WRITER.lock().clear();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::drivers::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    VGA_WRITER.lock().write_fmt(args).unwrap();
}
