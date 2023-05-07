/*
 *  Clock offset estimation tools for eng-clock
 *  RW Penney, May 2023
 */

use chrono::{ Duration, Local, TimeZone };
use ntp::{
    formats::timestamp::{ EPOCH_DELTA, TimestampFormat } };
use std::sync::mpsc;
use std::thread;
use crate::{ LocalTimestamp, OffsetEvent, UImessage, UIsender, weak_rand };


const NTP_SERVERS: [&str; 4] = [
    "2.uk.pool.ntp.org",
    "1.europe.pool.ntp.org",
    "ntp2d.mcc.ac.uk",
    "3.debian.pool.ntp.org"
];


#[derive(Clone, Copy, Debug)]
struct NtpPacket {
    t0: LocalTimestamp,
    t1: LocalTimestamp,
    t2: LocalTimestamp,
    t3: LocalTimestamp,
}

impl NtpPacket {
    fn from(pkt: ntp::packet::Packet,
            t_return: LocalTimestamp) -> Option<NtpPacket> {
        let t0 = Self::tx_timestamp(pkt.orig_time)?;
        let t1 = Self::tx_timestamp(pkt.recv_time)?;
        let t2 = Self::tx_timestamp(pkt.transmit_time)?;

        // FIXME - extract precision & estimate overall error margin

        Some(NtpPacket {
            t0, t1, t2,
            t3: t_return
        })
    }

    /// Compute offset of local clock from reference
    fn offset(&self) -> Duration {
        let o2 = (self.t3 - self.t2) - (self.t1 - self.t0);
        Duration::microseconds(o2.num_microseconds().unwrap_or(1<<60) / 2)
    }

    /// Compute round-trip communications delay
    fn transit(&self) -> Duration {
        (self.t3 - self.t2) + (self.t1 - self.t0)
    }

    fn tx_timestamp(t: TimestampFormat) -> Option<LocalTimestamp> {
        Local.timestamp_opt(t.sec as i64 - EPOCH_DELTA,
                            t.frac as u32).single()
        // FIXME - NTP epoch rollover occurs in February 2036
    }
}


pub struct OffsetEstimator {
    tkr_channel: mpsc::Sender<OffsetEvent>,
    ui_channel: UIsender,
    ntp_servers: Vec<String>
}

impl OffsetEstimator {
    pub fn new(tkr_channel: mpsc::Sender<OffsetEvent>,
               ui_channel: UIsender) -> OffsetEstimator {
        OffsetEstimator {
            tkr_channel,
            ui_channel,
            ntp_servers: NTP_SERVERS.into_iter().map(|h| String::from(h)).collect()
        }
    }

    /// Entry-point for clock-offset thread communicating via message queues
    pub fn run(&mut self) {
        loop {
            self.ntp_ping();
            thread::sleep(std::time::Duration::from_secs_f64(40.0));
            let offs = OffsetEvent {};
            self.tkr_channel.send(offs).unwrap();
            self.ui_channel.send(UImessage::Offset(offs)).unwrap();
        }
    }

    fn ntp_ping(&mut self) {
        // See https://datatracker.ietf.org/doc/html/rfc5905#section-7.3
        let host = &self.ntp_servers[weak_rand() as usize % self.ntp_servers.len()];
        println!("{}", host);

        if let Ok(raw) = ntp::request((host.clone(), 123u16)) {
            let t_return = chrono::Local::now();

            if let Some(pkt) = NtpPacket::from(raw, t_return) {

                println!("{:?}", pkt);
                println!("offs= {},  trans= {}", pkt.offset(), pkt.transit());
            }
        }
    }
}

// (C)Copyright 2023, RW Penney
