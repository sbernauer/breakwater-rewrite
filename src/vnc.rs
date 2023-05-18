use core::slice;
use std::sync::Arc;
use std::time::Duration;
use vncserver::*;

use crate::framebuffer::FrameBuffer;

const STATS_HEIGHT: usize = 35;

pub struct VncServer {
    fb: Arc<FrameBuffer>,
    screen: RfbScreenInfoPtr,
    target_fps: u32,
}

impl VncServer {
    pub fn new(fb: Arc<FrameBuffer>, port: u32, target_fps: u32) -> Self {
        let screen = rfb_get_screen(fb.get_width() as i32, fb.get_height() as i32, 8, 3, 4);
        unsafe {
            // We need to set bitsPerPixel and depth to the correct values,
            // otherwise some VNC clients (like gstreamer) won't work
            (*screen).bitsPerPixel = 32;
            (*screen).depth = 24;
            (*screen).serverFormat.depth = 24;
        }
        unsafe {
            (*screen).port = port as i32;
            (*screen).ipv6port = port as i32;
        }

        rfb_framebuffer_malloc(screen, (fb.get_size() * 4/* bytes per pixel */) as u64);
        rfb_init_server(screen);
        rfb_run_event_loop(screen, 1, 1);

        VncServer {
            fb,
            screen,
            target_fps,
        }
    }

    pub fn run(&self) {
        let target_loop_duration = Duration::from_millis(1_000 / self.target_fps as u64);

        let fb = &self.fb;
        let vnc_fb_slice: &mut [u32] = unsafe {
            slice::from_raw_parts_mut((*self.screen).frameBuffer as *mut u32, fb.get_size())
        };
        let fb_slice = unsafe { &*fb.get_buffer() };
        // A line less because the (height - STATS_SURFACE_HEIGHT) belongs to the stats and gets refreshed by them
        let height_up_to_stats_text = self.fb.get_height() - STATS_HEIGHT - 1;
        let fb_size_up_to_stats_text = fb.get_width() * height_up_to_stats_text;

        loop {
            let start = std::time::Instant::now();
            vnc_fb_slice[0..fb_size_up_to_stats_text]
                .copy_from_slice(&fb_slice[0..fb_size_up_to_stats_text]);

            // Only refresh the drawing surface, not the stats surface
            rfb_mark_rect_as_modified(
                self.screen,
                0,
                0,
                self.fb.get_width() as i32,
                height_up_to_stats_text as i32,
            );

            std::thread::sleep(target_loop_duration.saturating_sub(start.elapsed()));
        }
    }
}
