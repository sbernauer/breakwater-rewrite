use std::{cell::UnsafeCell, slice};

pub struct FrameBuffer {
    width: usize,
    height: usize,
    buffer: UnsafeCell<Vec<u32>>,
}

// FIXME Nothing to see here, I don't know what I'm doing ¯\_(ツ)_/¯
unsafe impl Sync for FrameBuffer {}

const INTERNAL_FRAMEBUFFER_SIZE: usize = 2_usize.pow(14);

impl FrameBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let mut buffer = Vec::with_capacity(INTERNAL_FRAMEBUFFER_SIZE.pow(2));
        buffer.resize_with(buffer.capacity(), || 0);
        FrameBuffer {
            width,
            height,
            buffer: UnsafeCell::from(buffer),
        }
    }

    pub fn get_width(&self) -> usize {
        self.width
    }

    pub fn get_height(&self) -> usize {
        self.height
    }

    pub fn get_size(&self) -> usize {
        self.width * self.height
    }

    #[inline(always)]
    pub fn get(&self, x: usize, y: usize) -> Option<u32> {
        if x < self.width && y < self.height {
            unsafe { Some((*self.buffer.get())[x + (y << 14)]) }
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn set(&self, x: usize, y: usize, rgba: u32) {
        // This function is using ~1% according to a flamegraph
        unsafe { (*self.buffer.get())[x + (y  << 14)] = rgba }
    }

    pub fn get_buffer(&self) -> *mut Vec<u32> {
        // TODO: rewrite for oversized framebuffer
        self.buffer.get()
    }

    pub fn as_bytes(&self) -> &[u8] {
        // TODO: rewrite for oversized framebuffer
        let buffer = self.buffer.get();
        let len_in_bytes: usize = unsafe { (*buffer).len() } * 4;

        unsafe { slice::from_raw_parts((*buffer).as_ptr() as *const u8, len_in_bytes) }
    }
}
