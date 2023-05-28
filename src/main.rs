use breakwater::{
    args::Args,
    framebuffer::FrameBuffer,
    network::Network,
    prometheus_exporter::PrometheusExporter,
    statistics::{Statistics, StatisticsEvent, StatisticsInformationEvent},
    vnc::VncServer,
};
use clap::Parser;
use env_logger::Env;
use std::sync::Arc;
use thread_priority::{ThreadBuilderExt, ThreadPriority};
use tokio::sync::{broadcast, mpsc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let args = Args::parse();

    let fb = Arc::new(FrameBuffer::new(args.width, args.height));

    // If we make the channel to big, stats will start to lag behind
    // TODO: Check performance impact in real-world scenario. Maybe the statistics thread blocks the other threads
    let (statistics_tx, statistics_rx) = mpsc::channel::<StatisticsEvent>(100);
    let (statistics_information_tx, statistics_information_rx_for_vnc_server) =
        broadcast::channel::<StatisticsInformationEvent>(2);
    let statistics_information_rx_for_prometheus_exporter = statistics_information_tx.subscribe();

    let mut statistics = Statistics::new(statistics_rx, statistics_information_tx);

    let fb_for_network = Arc::clone(&fb);
    let network = Network::new(args.listen_address, fb_for_network, statistics_tx.clone());

    let network_listener_thread = tokio::spawn(async move {
        network.listen().await.unwrap();
    });

    let fb_for_vnc_server = Arc::clone(&fb);
    // TODO Use tokio::spawn instead of std::thread::spawn
    // I was not able to get to work with async closure
    // We than also need to think about setting a priority
    let vnc_server_thread = std::thread::Builder::new()
        .name("breakwater vnc server thread".to_owned())
        .spawn_with_priority(
            ThreadPriority::Crossplatform(70.try_into().expect("Failed to get cross-platform ThreadPriority. Please report this error message together with your operating system.")),
            move |_| {
                let mut vnc_server = VncServer::new(
                    fb_for_vnc_server,
                    args.vnc_port,
                    args.fps,
                    statistics_tx,
                    statistics_information_rx_for_vnc_server,
                    &args.text,
                    &args.font,
                );
                vnc_server.run();
            },
        )
        .unwrap();

    let statistics_thread = tokio::spawn(async move {
        statistics.start().await;
    });

    let mut prometheus_exporter = PrometheusExporter::new(
        &args.prometheus_listen_address,
        statistics_information_rx_for_prometheus_exporter,
    );
    let prometheus_exporter_thread = tokio::spawn(async move {
        prometheus_exporter.run().await;
    });

    prometheus_exporter_thread.await?;
    network_listener_thread.await?;
    statistics_thread.await?;
    vnc_server_thread
        .join()
        .expect("Failed to join VNC server thread");

    Ok(())
}
