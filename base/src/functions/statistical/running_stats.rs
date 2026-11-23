/// Running count, mean and sum of squared deviations (Welford's algorithm).
///
/// `sumsq - sum * sum / n` subtracts two large and almost equal numbers when
/// the values are far from zero, and loses most of the digits.
#[derive(Default)]
pub(crate) struct RunningStats {
    count: u64,
    mean: f64,
    sum_sq_dev: f64,
}

impl RunningStats {
    #[inline]
    pub(crate) fn add(&mut self, value: f64) {
        self.count += 1;
        let delta = value - self.mean;
        self.mean += delta / self.count as f64;
        self.sum_sq_dev += delta * (value - self.mean);
    }

    pub(crate) fn count(&self) -> u64 {
        self.count
    }

    /// Sum of the squared deviations from the mean
    pub(crate) fn sum_sq_dev(&self) -> f64 {
        self.sum_sq_dev
    }

    /// Population variance. Needs at least one value.
    pub(crate) fn var_p(&self) -> f64 {
        self.sum_sq_dev / self.count as f64
    }

    /// Sample variance. Needs at least two values.
    pub(crate) fn var_s(&self) -> f64 {
        self.sum_sq_dev / (self.count as f64 - 1.0)
    }
}
