//! Limine bootloader protocol structures
//! Documentation: https://github.com/limine-bootloader/limine/blob/trunk/PROTOCOL.md

use core::ptr;

// Limine protocol magic numbers
const LIMINE_COMMON_MAGIC: [u64; 2] = [0xc7b1dd30df4c8b88, 0x0a82e883a194f07b];

// Base request structure
#[repr(C)]
pub struct LimineRequest<T> {
    id: [u64; 4],
    revision: u64,
    response: *const T,
}

unsafe impl<T> Sync for LimineRequest<T> {}

impl<T> LimineRequest<T> {
    pub const fn new(id_a: u64, id_b: u64) -> Self {
        LimineRequest {
            id: [LIMINE_COMMON_MAGIC[0], LIMINE_COMMON_MAGIC[1], id_a, id_b],
            revision: 0,
            response: ptr::null(),
        }
    }

    pub fn get_response(&self) -> Option<&'static T> {
        if self.response.is_null() {
            None
        } else {
            Some(unsafe { &*self.response })
        }
    }
}

// HHDM (Higher-Half Direct Map) Request
#[repr(C)]
pub struct LimineHhdmResponse {
    pub revision: u64,
    pub offset: u64,
}

#[used]
#[link_section = ".limine_reqs"]
pub static HHDM_REQUEST: LimineRequest<LimineHhdmResponse> =
    LimineRequest::new(0x48dcf1cb8ad2b852, 0x63984e959a98244b);

// Framebuffer Request (for future graphics support)
#[repr(C)]
pub struct LimineFramebufferResponse {
    pub revision: u64,
    pub framebuffer_count: u64,
    pub framebuffers: *const *const LimineFramebuffer,
}

#[repr(C)]
pub struct LimineFramebuffer {
    pub address: *mut u8,
    pub width: u64,
    pub height: u64,
    pub pitch: u64,
    pub bpp: u16,
    pub memory_model: u8,
    pub red_mask_size: u8,
    pub red_mask_shift: u8,
    pub green_mask_size: u8,
    pub green_mask_shift: u8,
    pub blue_mask_size: u8,
    pub blue_mask_shift: u8,
}

#[used]
#[link_section = ".limine_reqs"]
pub static FRAMEBUFFER_REQUEST: LimineRequest<LimineFramebufferResponse> =
    LimineRequest::new(0x9d5827dcd881dd75, 0xa3148604f6fab11b);

// Memory Map Request
#[repr(C)]
pub struct LimineMemoryMapResponse {
    pub revision: u64,
    pub entry_count: u64,
    pub entries: *const *const LimineMemoryMapEntry,
}

#[repr(C)]
pub struct LimineMemoryMapEntry {
    pub base: u64,
    pub length: u64,
    pub entry_type: u64,
}

#[allow(dead_code)]
pub const LIMINE_MEMMAP_USABLE: u64 = 0;
#[allow(dead_code)]
pub const LIMINE_MEMMAP_RESERVED: u64 = 1;
#[allow(dead_code)]
pub const LIMINE_MEMMAP_ACPI_RECLAIMABLE: u64 = 2;
#[allow(dead_code)]
pub const LIMINE_MEMMAP_ACPI_NVS: u64 = 3;
#[allow(dead_code)]
pub const LIMINE_MEMMAP_BAD_MEMORY: u64 = 4;
#[allow(dead_code)]
pub const LIMINE_MEMMAP_BOOTLOADER_RECLAIMABLE: u64 = 5;
#[allow(dead_code)]
pub const LIMINE_MEMMAP_KERNEL_AND_MODULES: u64 = 6;
#[allow(dead_code)]
pub const LIMINE_MEMMAP_FRAMEBUFFER: u64 = 7;

#[used]
#[link_section = ".limine_reqs"]
pub static MEMMAP_REQUEST: LimineRequest<LimineMemoryMapResponse> =
    LimineRequest::new(0x67cf3d9d378a806f, 0xe304acdfc50c3c62);

// Kernel Address Request
#[repr(C)]
pub struct LimineKernelAddressResponse {
    pub revision: u64,
    pub physical_base: u64,
    pub virtual_base: u64,
}

#[used]
#[link_section = ".limine_reqs"]
pub static KERNEL_ADDRESS_REQUEST: LimineRequest<LimineKernelAddressResponse> =
    LimineRequest::new(0x71ba76863cc55f63, 0xb2644a48c516a487);
