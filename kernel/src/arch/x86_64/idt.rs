/// Interrupt Descriptor Table (IDT) for x86_64
/// Handles CPU exceptions and hardware interrupts

use core::arch::asm;

#[repr(C, packed)]
struct IdtDescriptor {
    size: u16,
    offset: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    type_attr: u8,
    offset_mid: u16,
    offset_high: u32,
    reserved: u32,
}

impl IdtEntry {
    pub const fn null() -> Self {
        IdtEntry {
            offset_low: 0,
            selector: 0,
            ist: 0,
            type_attr: 0,
            offset_mid: 0,
            offset_high: 0,
            reserved: 0,
        }
    }

    pub const fn new(handler: usize) -> Self {
        IdtEntry {
            offset_low: (handler & 0xFFFF) as u16,
            selector: 0x08, // Kernel code segment
            ist: 0,
            type_attr: 0x8E, // Present, DPL=0, Interrupt Gate
            offset_mid: ((handler >> 16) & 0xFFFF) as u16,
            offset_high: ((handler >> 32) & 0xFFFFFFFF) as u32,
            reserved: 0,
        }
    }
}

const IDT_ENTRIES: usize = 256;

pub struct Idt {
    entries: [IdtEntry; IDT_ENTRIES],
}

impl Idt {
    pub const fn new() -> Self {
        Idt {
            entries: [IdtEntry::null(); IDT_ENTRIES],
        }
    }

    pub fn set_handler(&mut self, index: u8, handler: usize) {
        self.entries[index as usize] = IdtEntry::new(handler);
    }

    pub fn load(&'static self) {
        let descriptor = IdtDescriptor {
            size: (core::mem::size_of::<[IdtEntry; IDT_ENTRIES]>() - 1) as u16,
            offset: self.entries.as_ptr() as u64,
        };

        unsafe {
            asm!(
                "lidt [{}]",
                in(reg) &descriptor,
                options(nostack, preserves_flags)
            );
        }
    }
}

// Create wrapper functions that save/restore context
macro_rules! exception_wrapper {
    ($name:ident, $handler_name:ident) => {
        #[unsafe(naked)]
        pub extern "C" fn $name() {
            unsafe {
                core::arch::naked_asm!(
                    "push rax",
                    "push rcx",
                    "push rdx",
                    "push rsi",
                    "push rdi",
                    "push r8",
                    "push r9",
                    "push r10",
                    "push r11",
                    "call {0}",
                    "pop r11",
                    "pop r10",
                    "pop r9",
                    "pop r8",
                    "pop rdi",
                    "pop rsi",
                    "pop rdx",
                    "pop rcx",
                    "pop rax",
                    "iretq",
                    sym crate::arch::x86_64::interrupts::$handler_name,
                );
            }
        }
    };
}

exception_wrapper!(divide_by_zero_wrapper, divide_by_zero_handler);
exception_wrapper!(debug_wrapper, debug_handler);
exception_wrapper!(breakpoint_wrapper, breakpoint_handler);
exception_wrapper!(page_fault_wrapper, page_fault_handler);
exception_wrapper!(general_protection_fault_wrapper, general_protection_fault_handler);
exception_wrapper!(double_fault_wrapper, double_fault_handler);
exception_wrapper!(keyboard_wrapper, keyboard_interrupt_handler);

static mut IDT: Idt = Idt::new();

pub fn init() {
    unsafe {
        // Install exception handlers
        IDT.set_handler(0, divide_by_zero_wrapper as usize);
        IDT.set_handler(1, debug_wrapper as usize);
        IDT.set_handler(3, breakpoint_wrapper as usize);
        IDT.set_handler(8, double_fault_wrapper as usize);
        IDT.set_handler(13, general_protection_fault_wrapper as usize);
        IDT.set_handler(14, page_fault_wrapper as usize);

        // Install IRQ handlers (remapped to 32+)
        IDT.set_handler(33, keyboard_wrapper as usize); // IRQ1 -> vector 33

        // Load IDT
        IDT.load();
    }
}
