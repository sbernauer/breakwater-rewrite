use std::{process::Stdio, sync::Arc, time::Duration};

use log::debug;
use tokio::{io::AsyncWriteExt, process::Command, time};

use crate::framebuffer::FrameBuffer;

pub struct FfmpegSink {
    fb: Arc<FrameBuffer>,
}

impl FfmpegSink {
    pub fn new(fb: Arc<FrameBuffer>) -> Self {
        FfmpegSink { fb }
    }

    pub async fn run(&self, rtmp_address: &str) -> tokio::io::Result<()> {
        let video_size: String = format!("{}x{}", self.fb.get_width(), self.fb.get_height());

        let ffmpeg_args = [
            // Video input
            "-f",
            "rawvideo",
            "-pixel_format",
            "rgb0",
            "-video_size",
            &video_size,
            "-i",
            "-",
            // Audio input
            "-f",
            "lavfi",
            "-i",
            "anullsrc=channel_layout=stereo:sample_rate=44100",
            // Output
            "-vcodec",
            "libx264",
            "-acodec",
            "aac",
            "-pix_fmt",
            "yuv420p",
            "-x264-params",
            "keyint=48:min-keyint=48:scenecut=-1",
            "-preset",
            "fast", // ultrafast, superfast, veryfast, faster, fast, medium – default preset, slow, slower, veryslow
            "-crf",
            "28",
            "-r",
            "30",
            "-g",
            "60",
            "-ar",
            "44100",
            "-b:v",
            "4500k",
            "-b:a",
            "128k",
            "-threads",
            "8",
            "-f",
            "flv",
            rtmp_address,
        ];
        debug!("ffmpeg {}", ffmpeg_args.join(" "));
        let mut command = Command::new("ffmpeg")
            .args(ffmpeg_args)
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();

        let mut stdin = command
            .stdin
            .take()
            .expect("child did not have a handle to stdin");

        let mut interval = time::interval(Duration::from_micros(1_000_000 / 30));
        loop {
            let bytes = self.fb.as_bytes();
            stdin.write_all(bytes).await?;
            interval.tick().await;
        }
    }
}
