use std::{process::Stdio, sync::Arc, time::Duration};

use tokio::{io::AsyncWriteExt, process::Command, time};

use crate::framebuffer::FrameBuffer;

pub struct FfmpegSink {
    fb: Arc<FrameBuffer>,
}

impl FfmpegSink {
    pub fn new(fb: Arc<FrameBuffer>) -> Self {
        FfmpegSink { fb }
    }

    pub async fn run(&self) -> tokio::io::Result<()> {
        let width = self.fb.get_width();
        let height = self.fb.get_height();
        let video_size: String = format!("{width}x{height}");

        // ffmpeg -re -stream_loop -1 -i simplescreenrecorder_2.mp4 -pix_fmt yuvj420p -x264-params keyint=48:min-keyint=48:scenecut=-1 -b:v 4500k -b:a 128k -ar 44100 -acodec aac -vcodec libx264 -preset medium -crf 28 -threads 4 -f flv rtmp://a.rtmp.youtube.com/live2/XXX
        // ffmpeg -re -stream_loop -1 -i simplescreenrecorder-2022-07-07_21.57.20.mp4 -c:a copy -c:v copy -f flv -flvflags no_duration_filesize rtmp://127.0.0.1:1935/live/test

        let args = [
            "-f",
            "rawvideo",
            "-pixel_format",
            "rgb0",
            "-video_size",
            &video_size,
            "-i",
            "-",
            "-vcodec",
            "libx264",
            // "-profile:v",
            // "high",
            // "-preset",
            // "veryfast",
            // "-crf",
            // "25",
            // "-g",
            // "60", // FIXME
            "-f",
            "flv",
            "-flvflags",
            "no_duration_filesize",
            "rtmp://127.0.0.1:1935/live/test",
        ];
        println!("ffmpeg {}", args.join(" "));
        let mut command = Command::new("ffmpeg")
            .args(args)
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();
        // command.wait().await?;

        // const ffmpegRTMPArgs =[
        // 	'-vcodec', 'libx264',
        // 	'-profile:v', 'high',
        // 	'-preset', 'veryfast',
        // 	'-crf', '25',
        // 	'-g', `${this.config.videoDestination.framerate * 2}`, // group of pictures forces an I-Frame every so-and-so frames.
        // 	'-f', 'flv',
        // 	'-flvflags', 'no_duration_filesize',
        // 	this.

        let mut stdin = command
            .stdin
            .take()
            .expect("child did not have a handle to stdin");

        // let mut interval = time::interval(Duration::from_nanos(1_000_000_000 / 30));
        let mut interval = time::interval(Duration::from_micros(1_000_000 / 30));
        loop {
            let bytes = self.fb.as_bytes();
            // dbg!(bytes.len());
            // dbg!(&bytes[0..50]);
            stdin.write_all(bytes).await?;
            // println!("Finished writing");
            interval.tick().await;
        }
    }
}
