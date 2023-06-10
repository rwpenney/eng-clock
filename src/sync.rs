/*
 *  Clock offset estimation tools for eng-clock
 *  RW Penney, May 2023
 */

use sntpc::{ NtpContext, NtpResult, NtpTimestampGenerator, NtpUdpSocket };
use std::{
    net::{ SocketAddr, ToSocketAddrs, UdpSocket },
    rc::Rc,
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

#[derive(Clone, Debug)]
struct UdpSocketWrapper {
    skt: Rc<UdpSocket>
}

impl NtpUdpSocket for UdpSocketWrapper {
    fn send_to<T: ToSocketAddrs>(&self, buff: &[u8], addr: T) -> sntpc::Result<usize> {
        match self.skt.send_to(buff, addr) {
            Ok(usize) => Ok(usize),
            Err(_) => Err(sntpc::Error::Network)
        }
    }

    fn recv_from(&self, buff: &mut [u8]) -> sntpc::Result<(usize, SocketAddr)> {
        match self.skt.recv_from(buff) {
            Ok((size, addr)) => Ok((size, addr)),
            Err(_) => Err(sntpc::Error::Network)
        }
    }
}


pub struct OffsetEstimator {
    tkr_channel: mpsc::Sender<OffsetEvent>,
    ui_channel: UIsender,

    /// Time between wakeups, in seconds
    wakeup_interval: f32,

    /// Bayesian statistical model of clock-offset
    stats: BayesOffset,

    /// A collection of NTP server hostnames
    ntp_servers: Vec<String>,

    /// The desired maximum uncertainty in the clock-offset, in seconds
    target_precision: f32
}

impl OffsetEstimator {
    pub const DEFAULT_TGT_PRECISION: f32 = 0.03;
    pub const DEFAULT_WAKEUP_ITVL: f32 = 11.0;

    pub fn new(tkr_channel: mpsc::Sender<OffsetEvent>, ui_channel: UIsender,
               config: &SyncConfig) -> OffsetEstimator {
        OffsetEstimator {
            tkr_channel,
            ui_channel,
            wakeup_interval: config.wakeup_interval,
            stats: BayesOffset::new(30.0),
            ntp_servers: config.ntp_servers.clone(),
            target_precision: config.target_precision
        }
    }

    /// Entry-point for clock-offset thread communicating via message queues
    pub fn run(&mut self) {
        let skt = UdpSocket::bind("0.0.0.0:0")
                    .expect("Failed to bind UDP socket");
        skt.set_read_timeout(Some(std::time::Duration::from_secs_f64(2.5)))
           .expect("Failed to set UDP timeout");
        let wrapped_skt = UdpSocketWrapper { skt: Rc::new(skt) };
        let ntp_ctxt = NtpContext::new(StdTimestampGen::default());

        loop {
            let tick_time = self.check_precision(&wrapped_skt, &ntp_ctxt);

            let offs = OffsetEvent {
                avg_offset: self.stats.avg_offset(),
                stddev_offset: self.stats.stddev_offset(tick_time) };

            self.tkr_channel.send(offs).unwrap();
            self.ui_channel.send(UImessage::Offset(offs)).unwrap();

            thread::sleep(std::time::Duration::from_secs_f32(self.wakeup_interval));
        }
    }

    fn check_precision<T>(&mut self, skt: &UdpSocketWrapper,
                          ctxt: &NtpContext<T>) -> Timestamp
            where T: NtpTimestampGenerator + Copy {
        let now = utc_now();

        // Check if uncertainty in clock-offset is still acceptably small:
        if self.stats.stddev_offset(now) < self.target_precision {
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

    fn try_ntp_pings<T>(&self, skt: &UdpSocketWrapper, ctxt: &NtpContext<T>,
                        attempts: u8) -> sntpc::Result<NtpResult>
            where T: NtpTimestampGenerator + Copy {
        let mut err = None;

        for _ in 0 .. attempts {
            match self.ntp_ping(skt.clone(), ctxt.clone()) {
                Ok(ping) => return Ok(ping),
                Err(e) =>   if err.is_none() {
                                err = Some(Err(e)) }
            }
        }

        err.expect("Missing failure")
    }

    fn ntp_ping<T>(&self, skt: UdpSocketWrapper,
                   ctxt: NtpContext<T>) -> sntpc::Result<NtpResult>
            where T: NtpTimestampGenerator + Copy {
        // See https://datatracker.ietf.org/doc/html/rfc5905#section-7.3
        let servers = &self.ntp_servers;
        let host = &servers[weak_rand() as usize % servers.len()];
        sntpc::get_time((host.as_str(), 123u16), skt, ctxt)
        // ping.offset should be *added* to local clock to approximate reference time
    }
}

// (C)Copyright 2023, RW Penney
