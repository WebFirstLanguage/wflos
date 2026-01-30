/// Hardware-agnostic ring buffer (circular buffer)
/// Thread-safe and testable on host system

use core::sync::atomic::{AtomicUsize, Ordering};

pub struct RingBuffer<T, const N: usize> {
    buffer: [Option<T>; N],
    read_pos: AtomicUsize,
    write_pos: AtomicUsize,
}

impl<T: Copy, const N: usize> RingBuffer<T, N> {
    pub const fn new() -> Self {
        RingBuffer {
            buffer: [None; N],
            read_pos: AtomicUsize::new(0),
            write_pos: AtomicUsize::new(0),
        }
    }

    /// Push item to buffer, returns false if buffer is full
    pub fn push(&mut self, item: T) -> bool {
        let write_pos = self.write_pos.load(Ordering::Relaxed);
        let read_pos = self.read_pos.load(Ordering::Relaxed);
        let next_write = (write_pos + 1) % N;

        // Check if buffer is full
        if next_write == read_pos {
            return false;
        }

        self.buffer[write_pos] = Some(item);
        self.write_pos.store(next_write, Ordering::Release);
        true
    }

    /// Pop item from buffer, returns None if buffer is empty
    pub fn pop(&mut self) -> Option<T> {
        let read_pos = self.read_pos.load(Ordering::Relaxed);
        let write_pos = self.write_pos.load(Ordering::Acquire);

        // Check if buffer is empty
        if read_pos == write_pos {
            return None;
        }

        let item = self.buffer[read_pos].take();
        let next_read = (read_pos + 1) % N;
        self.read_pos.store(next_read, Ordering::Release);
        item
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.read_pos.load(Ordering::Relaxed) == self.write_pos.load(Ordering::Relaxed)
    }

    /// Check if buffer is full
    pub fn is_full(&self) -> bool {
        let write_pos = self.write_pos.load(Ordering::Relaxed);
        let read_pos = self.read_pos.load(Ordering::Relaxed);
        (write_pos + 1) % N == read_pos
    }

    /// Get number of items in buffer
    pub fn len(&self) -> usize {
        let write_pos = self.write_pos.load(Ordering::Relaxed);
        let read_pos = self.read_pos.load(Ordering::Relaxed);

        if write_pos >= read_pos {
            write_pos - read_pos
        } else {
            N - read_pos + write_pos
        }
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.read_pos.store(0, Ordering::Relaxed);
        self.write_pos.store(0, Ordering::Relaxed);
        for i in 0..N {
            self.buffer[i] = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buffer_is_empty() {
        let buffer: RingBuffer<u8, 8> = RingBuffer::new();
        assert!(buffer.is_empty());
        assert!(!buffer.is_full());
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn test_push_and_pop() {
        let mut buffer: RingBuffer<u8, 8> = RingBuffer::new();

        assert!(buffer.push(1));
        assert!(!buffer.is_empty());
        assert_eq!(buffer.len(), 1);

        assert_eq!(buffer.pop(), Some(1));
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn test_multiple_push_pop() {
        let mut buffer: RingBuffer<u8, 8> = RingBuffer::new();

        assert!(buffer.push(1));
        assert!(buffer.push(2));
        assert!(buffer.push(3));
        assert_eq!(buffer.len(), 3);

        assert_eq!(buffer.pop(), Some(1));
        assert_eq!(buffer.pop(), Some(2));
        assert_eq!(buffer.pop(), Some(3));
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_buffer_full() {
        let mut buffer: RingBuffer<u8, 4> = RingBuffer::new();

        assert!(buffer.push(1));
        assert!(buffer.push(2));
        assert!(buffer.push(3));
        assert!(buffer.is_full());

        // Buffer full, push should fail
        assert!(!buffer.push(4));
        assert_eq!(buffer.len(), 3);
    }

    #[test]
    fn test_wrap_around() {
        let mut buffer: RingBuffer<u8, 4> = RingBuffer::new();

        // Fill buffer
        assert!(buffer.push(1));
        assert!(buffer.push(2));
        assert!(buffer.push(3));
        assert!(buffer.is_full());

        // Remove items
        assert_eq!(buffer.pop(), Some(1));
        assert_eq!(buffer.pop(), Some(2));

        // Add more (wraps around)
        assert!(buffer.push(4));
        assert!(buffer.push(5));

        // Check order is correct
        assert_eq!(buffer.pop(), Some(3));
        assert_eq!(buffer.pop(), Some(4));
        assert_eq!(buffer.pop(), Some(5));
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_pop_empty() {
        let mut buffer: RingBuffer<u8, 8> = RingBuffer::new();
        assert_eq!(buffer.pop(), None);
    }

    #[test]
    fn test_clear() {
        let mut buffer: RingBuffer<u8, 8> = RingBuffer::new();

        buffer.push(1);
        buffer.push(2);
        buffer.push(3);
        assert_eq!(buffer.len(), 3);

        buffer.clear();
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn test_fifo_order() {
        let mut buffer: RingBuffer<char, 16> = RingBuffer::new();

        let test_data = ['H', 'E', 'L', 'L', 'O'];
        for &ch in &test_data {
            assert!(buffer.push(ch));
        }

        for &expected in &test_data {
            assert_eq!(buffer.pop(), Some(expected));
        }

        assert!(buffer.is_empty());
    }
}
