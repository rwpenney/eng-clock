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

pub mod stats;
mod ticker;

use gtk::glib;

pub type Timestamp = chrono::DateTime<chrono::Utc>;
pub type UIsender = glib::Sender<UImessage>;
pub type Ticker = ticker::Ticker;


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


/// Messages that can be sent asynchronously to GTK main loop from other threads
pub enum UImessage {
    Tick(TickEvent)
    // Add clock-offset stats
}


/// Get current time in UTC timezone
#[inline]
pub fn utc_now() -> Timestamp {
    chrono::Utc::now()
}

#[cfg(test)]
mod testing;

// (C)Copyright 2023, RW Penney
