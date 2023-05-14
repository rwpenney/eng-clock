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
    stats::BayesOffset };


const NTP_SERVERS: [&str; 8] = [
    "0.uk.pool.ntp.org",
    "2.uk.pool.ntp.org",
    "1.europe.pool.ntp.org",
    "3.europe.pool.ntp.org",
    "ntp2d.mcc.ac.uk",
    "1.asia.pool.ntp.org",
    "2.north-america.pool.ntp.org",
    "3.debian.pool.ntp.org"
    // FIXME - make NTP servers configurable
];


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
    ntp_servers: Vec<String>,
    stats: BayesOffset
}

impl OffsetEstimator {
    pub fn new(tkr_channel: mpsc::Sender<OffsetEvent>,
               ui_channel: UIsender) -> OffsetEstimator {
        OffsetEstimator {
            tkr_channel,
            ui_channel,
            ntp_servers: NTP_SERVERS.into_iter()
                                    .map(|h| String::from(h)).collect(),
            stats: BayesOffset::new(30.0)
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
            if let Ok(sync) = self.try_ntp_pings(&wrapped_skt, &ntp_ctxt, 3) {
                println!("{:?}", sync);
                // ping.offset should be *added* to local clock to approximate reference time

                let obs_time = utc_now();
                self.stats.add_observation(sync.offset as f32 * 1e-6,
                                           sync.roundtrip as f32 * 0.25e-6 +
                                            2.0f32.powi(sync.precision as i32),
                                           obs_time);
                // Heuristically assume that the offset margin of error
                // is about a quarter of the round-trip time

                let offs = OffsetEvent {
                    avg_offset: self.stats.avg_offset(),
                    stddev_offset: self.stats.stddev_offset(obs_time) };

                self.tkr_channel.send(offs).unwrap();
                self.ui_channel.send(UImessage::Offset(offs)).unwrap();
            }

            thread::sleep(std::time::Duration::from_secs_f64(17.0));
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

        err.unwrap().expect("Missing failure")
    }

    fn ntp_ping<T>(&self, skt: UdpSocketWrapper,
                   ctxt: NtpContext<T>) -> sntpc::Result<NtpResult>
            where T: NtpTimestampGenerator + Copy {
        // See https://datatracker.ietf.org/doc/html/rfc5905#section-7.3
        let host = &self.ntp_servers[weak_rand() as usize % self.ntp_servers.len()];
        sntpc::get_time((host.as_str(), 123u16), skt, ctxt)
    }
}

// (C)Copyright 2023, RW Penney
