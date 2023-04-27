/*
 *  Clock-ticking mechanisms for engineering clock display
 *  RW Penney, April 2023
 */

use gtk::glib;
use std::thread;

pub type Timestamp = chrono::DateTime<chrono::Utc>;
type UIsender = glib::Sender<UImessage>;


pub enum UImessage {
    Tick(Timestamp)
    // Add clock-offset stats
}


pub struct Ticker {
    channel: UIsender
}

impl Ticker {
    pub fn new(channel: &UIsender) -> Ticker {
        Ticker {
            channel: channel.clone()
        }
    }

    pub fn run(&self) {
        const MILLION: u64 = 1_000_000;
        loop {
            let now = chrono::Utc::now();

            let now_us = now.timestamp_micros() as u64;
            let step_us = (now_us / MILLION + 1) * MILLION - now_us;
            // FIXME - apply clock-offset correction

            self.channel.send(UImessage::Tick(now)).unwrap();
            println!("Ticking: {} + {}", now, step_us);

            thread::sleep(std::time::Duration::from_micros(step_us as u64));
        }
    }
}

// (C)Copyright 2023, RW Penney
