/// Exception and interrupt handlers for x86_64

use crate::{println, serial_println};

#[no_mangle]
pub extern "C" fn divide_by_zero_handler() {
    serial_println!("EXCEPTION: Divide by Zero");
    println!("EXCEPTION: Divide by Zero");
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

#[no_mangle]
pub extern "C" fn debug_handler() {
    serial_println!("EXCEPTION: Debug");
    println!("EXCEPTION: Debug");
}

#[no_mangle]
pub extern "C" fn breakpoint_handler() {
    serial_println!("EXCEPTION: Breakpoint");
    println!("EXCEPTION: Breakpoint");
}

#[no_mangle]
pub extern "C" fn page_fault_handler() {
    serial_println!("EXCEPTION: Page Fault");
    println!("EXCEPTION: Page Fault");

    // Read CR2 register for faulting address
    let faulting_address: u64;
    unsafe {
        core::arch::asm!(
            "mov {}, cr2",
            out(reg) faulting_address,
            options(nostack, preserves_flags)
        );
    }

    serial_println!("  Faulting address: {:#x}", faulting_address);
    println!("  Faulting address: {:#x}", faulting_address);

    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

#[no_mangle]
pub extern "C" fn general_protection_fault_handler() {
    serial_println!("EXCEPTION: General Protection Fault");
    println!("EXCEPTION: General Protection Fault");

    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

#[no_mangle]
pub extern "C" fn double_fault_handler() {
    serial_println!("EXCEPTION: Double Fault");
    println!("EXCEPTION: Double Fault");

    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
