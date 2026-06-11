//! nice-number axis scale: classic 1-2-5 tick algorithm (heckbert's "nice
//! numbers for graph labels", same granularity family plotters uses in
//! `key_points`, ref/vis/plotters). pure math, no text, no gpu.

/// A resolved value axis: nice bounds enclosing the data range plus the
/// tick values, ascending. `min`/`max` are what the plot scales against.
#[derive(Debug, Clone, PartialEq)]
pub struct Axis {
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub ticks: Vec<f32>,
}

impl Axis {
    /// 0..1 position of `value` inside the axis range.
    pub fn normalize(&self, value: f32) -> f32 {
        if self.max <= self.min {
            return 0.0;
        }
        (value - self.min) / (self.max - self.min)
    }
}

/// Build a nice tick scale for `[lo, hi]` aiming at `target` ticks.
/// Tolerates inverted, degenerate and non-finite input (error paths return
/// a usable 0..1 fallback instead of NaN geometry).
pub fn nice_ticks(lo: f32, hi: f32, target: usize) -> Axis {
    let (mut lo, mut hi) = if lo <= hi {
        (lo as f64, hi as f64)
    } else {
        (hi as f64, lo as f64)
    };
    if !lo.is_finite() || !hi.is_finite() {
        lo = 0.0;
        hi = 1.0;
    }
    if (hi - lo).abs() < f64::EPSILON {
        // degenerate range: widen around the value so a scale exists.
        let pad = if lo == 0.0 { 1.0 } else { lo.abs() * 0.5 };
        lo -= pad;
        hi += pad;
    }
    let target = target.max(2);
    let span = nice_num(hi - lo, false);
    let step = nice_num(span / (target - 1) as f64, true);
    let nice_lo = (lo / step).floor() * step;
    let nice_hi = (hi / step).ceil() * step;
    let count = ((nice_hi - nice_lo) / step).round() as usize;
    let ticks = (0..=count)
        .map(|i| (nice_lo + step * i as f64) as f32)
        .collect();
    Axis {
        min: nice_lo as f32,
        max: nice_hi as f32,
        step: step as f32,
        ticks,
    }
}

/// Round `x` to a "nice" value: 1, 2 or 5 times a power of ten.
/// `round = true` picks the nearest nice value, `false` the ceiling.
fn nice_num(x: f64, round: bool) -> f64 {
    let exp = x.log10().floor();
    let pow = 10f64.powf(exp);
    let f = x / pow;
    let nf = if round {
        match f {
            f if f < 1.5 => 1.0,
            f if f < 3.0 => 2.0,
            f if f < 7.0 => 5.0,
            _ => 10.0,
        }
    } else {
        match f {
            f if f <= 1.0 => 1.0,
            f if f <= 2.0 => 2.0,
            f if f <= 5.0 => 5.0,
            _ => 10.0,
        }
    };
    nf * pow
}

/// Format a tick value with exactly the decimals its step needs (step 20
/// prints "40", step 0.05 prints "0.85"). kills float noise like 0.30000001.
pub fn format_tick(value: f32, step: f32) -> String {
    let decimals = if step >= 1.0 {
        0
    } else {
        (-(step as f64).log10().floor()) as usize
    };
    format!("{value:.decimals$}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_covers(axis: &Axis, lo: f32, hi: f32) {
        assert!(axis.min <= lo, "{axis:?} must reach down to {lo}");
        assert!(axis.max >= hi, "{axis:?} must reach up to {hi}");
        let first = *axis.ticks.first().unwrap();
        let last = *axis.ticks.last().unwrap();
        assert_eq!(first, axis.min);
        assert_eq!(last, axis.max);
        for pair in axis.ticks.windows(2) {
            assert!(pair[0] < pair[1], "ticks must ascend: {:?}", axis.ticks);
            assert!(
                (pair[1] - pair[0] - axis.step).abs() < axis.step * 1e-3,
                "uniform step"
            );
        }
    }

    #[test]
    fn varied_ranges_produce_nice_bounded_ticks() {
        // (lo, hi): normalized chart data, percent-ish, offset ints,
        // negatives across zero, tiny fractions, large values.
        let cases: [(f32, f32); 6] = [
            (0.0, 0.95),
            (0.0, 100.0),
            (3.0, 97.0),
            (-50.0, 40.0),
            (0.001, 0.009),
            (12.0, 12800.0),
        ];
        for (lo, hi) in cases {
            let axis = nice_ticks(lo, hi, 5);
            assert_covers(&axis, lo, hi);
            assert!(
                (3..=9).contains(&axis.ticks.len()),
                "tick count near target for ({lo}, {hi}): {:?}",
                axis.ticks
            );
            // step is 1, 2 or 5 times a power of ten.
            let exp = (axis.step as f64).log10().floor();
            let mantissa = axis.step as f64 / 10f64.powf(exp);
            let nice = [1.0, 2.0, 5.0].iter().any(|n| (mantissa - n).abs() < 1e-6);
            assert!(nice, "step {} must be 1/2/5 family", axis.step);
        }
    }

    #[test]
    fn degenerate_inverted_and_non_finite_ranges_still_yield_a_scale() {
        for axis in [
            nice_ticks(5.0, 5.0, 5),
            nice_ticks(0.0, 0.0, 5),
            nice_ticks(9.0, 1.0, 5),
            nice_ticks(f32::NAN, f32::INFINITY, 5),
        ] {
            assert!(axis.max > axis.min, "usable range: {axis:?}");
            assert!(axis.ticks.len() >= 2);
            assert!(axis.ticks.iter().all(|t| t.is_finite()));
        }
        // inverted input covers the swapped range.
        assert_covers(&nice_ticks(9.0, 1.0, 5), 1.0, 9.0);
    }

    #[test]
    fn normalize_maps_range_to_unit_interval() {
        let axis = nice_ticks(0.0, 100.0, 5);
        assert_eq!(axis.normalize(axis.min), 0.0);
        assert_eq!(axis.normalize(axis.max), 1.0);
        let mid = axis.normalize((axis.min + axis.max) / 2.0);
        assert!((mid - 0.5).abs() < 1e-6);
    }

    #[test]
    fn format_tick_matches_step_precision() {
        assert_eq!(format_tick(40.0, 20.0), "40");
        assert_eq!(format_tick(0.3000001, 0.1), "0.3");
        assert_eq!(format_tick(0.85, 0.05), "0.85");
        assert_eq!(format_tick(1200.0, 200.0), "1200");
    }
}
