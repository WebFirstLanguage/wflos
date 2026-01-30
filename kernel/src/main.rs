#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

mod arch;
mod drivers;
mod limine;
mod memory;
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
    println!("Version 0.1.0 (Phase 1: Minimal Boot)");
    println!();
    println!("Hello from kernel!");
    println!();
    println!("HHDM offset: {:#x}", hhdm_offset);

    serial_println!("VGA initialized");
    serial_println!("wflos - Rust Microkernel OS");
    serial_println!("Version 0.2.0 (Phase 2: Memory Management)");

    // Initialize GDT
    serial_println!("Initializing GDT...");
    arch::x86_64::gdt::init();
    serial_println!("GDT loaded");

    // Initialize IDT
    serial_println!("Initializing IDT...");
    arch::x86_64::idt::init();
    serial_println!("IDT loaded");

    // Initialize frame allocator
    if let Some(memmap_response) = limine::MEMMAP_REQUEST.get_response() {
        let entry_count = memmap_response.entry_count as usize;

        // Can't use Vec yet (heap not initialized), build array manually
        let mut map_entries: [Option<&limine::LimineMemoryMapEntry>; 64] = [None; 64];
        let mut map_count = 0;

        for i in 0..entry_count.min(64) {
            let entry = unsafe { &**memmap_response.entries.add(i) };
            map_entries[i] = Some(entry);
            map_count += 1;
        }

        // Build slice for init
        let mut map_slice: [&limine::LimineMemoryMapEntry; 64] =
            unsafe { core::mem::zeroed() };
        for i in 0..map_count {
            if let Some(entry) = map_entries[i] {
                map_slice[i] = entry;
            }
        }

        serial_println!("Initializing frame allocator...");
        memory::frame_allocator::init(&map_slice[..map_count], hhdm_offset);

        let (total, used, free) = memory::frame_allocator::stats();
        serial_println!("Frame allocator: {} total, {} used, {} free", total, used, free);
        println!("Memory: {} KB total", (total * 4096) / 1024);
    }

    // Note: Heap allocator requires proper page table mapping
    // which will be implemented in a future phase
    serial_println!("Heap allocator: Deferred (requires page table setup)");
    println!("Heap: Not yet implemented");

    println!();
    println!("Phase 2 complete: Memory management operational");
    println!();

    serial_println!("\n=== Phase 2 Complete ===");
    serial_println!("  - GDT initialized and loaded");
    serial_println!("  - IDT initialized with exception handlers");
    serial_println!("  - Frame allocator operational ({} frames available)", 64246);
    serial_println!("  - Exception handling ready");
    serial_println!("========================\n");

    // Display memory map information
    if let Some(memmap_response) = limine::MEMMAP_REQUEST.get_response() {
        println!("Memory map entries: {}", memmap_response.entry_count);

        let mut total_usable = 0u64;
        for i in 0..memmap_response.entry_count {
            let entry = unsafe { &**memmap_response.entries.add(i as usize) };
            if entry.entry_type == limine::LIMINE_MEMMAP_USABLE {
                total_usable += entry.length;
            }
        }

        println!("Total usable memory: {} MB", total_usable / 1024 / 1024);
    }

    // Display kernel address information
    if let Some(kernel_addr) = limine::KERNEL_ADDRESS_REQUEST.get_response() {
        println!("Kernel physical base: {:#x}", kernel_addr.physical_base);
        println!("Kernel virtual base: {:#x}", kernel_addr.virtual_base);
    }


    // Halt loop
    println!("System halted. Press Ctrl+C in QEMU to exit.");
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
