use std::cell::UnsafeCell;

pub struct FrameBuffer {
    pub width: usize,
    pub height: usize,
    buffer: UnsafeCell<Vec<u32>>,
}

// Nothing to see here, I don't know what I'm doing ¯\_(ツ)_/¯
unsafe impl Sync for FrameBuffer {}

impl FrameBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let mut buffer = Vec::with_capacity(width * height);
        buffer.resize_with(width * height, || 0);
        FrameBuffer {
            width,
            height,
            buffer: UnsafeCell::from(buffer),
        }
    }

    #[inline(always)]
    pub fn get(&self, x: usize, y: usize) -> Option<u32> {
        if x < self.width && y < self.height {
            unsafe { Some((*self.buffer.get())[x + y * self.width]) }
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn get_unchecked(&self, x: usize, y: usize) -> u32 {
        unsafe { (*self.buffer.get())[x + y * self.width] }
    }

    #[inline(always)]
    pub fn set(&self, x: usize, y: usize, rgba: u32) {
        if x < self.width && y < self.height {
            unsafe { (*self.buffer.get())[x + y * self.width] = rgba }
        }
    }
}
