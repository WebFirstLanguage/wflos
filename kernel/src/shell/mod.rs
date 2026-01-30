//! Shell REPL (Read-Eval-Print Loop)
//! Provides interactive command-line interface

pub mod parser;
pub mod commands;

use crate::drivers;
use crate::{print, println};

const PROMPT: &str = "wflos> ";
const MAX_LINE_LENGTH: usize = 128;

// Static line buffer to avoid stack overflow
static mut LINE_BUFFER: [u8; MAX_LINE_LENGTH] = [0; MAX_LINE_LENGTH];

/// Run the shell REPL
pub fn run() -> ! {
    println!();
    println!("=== wflos Shell ===");
    println!("Type 'help' for available commands");
    println!();

    loop {
        // Display prompt
        print!("{}", PROMPT);

        // Read line
        let mut line_pos = 0;
        loop {
            if let Some(key) = drivers::keyboard::read_key() {
                match key {
                    '\n' => {
                        // Enter pressed
                        println!();
                        break;
                    }
                    '\x08' => {
                        // Backspace
                        if line_pos > 0 {
                            line_pos -= 1;
                            // Erase character: backspace, space, backspace
                            print!("\x08 \x08");
                        }
                    }
                    '\x1B' => {
                        // ESC - clear line
                        while line_pos > 0 {
                            print!("\x08 \x08");
                            line_pos -= 1;
                        }
                    }
                    '\t' => {
                        // Tab - ignore for now
                    }
                    c if c.is_ascii_graphic() || c == ' ' => {
                        // Printable character
                        if line_pos < MAX_LINE_LENGTH {
                            unsafe {
                                LINE_BUFFER[line_pos] = c as u8;
                            }
                            line_pos += 1;
                            print!("{}", c);
                        }
                    }
                    _ => {
                        // Ignore other characters
                    }
                }
            }
        }

        // Parse and execute command
        if line_pos > 0 {
            let line = unsafe {
                core::str::from_utf8(&LINE_BUFFER[..line_pos])
                    .unwrap_or("")
            };

            match parser::parse(line) {
                Ok(cmd) => commands::execute(cmd),
                Err(e) => println!("Error: {}", e),
            }
        }
    }
}
