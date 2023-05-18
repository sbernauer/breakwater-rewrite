use breakwater::{args::Args, framebuffer::FrameBuffer, network::Network, vnc::VncServer};
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

    let fb_for_vnc_server = Arc::clone(&fb);
    // TODO Use tokio::spawn instead of std::thread::spawn
    // I was not able to get to work with async closure
    let vnc_server_thread = std::thread::spawn(move || {
        let vnc_server = VncServer::new(fb_for_vnc_server, args.vnc_port, args.fps);
        vnc_server.run();
    });

    network_listener_thread.await?;
    vnc_server_thread
        .join()
        .expect("Failed to join VNC server thread");

    Ok(())
}
