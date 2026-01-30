/// Kernel heap allocator
/// Provides dynamic memory allocation (Box, Vec, String, etc.)

use crate::memory::frame_allocator;
use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

const HEAP_SIZE: usize = 64 * 1024; // 64KB heap
const HEAP_FRAMES: usize = (HEAP_SIZE + 4095) / 4096; // 16 frames

pub fn init(hhdm_offset: u64) -> Result<(), &'static str> {
    use crate::serial_println;

    serial_println!("  Allocating {} contiguous frames for heap...", HEAP_FRAMES);

    // Allocate contiguous frames in a single region
    let heap_phys = frame_allocator::allocate_contiguous_frames(HEAP_FRAMES)
        .ok_or("Failed to allocate contiguous heap frames")?;

    serial_println!("  Heap physical base: {:#x}", heap_phys);

    // Calculate virtual address using HHDM (all physical memory mapped here)
    let heap_start_virt = (hhdm_offset as usize) + heap_phys;
    serial_println!("  Heap virtual address: {:#x}", heap_start_virt);

    // Initialize the allocator
    unsafe {
        ALLOCATOR.lock().init(heap_start_virt as *mut u8, HEAP_SIZE);
    }

    serial_println!("  Allocator initialized ({} KB)", HEAP_SIZE / 1024);
    Ok(())
}

/// Verify heap works by performing a test allocation
pub fn verify_heap() {
    use crate::serial_println;
    use alloc::boxed::Box;

    let test_val = Box::new(0xDEAD_BEEFu64);
    if *test_val == 0xDEAD_BEEF {
        serial_println!("  Heap verification passed (Box<u64> = {:#x})", *test_val);
    } else {
        serial_println!("  Heap verification FAILED: unexpected value {:#x}", *test_val);
    }
    // Box is dropped here, returning memory to the allocator
}

/// Return heap statistics: (total_bytes, used_bytes, free_bytes)
pub fn stats() -> Option<(usize, usize, usize)> {
    let allocator = ALLOCATOR.lock();
    let free = allocator.free();
    let total = HEAP_SIZE;
    let used = total - free;
    Some((total, used, free))
}

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout);
}
