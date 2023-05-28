use simple_moving_average::{SingleSumSMA, SMA};
use std::{
    cmp::max,
    collections::{hash_map::Entry, HashMap},
    net::IpAddr,
    time::{Duration, Instant},
};
use tokio::sync::{broadcast, mpsc::Receiver};

pub const STATS_REPORT_INTERVAL: Duration = Duration::from_millis(1000);
pub const STATS_SLIDING_WINDOW_SIZE: usize = 5;

#[derive(Debug)]
pub enum StatisticsEvent {
    ConnectionCreated { ip: IpAddr },
    ConnectionClosed { ip: IpAddr },
    BytesRead { ip: IpAddr, bytes: u64 },
    FrameRendered,
}

#[derive(Clone, Debug, Default)]
pub struct StatisticsInformationEvent {
    pub frame: u64,
    pub connections: u32,
    pub ips: u32,
    pub legacy_ips: u32,
    pub bytes: u64,
    pub fps: u64,
    pub bytes_per_s: u64,

    pub connections_for_ip: HashMap<IpAddr, u32>,
    pub bytes_for_ip: HashMap<IpAddr, u64>,

    pub statistic_events: u64,
}

pub struct Statistics {
    statistics_rx: Receiver<StatisticsEvent>,
    statistics_information_tx: broadcast::Sender<StatisticsInformationEvent>,
    statistic_events: u64,

    frame: u64,
    connections_for_ip: HashMap<IpAddr, u32>,
    bytes_for_ip: HashMap<IpAddr, u64>,

    bytes_per_s_window: SingleSumSMA<u64, u64, STATS_SLIDING_WINDOW_SIZE>,
    fps_window: SingleSumSMA<u64, u64, STATS_SLIDING_WINDOW_SIZE>,
}

impl Statistics {
    pub fn new(
        statistics_rx: Receiver<StatisticsEvent>,
        statistics_information_tx: broadcast::Sender<StatisticsInformationEvent>,
    ) -> Self {
        Statistics {
            statistics_rx,
            statistics_information_tx,
            statistic_events: 0,
            frame: 0,
            connections_for_ip: HashMap::new(),
            bytes_for_ip: HashMap::new(),
            bytes_per_s_window: SingleSumSMA::new(),
            fps_window: SingleSumSMA::new(),
        }
    }

    pub async fn start(&mut self) {
        let mut start = Instant::now();
        let mut prev_statistics_information_event = StatisticsInformationEvent::default();

        while let Some(statistics_update) = self.statistics_rx.recv().await {
            self.statistic_events += 1;
            match statistics_update {
                StatisticsEvent::ConnectionCreated { ip } => {
                    *self.connections_for_ip.entry(ip).or_insert(0) += 1;
                }
                StatisticsEvent::ConnectionClosed { ip } => {
                    if let Entry::Occupied(mut o) = self.connections_for_ip.entry(ip) {
                        let connections = o.get_mut();
                        *connections -= 1;
                        if *connections == 0 {
                            o.remove_entry();
                        }
                    }
                }
                StatisticsEvent::BytesRead { ip, bytes } => {
                    *self.bytes_for_ip.entry(ip).or_insert(0) += bytes;
                }
                StatisticsEvent::FrameRendered => self.frame += 1,
            }

            // As there is an event for every frame we are guaranteed to land here every second
            let elapsed = start.elapsed();
            if elapsed > STATS_REPORT_INTERVAL {
                start = Instant::now();
                prev_statistics_information_event = self.calculate_statistics_information_event(
                    prev_statistics_information_event,
                    elapsed,
                );
                self.statistics_information_tx
                    .send(prev_statistics_information_event.clone())
                    .expect("Statistics information channel full (or disconnected)");
            }
        }
    }

    fn calculate_statistics_information_event(
        &mut self,
        prev: StatisticsInformationEvent,
        elapsed: Duration,
    ) -> StatisticsInformationEvent {
        let elapsed_ms = max(1, elapsed.as_millis()) as u64;
        let frame = self.frame;
        let connections = self.connections_for_ip.values().sum();
        let ips = self.connections_for_ip.len() as u32;
        let legacy_ips = self
            .connections_for_ip
            .keys()
            .filter(|ip| ip.is_ipv4())
            .count() as u32;
        let bytes = self.bytes_for_ip.values().sum();
        self.bytes_per_s_window
            .add_sample((bytes - prev.bytes) * 1000 / elapsed_ms);
        self.fps_window
            .add_sample((frame - prev.frame) * 1000 / elapsed_ms);
        let statistic_events = self.statistic_events;

        StatisticsInformationEvent {
            frame,
            connections,
            ips,
            legacy_ips,
            bytes,
            fps: self.fps_window.get_average(),
            bytes_per_s: self.bytes_per_s_window.get_average(),
            connections_for_ip: self.connections_for_ip.clone(),
            bytes_for_ip: self.bytes_for_ip.clone(),
            statistic_events,
        }
    }
}
