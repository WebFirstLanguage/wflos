//! VGA text mode driver
//! Physical address: 0xB8000
//! Access through Limine's Higher-Half Direct Map (HHDM)

use crate::sync::spinlock::Spinlock;
use crate::serial_println;
use core::fmt;
use core::ptr;

const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;
const VGA_BUFFER_PHYSICAL: usize = 0xB8000;

// Framebuffer text mode constants
const CHAR_WIDTH: usize = 8;
const CHAR_HEIGHT: usize = 16;

// Simple 8x16 bitmap font (subset of printable ASCII)
// Each character is 16 bytes (1 bit per pixel, 8 pixels wide, 16 pixels tall)
fn get_char_bitmap(c: u8) -> &'static [u8; 16] {
    // Minimal bitmap font for common ASCII characters
    static FONT_DATA: [[u8; 16]; 128] = include!("vga_font.rs");

    if (c as usize) < 128 {
        &FONT_DATA[c as usize]
    } else {
        &FONT_DATA[0] // Default to null character
    }
}

#[allow(dead_code)]
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

// Framebuffer info
struct FramebufferInfo {
    address: *mut u8,
    width: usize,
    height: usize,
    pitch: usize,
    bpp: u16,
}

pub struct VgaBuffer {
    buffer: *mut Buffer,
    column_position: usize,
    row_position: usize,
    color_code: ColorCode,
    // Limine terminal for fallback
    limine_terminal: Option<*const crate::limine::LimineTerminal>,
    limine_write: Option<extern "C" fn(*const crate::limine::LimineTerminal, *const u8, u64)>,
    // Framebuffer for graphics mode
    framebuffer: Option<FramebufferInfo>,
}

unsafe impl Send for VgaBuffer {}

impl VgaBuffer {
    const fn new_uninit() -> Self {
        VgaBuffer {
            buffer: ptr::null_mut(),
            column_position: 0,
            row_position: 0,
            color_code: ColorCode::new(Color::White, Color::Black),
            limine_terminal: None,
            limine_write: None,
            framebuffer: None,
        }
    }

    pub fn init(&mut self, hhdm_offset: u64) {
        // Try to use Limine framebuffer first
        if let Some(fb_response) = crate::limine::FRAMEBUFFER_REQUEST.get_response() {
            if fb_response.framebuffer_count > 0 {
                let fb = unsafe { &**fb_response.framebuffers };
                self.framebuffer = Some(FramebufferInfo {
                    address: fb.address,
                    width: fb.width as usize,
                    height: fb.height as usize,
                    pitch: fb.pitch as usize,
                    bpp: fb.bpp,
                });
                self.column_position = 0;
                self.row_position = 0;
                serial_println!("Using framebuffer: {}x{}, bpp={}", fb.width, fb.height, fb.bpp);
                return;
            }
        }

        // Try to use Limine terminal
        if let Some(term_response) = crate::limine::TERMINAL_REQUEST.get_response() {
            if term_response.terminal_count > 0 && term_response.write.is_some() {
                let terminal = unsafe { *term_response.terminals };
                self.limine_terminal = Some(terminal);
                self.limine_write = term_response.write;
                serial_println!("Using Limine terminal for VGA output");
                return;
            }
        }

        // Fallback to direct VGA buffer access
        let vga_virtual = hhdm_offset + VGA_BUFFER_PHYSICAL as u64;
        self.buffer = vga_virtual as *mut Buffer;
        self.column_position = 0;
        self.row_position = 0;
        self.color_code = ColorCode::new(Color::White, Color::Black);
        serial_println!("Using direct VGA buffer: phys={:#x}, virt={:#x}", VGA_BUFFER_PHYSICAL, vga_virtual);
    }

    fn draw_char_fb(&mut self, c: u8, x: usize, y: usize) {
        if let Some(ref fb) = self.framebuffer {
            let bitmap = get_char_bitmap(c);

            for row in 0..CHAR_HEIGHT {
                let bits = bitmap[row];
                for col in 0..CHAR_WIDTH {
                    let pixel_on = (bits & (0x80 >> col)) != 0;
                    let pixel_x = x * CHAR_WIDTH + col;
                    let pixel_y = y * CHAR_HEIGHT + row;

                    if pixel_x < fb.width && pixel_y < fb.height {
                        let offset = pixel_y * fb.pitch + pixel_x * (fb.bpp as usize / 8);
                        unsafe {
                            let pixel_ptr = fb.address.add(offset);
                            // Write white (0xFFFFFF) or black (0x000000)
                            if fb.bpp == 32 {
                                let color: u32 = if pixel_on { 0xFFFFFFFF } else { 0x00000000 };
                                ptr::write_volatile(pixel_ptr as *mut u32, color);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        // Use framebuffer if available
        if self.framebuffer.is_some() {
            match byte {
                b'\n' => {
                    self.column_position = 0;
                    self.row_position += 1;
                    if self.row_position >= VGA_HEIGHT {
                        self.row_position = VGA_HEIGHT - 1;
                        // TODO: Implement scrolling
                    }
                }
                byte => {
                    if self.column_position >= VGA_WIDTH {
                        self.column_position = 0;
                        self.row_position += 1;
                        if self.row_position >= VGA_HEIGHT {
                            self.row_position = VGA_HEIGHT - 1;
                        }
                    }
                    self.draw_char_fb(byte, self.column_position, self.row_position);
                    self.column_position += 1;
                }
            }
            return;
        }

        // Use Limine terminal if available
        if let (Some(terminal), Some(write_fn)) = (self.limine_terminal, self.limine_write) {
            let bytes = [byte];
            write_fn(terminal, bytes.as_ptr(), 1);
            return;
        }

        // Fallback to direct VGA buffer
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
        // Use Limine terminal if available (more efficient)
        if let (Some(terminal), Some(write_fn)) = (self.limine_terminal, self.limine_write) {
            write_fn(terminal, s.as_ptr(), s.len() as u64);
            return;
        }

        // Fallback to direct VGA buffer
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
        // Limine terminal handles clearing internally, just reset position
        if self.limine_terminal.is_some() {
            // Send ANSI clear screen sequence
            if let (Some(terminal), Some(write_fn)) = (self.limine_terminal, self.limine_write) {
                let clear_seq = b"\x1B[2J\x1B[H"; // Clear screen + move cursor to home
                write_fn(terminal, clear_seq.as_ptr(), clear_seq.len() as u64);
            }
            return;
        }

        // Fallback to direct VGA buffer
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
