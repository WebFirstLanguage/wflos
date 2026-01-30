/// Physical frame allocator using bitmap
/// Manages 4KB physical memory frames

use crate::limine::{LimineMemoryMapEntry, LIMINE_MEMMAP_USABLE};
use crate::sync::spinlock::Spinlock;

const FRAME_SIZE: usize = 4096;
const MAX_FRAMES: usize = 262144; // Support up to 1GB of RAM (256K frames)
const BITMAP_SIZE: usize = MAX_FRAMES / 8; // 32KB bitmap

pub struct FrameAllocator {
    bitmap: [u8; BITMAP_SIZE],
    total_frames: usize,
    used_frames: usize,
    base_address: usize,
    hhdm_offset: u64,
}

impl FrameAllocator {
    pub const fn new() -> Self {
        FrameAllocator {
            bitmap: [0; BITMAP_SIZE],
            total_frames: 0,
            used_frames: 0,
            base_address: 0,
            hhdm_offset: 0,
        }
    }

    /// Initialize allocator with memory map from Limine
    pub fn init(&mut self, memory_map: &[&LimineMemoryMapEntry], hhdm_offset: u64) {
        self.hhdm_offset = hhdm_offset;

        // Find the first usable region for our base
        for entry in memory_map {
            if entry.entry_type == LIMINE_MEMMAP_USABLE {
                if self.base_address == 0 {
                    self.base_address = entry.base as usize;
                }

                // Count frames in this region
                let frames = (entry.length as usize) / FRAME_SIZE;
                self.total_frames += frames;
            }
        }

        // Mark all frames as free initially (bitmap already zeroed)
    }

    /// Allocate a single frame, returns physical address
    pub fn allocate_frame(&mut self) -> Option<usize> {
        // Find first free frame
        for frame_index in 0..self.total_frames {
            let byte_index = frame_index / 8;
            let bit_index = frame_index % 8;

            if self.bitmap[byte_index] & (1 << bit_index) == 0 {
                // Frame is free, mark as used
                self.bitmap[byte_index] |= 1 << bit_index;
                self.used_frames += 1;

                // Calculate physical address
                let phys_addr = self.base_address + (frame_index * FRAME_SIZE);
                return Some(phys_addr);
            }
        }

        None // Out of memory
    }

    /// Deallocate a frame, returns it to the free pool
    pub fn deallocate_frame(&mut self, phys_addr: usize) {
        if phys_addr < self.base_address {
            return; // Invalid address
        }

        let frame_index = (phys_addr - self.base_address) / FRAME_SIZE;
        if frame_index >= self.total_frames {
            return; // Out of range
        }

        let byte_index = frame_index / 8;
        let bit_index = frame_index % 8;

        // Mark as free
        if self.bitmap[byte_index] & (1 << bit_index) != 0 {
            self.bitmap[byte_index] &= !(1 << bit_index);
            self.used_frames -= 1;
        }
    }

    pub fn total_frames(&self) -> usize {
        self.total_frames
    }

    pub fn used_frames(&self) -> usize {
        self.used_frames
    }

    pub fn free_frames(&self) -> usize {
        self.total_frames - self.used_frames
    }
}

static FRAME_ALLOCATOR: Spinlock<FrameAllocator> = Spinlock::new(FrameAllocator::new());

pub fn init(memory_map: &[&LimineMemoryMapEntry], hhdm_offset: u64) {
    FRAME_ALLOCATOR.lock().init(memory_map, hhdm_offset);
}

pub fn allocate_frame() -> Option<usize> {
    FRAME_ALLOCATOR.lock().allocate_frame()
}

pub fn deallocate_frame(phys_addr: usize) {
    FRAME_ALLOCATOR.lock().deallocate_frame(phys_addr);
}

pub fn stats() -> (usize, usize, usize) {
    let allocator = FRAME_ALLOCATOR.lock();
    (allocator.total_frames(), allocator.used_frames(), allocator.free_frames())
}
