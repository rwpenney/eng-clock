/*
 *  Supporting functions for engineering clock display
 *  RW Penney, April 2023
 */

/*  eng-clock - a dynamically synchronized realtime clock display
    Copyright (C) 2023, RW Penney

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>
 */

pub mod config;
pub mod sync;
pub mod stats;
pub mod ticker;

#[cfg(test)]
mod testing;

use gtk::glib;

pub type Timestamp = chrono::DateTime<chrono::Utc>;
pub type UIsender = glib::Sender<UImessage>;
pub type Ticker = ticker::Ticker;

pub const MILLIS_PER_DAY: f32 = 86400e3;


/// Clock-ticking event
#[derive(Clone, Copy)]
pub struct TickEvent {
    /// The (corrected) time that should be displayed to the user
    pub t_nominal: Timestamp,

    /// The (uncorrected) time at which this message was sent
    pub t_transmit: Timestamp,

    /// The number of ticks since an arbitrary origin, typically the POSIX epoch
    pub tick_id: i64
}


/// Clock-offset update event
#[derive(Clone, Copy, Debug)]
pub struct OffsetEvent {
    /// The latest best-fit correction to be added to the local clock
    pub avg_offset: chrono::Duration,

    /// The nominal error on the clock-offset, in seconds
    pub stddev_offset: f32,
}


/// Messages that can be sent asynchronously to GTK main loop from other threads
pub enum UImessage {
    Tick(TickEvent),
    Offset(OffsetEvent)
}


/// Get current time in UTC timezone
#[inline]
pub fn utc_now() -> Timestamp {
    chrono::Utc::now()
}


/// Crude method for generating pseudo-random numbers
fn weak_rand() -> u32 {
    use std::time::SystemTime;

    static mut COUNTER: u128 = 0x4564a54753fa4c49;

    let dt = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
                              .expect("Failed to compute unix timestamp");

    unsafe {
        COUNTER = (COUNTER * 0x5deece66d + 11) & ((1 << 48) - 1);
        ((dt.as_nanos() * 0x56cae88f ^ COUNTER) % 4294967291) as u32
    }
}


#[cfg(test)]
mod tests {
    use super::weak_rand;
    use crate::testing::*;

    #[test]
    fn rand_dist() {
        const N: i32 = 1000;

        for modulus in [997, 10891, 1201201] {
            let samples: Vec<f64> =
                (0 .. N).map(|_| (weak_rand() % modulus) as f64
                                    / (modulus as f64)).collect();
            println!("{:?}", samples);

            let mean = samples.iter().sum::<f64>() / (N as f64);
            let vrnc = samples.iter().map(|x| x * x).sum::<f64>() / (N as f64)
                            - mean * mean;

            assert_close(mean, 0.5, 1.0 / (N as f64).sqrt());
            assert_close(vrnc, 1.0 / 12.0, 0.3 / (N as f64).sqrt());
        }
    }
}

// (C)Copyright 2023, RW Penney
