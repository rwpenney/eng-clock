/*
 *  Statistical estimators for eng-clock
 *  RW Penney, April 2023
 */

use chrono;


/// Exponentially smoothed moving average filter
#[derive(Clone, Copy)]
pub struct ExpoAvg {
    eps: f64,
    numerator: f64,
    denominator: f64
}

impl ExpoAvg {
    pub fn new(eps: f64) -> ExpoAvg {
        assert!(0.0 < eps && eps < 1.0);

        ExpoAvg {
            eps,
            numerator: 0.0,
            denominator: 0.0
        }
    }

    // Extract current estimator of moving mean
    pub fn query(&self) -> Option<f64> {
        if self.denominator != 0.0 {
            Some(self.numerator / self.denominator)
        } else {
            None
        }
    }

    // Incorporate a new real-valued sample value, returning new average
    pub fn add_sample(&mut self, x: f64) -> f64 {
        self.numerator += self.eps * (x - self.numerator);
        self.denominator += self.eps * (1.0 - self.denominator);

        self.numerator / self.denominator
    }

    // Incorporate a new time-like sample, nominally in nanoseconds, returning new average
    pub fn add_duration(&mut self, dt: chrono::Duration) -> chrono::Duration {
        let dt = self.add_sample(dt.num_nanoseconds()
                                   .map(|n| n as f64)
                                   .unwrap_or(1e10));
        chrono::Duration::nanoseconds(dt.round() as i64)
    }
}


#[cfg(test)]
mod tests {
    use chrono::Duration;
    use super::ExpoAvg;
    use crate::testing::*;

    #[test]
    fn expavg_const() {
        const ITERATIONS: i32 = 13;

        for eps in [ 0.01, 0.02, 0.05, 0.1, 0.2 ] {
            for v in -10..=10 {
                let mut filter = ExpoAvg::new(eps);

                for i in 0..ITERATIONS {
                    let avg = filter.add_sample(v as f64);
                    assert_close(avg, v as f64, 1e-9);

                    assert_eq!(filter.query().unwrap_or(-811.823), avg);

                    let norm = 1.0 - (1.0 - eps).powi(i+1);
                    assert_close(filter.denominator, norm, 1e-12);
                }
            }
        }
    }

    #[test]
    fn expavg_ramp() {
        for eps in [ 0.01, 0.02, 0.05, 0.1, 0.2 ] {
            const MOD: i32 = 7;
            const N: i32 = MOD * 10;

            let mut filter = ExpoAvg::new(eps);

            for i in 0..N {
                filter.add_sample((i % MOD) as f64);
            }

            let expected = ((MOD as f64) - (1.0 - (1.0 - eps).powi(MOD)) / eps)
                                / (1.0 - (1.0 - eps).powi(MOD));

            assert_close(filter.query().unwrap(), expected, 1e-12);
        }
    }

    #[test]
    fn expavg_durations() {
        let mut filter = ExpoAvg::new(0.1);

        for _ in 0..20 {
            filter.add_duration(Duration::microseconds(67));
        }

        assert_close(filter.query().unwrap(), 67e3, 1e-12);
    }
}

// (C)Copyright 2023, RW Penney
