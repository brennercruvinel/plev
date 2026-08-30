//! Fixed-capacity rolling sample window with mean and percentile queries.
//!
//! Pure data structure: no clocks, no GPU. `PerfMonitor` feeds it one
//! sample per frame; queries walk at most `capacity` samples (120 by
//! default), so per-frame cost stays trivial.

pub struct RollingWindow {
    capacity: usize,
    samples: Vec<f64>,
    /// Index of the oldest sample once the window is full.
    head: usize,
}

impl RollingWindow {
    /// `capacity` is clamped to at least 1.
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        Self {
            capacity,
            samples: Vec::with_capacity(capacity),
            head: 0,
        }
    }

    pub fn push(&mut self, value: f64) {
        if self.samples.len() < self.capacity {
            self.samples.push(value);
        } else {
            self.samples[self.head] = value;
            self.head = (self.head + 1) % self.capacity;
        }
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn sum(&self) -> f64 {
        self.samples.iter().sum()
    }

    /// Arithmetic mean of the current window; 0.0 when empty.
    pub fn mean(&self) -> f64 {
        if self.samples.is_empty() {
            0.0
        } else {
            self.sum() / self.samples.len() as f64
        }
    }

    /// Nearest-rank percentile (`p` in 0..=100) over the current window;
    /// 0.0 when empty. Sorts a copy: bounded by `capacity`, cheap for the
    /// window sizes used here.
    pub fn percentile(&self, p: f64) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let mut sorted = self.samples.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let n = sorted.len();
        let rank = ((p / 100.0) * n as f64).ceil() as usize;
        sorted[rank.clamp(1, n) - 1]
    }
}
