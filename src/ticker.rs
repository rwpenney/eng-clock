/*
 *  Clock-ticking mechanisms for engineering clock display
 *  RW Penney, April 2023
 */

use std::thread;
use chrono::{ NaiveDateTime, Utc };
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
        let (t_next_nominal, tick_id, wait) = Ticker::predict_next(utc_now());

        thread::sleep(wait);

        ( t_next_nominal, tick_id )
    }

    #[inline]
    fn predict_next(now: Timestamp) -> (Timestamp, i64, std::time::Duration) {
        let now_us = now.timestamp_micros();
        let tick_id = (now_us + Ticker::PERIOD_US + Ticker::PERIOD_US / 4)
                            / Ticker::PERIOD_US;
        let step_us = (tick_id * Ticker::PERIOD_US) - now_us;
        let t_next_nominal = Timestamp::from_utc(
            NaiveDateTime::from_timestamp_micros(tick_id * Ticker::PERIOD_US)
                .unwrap(), Utc);
        // FIXME - apply clock-offset and UI-latency corrections

        ( t_next_nominal,
          tick_id,
          std::time::Duration::from_micros(step_us as u64) )
    }
}


#[cfg(test)]
mod tests {
    use super::{ Ticker, Timestamp };
    use crate::testing::*;

    #[test]
    fn prediction() {
        fn next(s: i32, f: (i32, i32, i32)) -> (Timestamp, i64, u32) {
            let (t_nom, tick, wait) = Ticker::predict_next(mk_time(s, f));
            ( t_nom, tick % 40, wait.as_micros() as u32 )
        }

        assert_eq!(next(0, (0, 0, 0)),
                   ( mk_time(0, (250, 0, 0)), 1, 250_000 ));
        assert_eq!(next(281, (149, 151, 157)),
                   ( mk_time(281, (250, 0, 0)), 5, 100_849 ));
        assert_eq!(next(977, (739, 743, 751)),
                   ( mk_time(978, (0, 0, 0)), 32, 260_257 ));
    }
}

// (C)Copyright 2023, RW Penney
