/*
 *  Clock offset estimation tools for eng-clock
 *  RW Penney, May 2023
 */

use sntpc::{ NtpContext, NtpResult, NtpTimestampGenerator };
use std::{
    net::{ UdpSocket },
    sync::mpsc,
    thread };
use crate::{
    OffsetEvent, Timestamp, UImessage, UIsender, utc_now, weak_rand,
    config::SyncConfig,
    stats::BayesOffset };


#[derive(Clone, Copy, Default)]
struct StdTimestampGen {
    t: Timestamp
}

impl NtpTimestampGenerator for StdTimestampGen {
    fn init(&mut self) {
        self.t = utc_now();
    }

    fn timestamp_sec(&self) -> u64 {
        self.t.timestamp() as u64
    }

    fn timestamp_subsec_micros(&self) -> u32 {
        self.t.timestamp_subsec_micros()
    }
}


pub struct OffsetEstimator {
    tkr_channel: mpsc::Sender<OffsetEvent>,
    ui_channel: UIsender,

    /// Time between wakeups, in seconds
    wakeup_interval: f32,

    /// Bayesian statistical model of clock-offset
    stats: BayesOffset,

    /// A collection of NTP server hostname:port addresses
    ntp_servers: Vec<String>,

    /// The desired maximum uncertainty in the clock-offset, in seconds
    target_precision: f32
}

impl OffsetEstimator {
    pub const DEFAULT_TGT_PRECISION: f32 = 0.03;
    pub const DEFAULT_WAKEUP_ITVL: f32 = 11.0;

    pub fn new(tkr_channel: mpsc::Sender<OffsetEvent>, ui_channel: UIsender,
               config: &SyncConfig) -> OffsetEstimator {

        let ntp_servers =
            config.ntp_servers.iter()
                  .map(|h| format!("{h}:123"))
                  .collect::<Vec<_>>();

        OffsetEstimator {
            tkr_channel,
            ui_channel,
            wakeup_interval: config.wakeup_interval,
            stats: BayesOffset::new(30.0),
            ntp_servers,
            target_precision: config.target_precision
        }
    }

    /// Entry-point for clock-offset thread communicating via message queues
    pub fn run(&mut self) {
        let skt = UdpSocket::bind("0.0.0.0:0")
                    .expect("Failed to bind UDP socket");
        skt.set_read_timeout(Some(std::time::Duration::from_secs_f64(2.5)))
           .expect("Failed to set UDP timeout");
        let ntp_ctxt = NtpContext::new(StdTimestampGen::default());
        let mut tick_count: u32 = 0;

        loop {
            let warmup = tick_count < 3;

            let tick_time = self.check_precision(&skt, &ntp_ctxt, warmup);
            tick_count += 1;

            let offs = OffsetEvent {
                avg_offset: self.stats.avg_offset(),
                stddev_offset: self.stats.stddev_offset(tick_time) };

            self.tkr_channel.send(offs).unwrap();
            self.ui_channel.send(UImessage::Offset(offs)).unwrap();

            let pause = if !warmup {
                std::time::Duration::from_secs_f32(self.wakeup_interval)
            } else {
                std::time::Duration::from_millis(500)
            };
            thread::sleep(pause);
        }
    }

    fn check_precision<T>(&mut self, skt: &UdpSocket,
                          ctxt: &NtpContext<T>, force_ping: bool) -> Timestamp
            where T: NtpTimestampGenerator + Copy {
        let now = utc_now();

        // Check if uncertainty in clock-offset is still acceptably small:
        if !force_ping && self.stats.stddev_offset(now) < self.target_precision {
            return now;
        }

        if let Ok(sync) = self.try_ntp_pings(skt, ctxt, 3) {
            let obs_time = utc_now();
            self.stats.add_observation(sync.offset as f32 * 1e-6,
                                       sync.roundtrip as f32 * 0.25e-6 +
                                            2.0f32.powi(sync.precision as i32),
                                       obs_time);
            // Heuristically assume that the offset margin of error
            // is about a quarter of the round-trip time
            obs_time
        } else {
            utc_now()
        }
    }

    fn try_ntp_pings<T>(&self, skt: &UdpSocket, ctxt: &NtpContext<T>,
                        attempts: u8) -> sntpc::Result<NtpResult>
            where T: NtpTimestampGenerator + Copy {
        let mut err = None;

        for _ in 0 .. attempts {
            match self.ntp_ping(skt, *ctxt) {
                Ok(ping) => return Ok(ping),
                Err(e) =>   if err.is_none() {
                                err = Some(Err(e)) }
            }
        }

        err.expect("Missing failure")
    }

    fn ntp_ping<T>(&self, skt: &UdpSocket,
                   ctxt: NtpContext<T>) -> sntpc::Result<NtpResult>
            where T: NtpTimestampGenerator + Copy {
        // See https://datatracker.ietf.org/doc/html/rfc5905#section-7.3

        use std::net::ToSocketAddrs;

        let servers = &self.ntp_servers;
        let tgt_server = &servers[weak_rand() as usize % servers.len()];

        let remote =
            tgt_server.to_socket_addrs()
                      .map_err(|_err| sntpc::Error::AddressResolve)?
                      .next()
                      .ok_or(sntpc::Error::AddressResolve)?;

        // FIXME - promoto runtime to parent
        let runtime = tokio::runtime::Builder::new_current_thread().build().unwrap();
        runtime.block_on(async {
            sntpc::get_time(remote, skt, ctxt).await
        })
        // ping.offset should be *added* to local clock to approximate reference time
    }
}

// (C)Copyright 2023-2026, RW Penney
