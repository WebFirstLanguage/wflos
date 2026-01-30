#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

mod arch;
mod drivers;
mod limine;
mod memory;
mod shell;
mod sync;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("KERNEL PANIC: {}", info);
    loop {
        core::hint::spin_loop();
    }
}

#[no_mangle]
extern "C" fn _start() -> ! {
    // Initialize serial port first for early debugging
    drivers::serial::init();
    serial_println!("Serial port initialized");

    // Get HHDM offset from Limine
    let hhdm_offset = limine::HHDM_REQUEST
        .get_response()
        .expect("Limine HHDM request failed")
        .offset;

    serial_println!("HHDM offset: {:#x}", hhdm_offset);

    // Initialize VGA driver
    drivers::vga::init(hhdm_offset);

    // Clear screen
    drivers::vga::clear_screen();

    // Display boot message
    println!("wflos - Rust Microkernel OS");
    println!("Version 0.4.0 (Phase 4: Command-Line Interface)");
    println!();
    println!("Booting kernel...");
    println!();

    serial_println!("VGA initialized");
    serial_println!("wflos - Rust Microkernel OS");
    serial_println!("Version 0.4.0 (Phase 4: Command-Line Interface)");

    // Initialize GDT
    serial_println!("Initializing GDT...");
    arch::x86_64::gdt::init();
    serial_println!("GDT loaded");

    // Initialize IDT
    serial_println!("Initializing IDT...");
    arch::x86_64::idt::init();
    serial_println!("IDT loaded");

    // Initialize PIC
    serial_println!("Initializing PIC...");
    arch::x86_64::pic::init();
    serial_println!("PIC initialized and remapped");

    // Initialize frame allocator (before interrupts and heap)
    if let Some(memmap_response) = limine::MEMMAP_REQUEST.get_response() {
        let entry_count = memmap_response.entry_count as usize;

        // Can't use Vec yet (heap not initialized), build array manually
        // Use a dummy reference that will be overwritten for each valid entry
        let dummy = unsafe { &**memmap_response.entries };
        let mut map_slice: [&limine::LimineMemoryMapEntry; 64] = [dummy; 64];
        let mut map_count = 0;

        for (i, slot) in map_slice.iter_mut().enumerate().take(entry_count.min(64)) {
            let entry = unsafe { &**memmap_response.entries.add(i) };
            *slot = entry;
            map_count += 1;
        }

        let initialized_slice = &map_slice[..map_count];

        serial_println!("Initializing frame allocator...");
        memory::frame_allocator::init(initialized_slice, hhdm_offset);

        let (total, used, free) = memory::frame_allocator::stats();
        serial_println!("Frame allocator: {} total, {} used, {} free", total, used, free);
        println!("Memory: {} KB total", (total * 4096) / 1024);
    }

    // Initialize heap allocator (before interrupts)
    serial_println!("Initializing heap allocator...");
    match memory::heap::init(hhdm_offset) {
        Ok(()) => {
            serial_println!("Heap allocator initialized");
            println!("Heap: 64 KB initialized");
            memory::heap::verify_heap();
        }
        Err(e) => {
            serial_println!("Heap allocator failed: {}", e);
            println!("Heap: FAILED ({})", e);
        }
    }

    // Initialize keyboard
    serial_println!("Initializing keyboard...");
    drivers::keyboard::init();
    serial_println!("Keyboard initialized");

    // Enable interrupts (after all initialization is complete)
    serial_println!("Enabling interrupts...");
    unsafe {
        core::arch::asm!("sti");
    }
    serial_println!("Interrupts enabled");

    println!();
    println!("Phase 5 complete: Heap allocator operational");
    println!();

    serial_println!("\n=== Phase 5 Complete ===");
    serial_println!("  - GDT initialized and loaded");
    serial_println!("  - IDT initialized with exception handlers");
    let (total, _used, _free) = memory::frame_allocator::stats();
    serial_println!("  - Frame allocator operational ({} frames available)", total);
    serial_println!("  - Heap allocator initialized (64 KB)");
    serial_println!("  - PIC remapped (IRQs at vectors 32-47)");
    serial_println!("  - Keyboard driver ready (IRQ1)");
    serial_println!("  - Interrupts enabled");
    serial_println!("  - Shell ready for commands");
    serial_println!("========================\n");

    // Keyboard is ready - launch shell
    serial_println!("Launching shell...");

    // Run the shell REPL (never returns)
    shell::run();
}
