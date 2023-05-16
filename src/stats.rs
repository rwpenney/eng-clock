/*
 *  Statistical estimators for eng-clock
 *  RW Penney, April 2023
 */

use chrono;
use crate::Timestamp;


/// Exponentially smoothed moving average filter
#[derive(Clone, Copy)]
pub struct ExpoAvg {
    /// The weight given to each new sample (typically close to zero,
    /// such that the averaging timescale is roughly 1/eps)
    eps: f64,

    /// An unscaled rolling-average of the input samples
    numerator: f64,

    /// The normalizing function applied to the unscaled rolling average
    denominator: f64
}

impl ExpoAvg {
    /// Create new moving-average filter, with smoothing timescale 1.0/eps
    pub fn new(eps: f64) -> ExpoAvg {
        assert!(0.0 < eps && eps < 1.0);

        ExpoAvg {
            eps,
            numerator: 0.0,
            denominator: 0.0
        }
    }

    /// Extract current estimator of moving mean
    pub fn query(&self) -> Option<f64> {
        if self.denominator != 0.0 {
            Some(self.numerator / self.denominator)
        } else {
            None
        }
    }

    /// Incorporate a new real-valued sample value, returning new average
    pub fn add_sample(&mut self, x: f64) -> f64 {
        self.numerator += self.eps * (x - self.numerator);
        self.denominator += self.eps * (1.0 - self.denominator);

        self.numerator / self.denominator
    }

    /// Incorporate a new time-like sample, nominally in nanoseconds, returning new average
    pub fn add_duration(&mut self, dt: chrono::Duration) -> chrono::Duration {
        let dt = self.add_sample(dt.num_nanoseconds()
                                   .map(|n| n as f64)
                                   .unwrap_or(1e10));
        chrono::Duration::nanoseconds(dt.round() as i64)
    }
}


/// Recursive Bayesian estimator of clock-offset,
/// assuming Gaussian prior and measurement error
pub struct BayesOffset {
    /// The posterior mean clock-offset, in seconds
    mean: f32,

    /// The variance of the posterior distribution, in square-seconds
    variance: f32,

    /// The (uncorrected) time at which an observation was last provided
    last_obs_time: Option<Timestamp>,

    /// The diffusive growth rate of the offset uncertainty,
    /// in seconds per square-root day
    diffusivity: f32
}

impl BayesOffset {
    /// The minimum credible uncertainty in a clock-offset measurement (in seconds)
    const MIN_PRECISION: f32 = 1e-6;

    /// Create a new offset-estimator with zero bias and given standard-deviation
    pub fn new(dt0: f32) -> BayesOffset {
        BayesOffset {
            mean: 0.0,
            variance: BayesOffset::clamp_variance(dt0),
            last_obs_time: None,
            diffusivity: 0.5
        }
    }

    /// Supply a new measurement of the clock offset
    pub fn add_observation(&mut self, offset: f32, precision: f32,
                           obs_time: Timestamp) {
        let var_obs = BayesOffset::clamp_variance(precision);
        let inst_var = self.diffused_variance(obs_time);
        let var_rat = inst_var / var_obs;

        self.mean = self.mean / (1.0 + var_rat) +
                    offset / (1.0 + 1.0 / var_rat);
        self.variance = inst_var / (1.0 + var_rat);
        self.last_obs_time = Some(obs_time);
    }

    /// Maximum-likelihood estimator of the clock offset
    pub fn avg_offset(&self) -> chrono::Duration {
        chrono::Duration::microseconds((self.mean * 1e6) as i64)
    }

    /// Extrapolate the offset variance allowing for diffusive growth
    /// since the previous observation
    fn diffused_variance(&self, obs_time: Timestamp) -> f32 {
        if let Some(t0) = self.last_obs_time {
            let dt_days = (obs_time - t0).num_milliseconds() as f32
                                / crate::MILLIS_PER_DAY;
            self.variance + self.diffusivity.powi(2) * dt_days
        } else {
            self.variance
        }
    }

    /// Current estimate of the margin of error in the clock offset,
    /// allowing for growth since the time of the latest measurement
    pub fn stddev_offset(&self, obs_time: Timestamp) -> f32 {
        self.diffused_variance(obs_time).sqrt()
    }

    fn clamp_variance(dt: f32) -> f32 {
        if dt > BayesOffset::MIN_PRECISION {
            dt * dt
        } else {
            BayesOffset::MIN_PRECISION * BayesOffset::MIN_PRECISION
        }
    }
}


#[cfg(test)]
mod tests {
    use chrono::Duration;
    use super::{ BayesOffset, ExpoAvg };
    use crate::utc_now;
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

    #[test]
    fn bo_init() {
        let bo0 = BayesOffset::new(1.5);

        assert_eq!(bo0.mean, 0.0);
        assert_eq!(bo0.variance, 2.25);
        assert_eq!(bo0.last_obs_time, None);
        assert!(bo0.diffusivity > 1e-2 && bo0.diffusivity < 10.0);

        assert_eq!(BayesOffset::new(0.0).variance,
                   BayesOffset::MIN_PRECISION.powi(2));
    }

    #[test]
    fn bo_mean_units() {
        let mut bo = BayesOffset::new(0.0);

        bo.mean = 1.0;
        assert_eq!(bo.avg_offset(), chrono::Duration::seconds(1));

        bo.mean = 0.125;
        assert_eq!(bo.avg_offset(), chrono::Duration::milliseconds(125));
    }

    #[test]
    fn bo_variances() {
        let mut bo = BayesOffset::new(2.5);
        bo.diffusivity = 3.0;

        assert_eq!(bo.stddev_offset(utc_now()), 2.5);
        assert_eq!(bo.diffused_variance(utc_now()), 6.25);

        bo.last_obs_time = Some(mk_time(0, (0, 0, 0)));
        let t1 = mk_time(86400, (0, 0, 0));

        assert_close(bo.diffused_variance(t1) as f64, 6.25 + 9.0, 1e-9);
        assert_close(bo.stddev_offset(t1) as f64, (15.25f64).sqrt(), 1e-7);
    }

    #[test]
    fn bo_simple_update() {
        const PRECISION: f32 = 1.7e-2;
        let mut bo = BayesOffset::new(PRECISION);
        let t = utc_now();

        bo.mean = 2.0;
        bo.add_observation(3.0, PRECISION, t);

        assert_close(bo.mean as f64, 2.5, 1e-9);
        assert_close(bo.variance as f64,
                     0.5 * PRECISION.powi(2) as f64, 1e-8);
        assert_eq!(bo.last_obs_time, Some(t));
    }

    #[test]
    fn bo_update() {
        let p0: f32 = 1.7e-2;
        let t0 = mk_time(0, (0, 0, 0));
        let t1 = t0 + chrono::Duration::days(4);

        let mut bo = BayesOffset::new(p0);
        bo.mean = 1.9;
        bo.diffusivity = 0.5 * p0;
        bo.last_obs_time = Some(t0);

        bo.add_observation(2.7, p0 * 3.0, t1);

        assert_close(bo.mean as f64,
                     1.9 / (1.0 + 2.0 / 9.0) + 2.7 / (1.0 + 9.0 / 2.0), 1e-6);
        assert_close(bo.variance as f64,
                     2.0 * p0.powi(2) as f64 / (1.0 + 2.0 / 9.0), 1e-8);
        assert_eq!(bo.last_obs_time, Some(t1));
    }
}

// (C)Copyright 2023, RW Penney
