use std::{sync::Arc, time::Duration};

use framebuffer::FrameBuffer;
use network::Network;

mod framebuffer;
mod network;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fb = Arc::new(FrameBuffer::new(100, 100));
    let fb_for_network = Arc::clone(&fb);
    let network = Network::new("[::]:1234", fb_for_network);

    tokio::spawn(async move {
        network.listen().await.unwrap();
    });

    loop {
        println!("{}", fb.get(0, 0));
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
