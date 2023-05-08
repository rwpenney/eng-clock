/*
 *  Clock offset estimation tools for eng-clock
 *  RW Penney, May 2023
 */

use sntpc::{ NtpContext, NtpTimestampGenerator, NtpUdpSocket };
use std::{
    net::{ SocketAddr, ToSocketAddrs, UdpSocket },
    rc::Rc,
    sync::mpsc,
    thread };
use crate::{ OffsetEvent, Timestamp, UImessage, UIsender, utc_now, weak_rand };


const NTP_SERVERS: [&str; 8] = [
    "0.uk.pool.ntp.org",
    "2.uk.pool.ntp.org",
    "1.europe.pool.ntp.org",
    "3.europe.pool.ntp.org",
    "ntp2d.mcc.ac.uk",
    "1.asia.pool.ntp.org",
    "2.north-america.pool.ntp.org",
    "3.debian.pool.ntp.org"
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


#[derive(Debug)]
struct NtpServer(String);

impl ToSocketAddrs for NtpServer {
    type Iter = std::vec::IntoIter<SocketAddr>;

    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        ToSocketAddrs::to_socket_addrs(&(&*self.0, 123u16))
    }
}

pub struct OffsetEstimator {
    tkr_channel: mpsc::Sender<OffsetEvent>,
    ui_channel: UIsender,
    ntp_servers: Vec<NtpServer>
}

impl OffsetEstimator {
    pub fn new(tkr_channel: mpsc::Sender<OffsetEvent>,
               ui_channel: UIsender) -> OffsetEstimator {
        OffsetEstimator {
            tkr_channel,
            ui_channel,
            ntp_servers: NTP_SERVERS.into_iter().map(|h|
                                NtpServer(String::from(h))).collect()
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
            self.ntp_ping(wrapped_skt.clone(), ntp_ctxt.clone());
            thread::sleep(std::time::Duration::from_secs_f64(17.0));
            let offs = OffsetEvent {};
            self.tkr_channel.send(offs).unwrap();
            self.ui_channel.send(UImessage::Offset(offs)).unwrap();
        }
    }

    fn ntp_ping<T>(&mut self, skt: UdpSocketWrapper, ctxt: NtpContext<T>)
            where T: NtpTimestampGenerator + Copy {
        // See https://datatracker.ietf.org/doc/html/rfc5905#section-7.3
        let host = &self.ntp_servers[weak_rand() as usize % self.ntp_servers.len()];
        println!("{:?}", host);

        let ping = sntpc::get_time(host, skt, ctxt);
        println!("{:?}", ping);

        // FIXME - accumulate robust statistics on clock offset & share with UI & Ticker
    }
}

// (C)Copyright 2023, RW Penney
