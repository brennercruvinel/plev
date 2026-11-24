//! How the runs of one thing spread: counted in equal steps from the
//! shortest to the longest, with the shortest, the median, the 95th
//! percentile and the longest. The dev mode draws one for a zone, from
//! every run of it in the frames a `Recorder` keeps.

/// Runs counted by how long they took.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Histogram {
    /// Runs counted.
    pub count: u64,
    pub total_ns: u64,
    pub min_ns: u64,
    pub max_ns: u64,
    pub median_ns: u64,
    pub p95_ns: u64,
    /// Runs in each equal step from `min_ns` to `max_ns`; the last step
    /// holds the longest run. Empty of no run.
    pub bins: Vec<u32>,
}

impl Histogram {
    /// `samples`, in nanoseconds, counted in `bins` steps. Runs that all
    /// took the same land in the first step.
    pub fn of(mut samples: Vec<u64>, bins: usize) -> Self {
        if samples.is_empty() || bins == 0 {
            return Self::default();
        }
        samples.sort_unstable();
        let last = samples.len() - 1;
        let (min_ns, max_ns) = (samples[0], samples[last]);
        // one more than the width, so the longest run stays in the last step
        let width = u128::from(max_ns - min_ns) + 1;
        let mut counts = vec![0u32; bins];
        for &sample in &samples {
            let step = u128::from(sample - min_ns) * bins as u128 / width;
            counts[(step as usize).min(bins - 1)] += 1;
        }
        Self {
            count: samples.len() as u64,
            total_ns: samples.iter().sum(),
            min_ns,
            max_ns,
            median_ns: samples[rank(last, 50)],
            p95_ns: samples[rank(last, 95)],
            bins: counts,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// The mean run; zero of no run.
    pub fn mean_ns(&self) -> u64 {
        self.total_ns.checked_div(self.count).unwrap_or(0)
    }

    /// The step with the most runs, for a scale.
    pub fn tallest(&self) -> u32 {
        self.bins.iter().copied().max().unwrap_or(0)
    }
}

/// The place of the `percent`th percentile among `last + 1` sorted
/// samples, to the nearest one.
fn rank(last: usize, percent: usize) -> usize {
    (last * percent + 50) / 100
}
