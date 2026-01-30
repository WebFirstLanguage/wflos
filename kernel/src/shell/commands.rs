/// Built-in shell commands
/// Implements command execution

use crate::{println, drivers, memory};

#[derive(Debug, PartialEq)]
pub enum Command<'a> {
    Empty,
    Help,
    Clear,
    Echo(&'a str),
    Version,
    MemInfo,
    Halt,
}

pub fn execute(cmd: Command) {
    match cmd {
        Command::Empty => {
            // Do nothing
        }
        Command::Help => cmd_help(),
        Command::Clear => cmd_clear(),
        Command::Echo(text) => cmd_echo(&text),
        Command::Version => cmd_version(),
        Command::MemInfo => cmd_meminfo(),
        Command::Halt => cmd_halt(),
    }
}

fn cmd_help() {
    println!("Available commands:");
    println!("  help      - Show this help message");
    println!("  clear     - Clear the screen");
    println!("  echo TEXT - Print text to screen");
    println!("  version   - Show kernel version");
    println!("  meminfo   - Display memory information");
    println!("  halt      - Halt the system");
}

fn cmd_clear() {
    drivers::vga::clear_screen();
}

fn cmd_echo(text: &str) {
    println!("{}", text);
}

fn cmd_version() {
    println!("wflos - Rust Microkernel OS");
    println!("Version 0.4.0 (Phase 4: Command-Line Interface)");
    println!("Built with Rust on Apple Silicon M1 for x86_64");
    println!();
    println!("Features:");
    println!("  - Cross-compilation (ARM64 -> x86_64)");
    println!("  - Limine bootloader protocol");
    println!("  - VGA text mode driver");
    println!("  - Serial port debugging");
    println!("  - GDT and IDT configured");
    println!("  - Physical frame allocator");
    println!("  - PS/2 keyboard input");
    println!("  - Interactive shell");
}

fn cmd_meminfo() {
    let (total, used, free) = memory::frame_allocator::stats();

    println!("Memory Information:");
    println!("  Total frames: {} ({} KB)", total, (total * 4) / 1024);
    println!("  Used frames:  {} ({} KB)", used, (used * 4) / 1024);
    println!("  Free frames:  {} ({} KB)", free, (free * 4) / 1024);
    println!();
    println!("Frame size: 4 KB");
}

fn cmd_halt() {
    println!("Halting system...");
    println!("You can close QEMU or press Ctrl+A then X to exit.");

    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
