// Unit-test helpers for eng-clock
// RW Penney, April 2023

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

// (C)Copyright 2023, RW Penney
