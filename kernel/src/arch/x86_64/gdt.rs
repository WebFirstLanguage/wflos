//! Global Descriptor Table (GDT) for x86_64
//! Required for long mode, defines code and data segments

use core::arch::asm;

#[repr(C, packed)]
struct GdtDescriptor {
    size: u16,
    offset: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_mid: u8,
    access: u8,
    granularity: u8,
    base_high: u8,
}

impl GdtEntry {
    pub const fn null() -> Self {
        GdtEntry {
            limit_low: 0,
            base_low: 0,
            base_mid: 0,
            access: 0,
            granularity: 0,
            base_high: 0,
        }
    }

    pub const fn new(access: u8, flags: u8) -> Self {
        GdtEntry {
            limit_low: 0,
            base_low: 0,
            base_mid: 0,
            access,
            granularity: flags,
            base_high: 0,
        }
    }
}

// GDT access bits
const PRESENT: u8 = 1 << 7;
const DPL_0: u8 = 0 << 5;
const DPL_3: u8 = 3 << 5;
const DESCRIPTOR_TYPE: u8 = 1 << 4;
const EXECUTABLE: u8 = 1 << 3;
const RW: u8 = 1 << 1;

// GDT flags
const GRANULARITY: u8 = 1 << 7;
const LONG_MODE: u8 = 1 << 5;

const GDT_ENTRY_COUNT: usize = 9;

// Segment selectors (byte offsets into GDT)
pub const KERNEL_CODE_SELECTOR: u16 = 0x28;
#[allow(dead_code)]
pub const KERNEL_DATA_SELECTOR: u16 = 0x30;

pub struct Gdt {
    table: [GdtEntry; GDT_ENTRY_COUNT],
}

impl Gdt {
    /// Layout matches Limine bootloader's GDT selector assignments:
    ///   0x28 = 64-bit kernel code, 0x30 = 64-bit kernel data
    pub const fn new() -> Self {
        Gdt {
            table: [
                GdtEntry::null(), // 0x00: Null descriptor
                GdtEntry::null(), // 0x08: Reserved (Limine 16-bit code)
                GdtEntry::null(), // 0x10: Reserved (Limine 16-bit data)
                GdtEntry::null(), // 0x18: Reserved (Limine 32-bit code)
                GdtEntry::null(), // 0x20: Reserved (Limine 32-bit data)
                GdtEntry::new(    // 0x28: Kernel code segment (64-bit)
                    PRESENT | DPL_0 | DESCRIPTOR_TYPE | EXECUTABLE | RW,
                    GRANULARITY | LONG_MODE,
                ),
                GdtEntry::new(    // 0x30: Kernel data segment (64-bit)
                    PRESENT | DPL_0 | DESCRIPTOR_TYPE | RW,
                    GRANULARITY,
                ),
                GdtEntry::new(    // 0x38: User code segment (64-bit)
                    PRESENT | DPL_3 | DESCRIPTOR_TYPE | EXECUTABLE | RW,
                    GRANULARITY | LONG_MODE,
                ),
                GdtEntry::new(    // 0x40: User data segment (64-bit)
                    PRESENT | DPL_3 | DESCRIPTOR_TYPE | RW,
                    GRANULARITY,
                ),
            ],
        }
    }

    pub fn load(&'static self) {
        use crate::serial_println;

        let gdt_size = (core::mem::size_of::<[GdtEntry; GDT_ENTRY_COUNT]>() - 1) as u16;
        let gdt_offset = self.table.as_ptr() as u64;

        let descriptor = GdtDescriptor {
            size: gdt_size,
            offset: gdt_offset,
        };

        serial_println!("  GDT descriptor: size={}, offset={:#x}", gdt_size, gdt_offset);

        unsafe {
            serial_println!("  Loading GDT...");
            asm!(
                "lgdt [{}]",
                in(reg) &descriptor,
                options(nostack, preserves_flags)
            );
            serial_println!("  GDT loaded (Limine selectors 0x28/0x30 preserved)");
        }
    }
}

static GDT: Gdt = Gdt::new();

pub fn init() {
    GDT.load();
}
