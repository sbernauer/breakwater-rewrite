use args::Args;
use clap::Parser;
use framebuffer::FrameBuffer;
use network::Network;
use std::{sync::Arc, time::Duration};

mod args;
mod framebuffer;
mod network;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let fb = Arc::new(FrameBuffer::new(args.width, args.height));
    let fb_for_network = Arc::clone(&fb);
    let network = Network::new(args.listen_address, fb_for_network);

    tokio::spawn(async move {
        network.listen().await.unwrap();
    });

    loop {
        println!("{}", fb.get_unchecked(0, 0));
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
