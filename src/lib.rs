/*
 *  Clock-ticking mechanisms for engineering clock display
 *  RW Penney, April 2023
 */

use gtk::glib;
use std::thread;

pub type Timestamp = chrono::DateTime<chrono::Utc>;
type UIsender = glib::Sender<UImessage>;


#[derive(Clone, Copy)]
pub struct TickEvent {
    pub t_nominal: Timestamp,
    pub t_transmit: Timestamp,
    pub tick_id: i64
}


pub enum UImessage {
    Tick(TickEvent)
    // Add clock-offset stats
}


#[inline]
pub fn utc_now() -> Timestamp {
    chrono::Utc::now()
}


pub struct Ticker {
    channel: UIsender
}

impl Ticker {
    const PERIOD_US: i64 = 250_000;

    pub fn new(channel: &UIsender) -> Ticker {
        Ticker {
            channel: channel.clone()
        }
    }

    pub fn run(&self) {
        loop {
            let (t_nominal, tick_id) = self.wait_next();
            let t_transmit = utc_now();

            self.channel.send(
                UImessage::Tick(TickEvent { t_nominal, t_transmit, tick_id })
            ).unwrap();

            println!("Ticking: {} / {}", t_nominal, t_transmit);

            // FIXME - read incoming messages here
        }
    }

    #[inline]
    fn wait_next(&self) -> (Timestamp, i64) {
        let now = utc_now();

        let now_us = now.timestamp_micros();
        let tick_id = (now_us + Ticker::PERIOD_US) / Ticker::PERIOD_US;
        let step_us = (tick_id * Ticker::PERIOD_US) - now_us;
        // FIXME - apply clock-offset correction

        thread::sleep(std::time::Duration::from_micros(step_us as u64));

        ( now + chrono::Duration::microseconds(step_us), tick_id )
    }
}

// (C)Copyright 2023, RW Penney
