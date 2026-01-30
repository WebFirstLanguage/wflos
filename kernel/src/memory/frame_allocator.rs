/// Physical frame allocator using bitmap
/// Manages 4KB physical memory frames
/// Properly handles non-contiguous memory regions from the bootloader memory map

use crate::limine::{LimineMemoryMapEntry, LIMINE_MEMMAP_USABLE};
use crate::sync::spinlock::Spinlock;

const FRAME_SIZE: usize = 4096;
const MAX_FRAMES: usize = 262144; // Support up to 1GB of RAM (256K frames)
const BITMAP_SIZE: usize = MAX_FRAMES / 8; // 32KB bitmap
const MAX_REGIONS: usize = 64;

#[derive(Clone, Copy)]
struct MemoryRegion {
    base: usize,
    frame_count: usize,
}

impl MemoryRegion {
    const fn empty() -> Self {
        MemoryRegion { base: 0, frame_count: 0 }
    }
}

pub struct FrameAllocator {
    bitmap: [u8; BITMAP_SIZE],
    total_frames: usize,
    used_frames: usize,
    regions: [MemoryRegion; MAX_REGIONS],
    region_count: usize,
    hhdm_offset: u64,
}

impl FrameAllocator {
    pub const fn new() -> Self {
        FrameAllocator {
            bitmap: [0; BITMAP_SIZE],
            total_frames: 0,
            used_frames: 0,
            regions: [MemoryRegion::empty(); MAX_REGIONS],
            region_count: 0,
            hhdm_offset: 0,
        }
    }

    /// Initialize allocator with memory map from Limine
    pub fn init(&mut self, memory_map: &[&LimineMemoryMapEntry], hhdm_offset: u64) {
        self.hhdm_offset = hhdm_offset;

        for entry in memory_map {
            if entry.entry_type == LIMINE_MEMMAP_USABLE && self.region_count < MAX_REGIONS {
                let frames = (entry.length as usize) / FRAME_SIZE;
                self.regions[self.region_count] = MemoryRegion {
                    base: entry.base as usize,
                    frame_count: frames,
                };
                self.region_count += 1;
                self.total_frames += frames;
            }
        }

        // Mark all frames as free initially (bitmap already zeroed)
    }

    /// Convert a bitmap frame index to a physical address by walking regions
    fn frame_index_to_phys(&self, index: usize) -> Option<usize> {
        let mut offset = 0;
        for i in 0..self.region_count {
            let region = &self.regions[i];
            if index < offset + region.frame_count {
                let frame_in_region = index - offset;
                return Some(region.base + frame_in_region * FRAME_SIZE);
            }
            offset += region.frame_count;
        }
        None
    }

    /// Convert a physical address to a bitmap frame index
    fn phys_to_frame_index(&self, phys_addr: usize) -> Option<usize> {
        let mut offset = 0;
        for i in 0..self.region_count {
            let region = &self.regions[i];
            let region_end = region.base + region.frame_count * FRAME_SIZE;
            if phys_addr >= region.base && phys_addr < region_end {
                let frame_in_region = (phys_addr - region.base) / FRAME_SIZE;
                return Some(offset + frame_in_region);
            }
            offset += region.frame_count;
        }
        None
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

                // Convert bitmap index to physical address via region walk
                return self.frame_index_to_phys(frame_index);
            }
        }

        None // Out of memory
    }

    /// Allocate N contiguous physical frames from a single region.
    /// Returns the physical address of the first frame.
    pub fn allocate_contiguous_frames(&mut self, count: usize) -> Option<usize> {
        if count == 0 {
            return None;
        }

        let mut region_start_index = 0;

        for r in 0..self.region_count {
            let region = &self.regions[r];

            if region.frame_count >= count {
                // Search within this region for `count` consecutive free frames
                let mut run_start = 0;
                let mut run_len = 0;

                for f in 0..region.frame_count {
                    let frame_index = region_start_index + f;
                    let byte_index = frame_index / 8;
                    let bit_index = frame_index % 8;

                    if self.bitmap[byte_index] & (1 << bit_index) == 0 {
                        if run_len == 0 {
                            run_start = f;
                        }
                        run_len += 1;
                        if run_len == count {
                            // Found enough contiguous frames — mark them all used
                            let base_frame_index = region_start_index + run_start;
                            for i in 0..count {
                                let idx = base_frame_index + i;
                                self.bitmap[idx / 8] |= 1 << (idx % 8);
                            }
                            self.used_frames += count;
                            return Some(region.base + run_start * FRAME_SIZE);
                        }
                    } else {
                        run_len = 0;
                    }
                }
            }

            region_start_index += region.frame_count;
        }

        None // Could not find enough contiguous frames
    }

    /// Deallocate a frame, returns it to the free pool
    pub fn deallocate_frame(&mut self, phys_addr: usize) {
        let frame_index = match self.phys_to_frame_index(phys_addr) {
            Some(idx) => idx,
            None => return, // Address doesn't belong to any known region
        };

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

pub fn allocate_contiguous_frames(count: usize) -> Option<usize> {
    FRAME_ALLOCATOR.lock().allocate_contiguous_frames(count)
}

pub fn deallocate_frame(phys_addr: usize) {
    FRAME_ALLOCATOR.lock().deallocate_frame(phys_addr);
}

pub fn stats() -> (usize, usize, usize) {
    let allocator = FRAME_ALLOCATOR.lock();
    (allocator.total_frames(), allocator.used_frames(), allocator.free_frames())
}
