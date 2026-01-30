/// Kernel heap allocator
/// Provides dynamic memory allocation (Box, Vec, String, etc.)

use crate::memory::frame_allocator;
use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

const HEAP_SIZE: usize = 1024 * 1024; // 1MB heap
const HEAP_FRAMES: usize = HEAP_SIZE / 4096; // 256 frames

pub fn init(hhdm_offset: u64) -> Result<(), &'static str> {
    // Allocate frames for the heap
    let mut heap_start = None;

    for i in 0..HEAP_FRAMES {
        if let Some(frame) = frame_allocator::allocate_frame() {
            if i == 0 {
                heap_start = Some(frame);
            }
            // Frames should be contiguous for simplicity
            // In a real implementation, we'd map these to virtual addresses
        } else {
            return Err("Failed to allocate frames for heap");
        }
    }

    let heap_start = heap_start.ok_or("No frames allocated")?;

    // Calculate virtual address using HHDM
    let heap_start_virt = (hhdm_offset as usize) + heap_start;

    // Initialize the allocator
    unsafe {
        ALLOCATOR.lock().init(heap_start_virt as *mut u8, HEAP_SIZE);
    }

    Ok(())
}

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout);
}
