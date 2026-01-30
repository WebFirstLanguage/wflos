/// Kernel heap allocator
/// Provides dynamic memory allocation (Box, Vec, String, etc.)

use crate::memory::frame_allocator;
use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

const HEAP_SIZE: usize = 8 * 1024; // 8KB heap
const HEAP_FRAMES: usize = (HEAP_SIZE + 4095) / 4096; // Round up to frames (2 frames)

pub fn init(hhdm_offset: u64) -> Result<(), &'static str> {
    use crate::serial_println;

    serial_println!("  Allocating {} frames for heap...", HEAP_FRAMES);

    // Allocate contiguous frames (we need them to be sequential)
    let first_frame = frame_allocator::allocate_frame()
        .ok_or("Failed to allocate heap frame")?;

    serial_println!("  First frame at: {:#x}", first_frame);

    for i in 1..HEAP_FRAMES {
        if frame_allocator::allocate_frame().is_none() {
            return Err("Failed to allocate enough heap frames");
        }
    }

    // Calculate virtual address using HHDM (all physical memory mapped here)
    let heap_start_virt = (hhdm_offset as usize) + first_frame;
    serial_println!("  Heap virtual address: {:#x}", heap_start_virt);

    // Initialize the allocator
    unsafe {
        ALLOCATOR.lock().init(heap_start_virt as *mut u8, HEAP_SIZE);
    }

    serial_println!("  Allocator initialized");
    Ok(())
}

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout);
}
