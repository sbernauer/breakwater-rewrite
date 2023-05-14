use breakwater::{args::Args, framebuffer::FrameBuffer, network::Network};
use clap::Parser;
use env_logger::Env;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let args = Args::parse();

    let fb = Arc::new(FrameBuffer::new(args.width, args.height));
    let fb_for_network = Arc::clone(&fb);
    let network = Network::new(args.listen_address, fb_for_network);

    let network_listener_thread = tokio::spawn(async move {
        network.listen().await.unwrap();
    });

    network_listener_thread.await?;

    Ok(())
}
