/*
 *  Clock-ticking mechanisms for engineering clock display
 *  RW Penney, April 2023
 */

use std::thread;
use crate::{ TickEvent, Timestamp, UImessage, UIsender, utc_now };


pub struct Ticker {
    channel: UIsender
}

impl Ticker {
    /// The time-interval between screen updates, in microseconds
    const PERIOD_US: i64 = 250_000;

    pub fn new(channel: &UIsender) -> Ticker {
        Ticker {
            channel: channel.clone()
        }
    }

    /// Entry-point for tick-generating thread communicating via GLIB messages
    pub fn run(&self) {
        loop {
            let (t_nominal, tick_id) = self.wait_next();
            let t_transmit = utc_now();

            self.channel.send(
                UImessage::Tick(TickEvent { t_nominal, t_transmit, tick_id })
            ).unwrap();

            // FIXME - read incoming messages here
        }
    }

    /// Compute nominal time of next clock update, and sleep until it ready for GUI update
    #[inline]
    fn wait_next(&self) -> (Timestamp, i64) {
        let now = utc_now();

        let now_us = now.timestamp_micros();
        let tick_id = (now_us + Ticker::PERIOD_US + Ticker::PERIOD_US / 4)
                            / Ticker::PERIOD_US;
        let step_us = (tick_id * Ticker::PERIOD_US) - now_us;
        let t_next_nominal = now + chrono::Duration::microseconds(step_us);
        // FIXME - apply clock-offset and UI-latency corrections

        thread::sleep(std::time::Duration::from_micros(step_us as u64));

        ( t_next_nominal, tick_id )
    }
}

// (C)Copyright 2023, RW Penney
