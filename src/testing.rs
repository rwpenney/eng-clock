// Unit-test helpers for eng-clock
// RW Penney, April 2023

use chrono::{ Duration, TimeZone, Utc };
use super::Timestamp;


/// Check floating numbers agree to within given absolute tolerance
pub fn assert_close(x: f64, y: f64, delta: f64) {
    if !x.is_finite() || !y.is_finite() {
        assert!(x == y, "{:?} !~ {:?}", x, y);
    } else {
        let scale = (x - y).abs() / delta;
        assert!((x - y).abs() <= delta,
                "{:?} !~ {:?} (scale={:?})", x, y, scale);
    }
}


/// Shorthand for generating timestamps relative to an arbitrary date
pub fn mk_time(seconds: i32, fracs: (i32, i32, i32)) -> Timestamp {
    Utc.with_ymd_and_hms(1991, 7, 10, 0, 0, 0).unwrap()
        + Duration::seconds(seconds as i64)
        + Duration::nanoseconds((fracs.2 + 1000 * (fracs.1 + 1000 * fracs.0)) as i64)
}

// (C)Copyright 2023, RW Penney
