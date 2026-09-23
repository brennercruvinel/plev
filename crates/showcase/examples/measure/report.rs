//! The samples as markdown: a summary row per scenario, then one table
//! per scenario with every stage's calls, items and time at the 50th and
//! 95th percentile (nearest rank) over the measured turns.

use std::fmt::Write as _;

use engine::perf::RollingWindow;
use engine::perf::stages::{Stage, StageCost};

use crate::harness::{DT, Sample};
use crate::scenarios::{Config, Run, WARM_UP};

/// Nearest-rank percentile `p` of `values`; 0 when there are none.
fn pct(values: impl Iterator<Item = f64>, p: f64) -> f64 {
    let values: Vec<f64> = values.collect();
    let mut w = RollingWindow::new(values.len());
    for v in values {
        w.push(v);
    }
    w.percentile(p)
}

fn stat(samples: &[&Sample], stage: Stage, p: f64, of: impl Fn(StageCost) -> f64) -> f64 {
    pct(samples.iter().map(|s| of(s.stages.get(stage))), p)
}

fn micros(c: StageCost) -> f64 {
    c.micros()
}

fn calls(c: StageCost) -> f64 {
    c.calls as f64
}

fn items(c: StageCost) -> f64 {
    c.items as f64
}

fn us(v: f64) -> String {
    if v == 0.0 {
        "0".to_string()
    } else if v < 10.0 {
        format!("{v:.2}")
    } else {
        format!("{v:.1}")
    }
}

/// A signed time: the precision of [`us`] by magnitude.
fn signed_us(v: f64) -> String {
    if v < 0.0 {
        format!("-{}", us(-v))
    } else {
        us(v)
    }
}

/// A turn read a frame's GPU timestamps.
fn read_gpu(s: &Sample) -> bool {
    s.stages.get(Stage::GpuSpan).calls > 0
}

pub fn markdown(cfg: &Config, runs: &[Run], build: &str) -> String {
    let v = cfg.viewport;
    let read_any = runs.iter().flat_map(|r| &r.samples).any(read_gpu);
    let timestamps = match (cfg.gpu_timestamps, read_any) {
        (false, _) => "gpu timestamps off",
        (true, true) => "gpu timestamps on",
        (true, false) => "gpu timestamps asked for and unavailable (none read)",
    };
    let mut md = String::new();
    let _ = writeln!(
        md,
        "{} measured turns per scenario after {} warm-up turns, one turn every {:.2} ms \
         (a drawn turn's tick gets the time since the last drawn turn), \
         viewport {}x{} logical at scale {} ({}x{} physical), theme {}, {build} build, \
         stages {}, {timestamps}. time is microseconds; p50 and p95 are nearest-rank over \
         the turns.\n",
        cfg.frames,
        WARM_UP,
        DT * 1000.0,
        v.width,
        v.height,
        v.scale,
        v.physical().0,
        v.physical().1,
        cfg.theme,
        if cfg.gpu_sync {
            "on with the gpu wait"
        } else {
            "on, no gpu wait"
        }
    );
    summary(&mut md, runs);
    if cfg.gpu_timestamps {
        gpu_summary(&mut md, runs, read_any);
    }
    for run in runs {
        scenario(&mut md, run);
    }
    md
}

/// What the GPU spent per scenario, over the drawn turns: the wait next
/// to the timestamps, and the latency the wait holds beyond them.
fn gpu_summary(md: &mut String, runs: &[Run], read_any: bool) {
    let _ = writeln!(
        md,
        "gpu time per scenario, p50 over the drawn turns. the timestamps of a frame are \
         read one drawn frame later (engine::perf::gpu_time), so a turn's gpu stages are \
         the frame drawn before it. the span runs from the first pass's start to the \
         composite's end; the layer passes from the first one's start to the last one's \
         end; the composite from the end of the layer passes (or its own start) to its \
         end. submit latency is the wait's p50 minus the span's p50.\n"
    );
    let _ = writeln!(
        md,
        "| scenario | turns read | gpu wait p50 | gpu span p50 | submit latency | layer passes \
         p50 | gpu layer passes p50 | gpu composite p50 | gpu span p95 |"
    );
    let _ = writeln!(md, "|{}", " --- |".repeat(9));
    for run in runs {
        let drawn: Vec<&Sample> = run.samples.iter().filter(|s| s.drawn).collect();
        let read: Vec<&Sample> = drawn.iter().copied().filter(|s| read_gpu(s)).collect();
        if !read_any || read.is_empty() {
            let _ = writeln!(
                md,
                "| {} | 0 of {} |{}",
                run.key,
                drawn.len(),
                " unavailable |".repeat(7)
            );
            continue;
        }
        let wait = stat(&drawn, Stage::GpuWait, 50.0, micros);
        let span = stat(&read, Stage::GpuSpan, 50.0, micros);
        let _ = writeln!(
            md,
            "| {} | {} of {} | {} | {} | {} | {} | {} | {} | {} |",
            run.key,
            read.len(),
            drawn.len(),
            us(wait),
            us(span),
            signed_us(wait - span),
            stat(&read, Stage::GpuLayers, 50.0, items),
            us(stat(&read, Stage::GpuLayers, 50.0, micros)),
            us(stat(&read, Stage::GpuComposite, 50.0, micros)),
            us(stat(&read, Stage::GpuSpan, 95.0, micros)),
        );
    }
    md.push('\n');
}

fn summary(md: &mut String, runs: &[Run]) {
    let _ = writeln!(
        md,
        "| scenario | frames drawn | frame p50 | frame p95 | input p50 | scene build p50 | \
         draw p50 | gpu wait p50 | layers rebuilt p50 | nodes rebuilt p50 | upload bytes p50 |"
    );
    let _ = writeln!(md, "|{}", " --- |".repeat(11));
    for run in runs {
        let all: Vec<&Sample> = run.samples.iter().collect();
        let drawn = all.iter().filter(|s| s.drawn).count();
        let t = |stage| us(stat(&all, stage, 50.0, micros));
        let _ = writeln!(
            md,
            "| {} | {drawn} of {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            run.key,
            all.len(),
            t(Stage::Frame),
            us(stat(&all, Stage::Frame, 95.0, micros)),
            t(Stage::Input),
            t(Stage::SceneBuild),
            t(Stage::Draw),
            t(Stage::GpuWait),
            stat(&all, Stage::Geometry, 50.0, calls),
            stat(&all, Stage::Geometry, 50.0, items),
            stat(&all, Stage::Upload, 50.0, items),
        );
    }
    md.push('\n');
}

fn scenario(md: &mut String, run: &Run) {
    let all: Vec<&Sample> = run.samples.iter().collect();
    let drawn: Vec<&Sample> = all.iter().copied().filter(|s| s.drawn).collect();
    let _ = writeln!(md, "### {}\n", run.title);
    let _ = writeln!(md, "{}\n", run.setup);
    let probes = |p| {
        pct(
            all.iter().map(|s| {
                Stage::ALL
                    .iter()
                    .map(|&st| s.stages.get(st).calls)
                    .sum::<u64>() as f64
            }),
            p,
        )
    };
    let _ = writeln!(
        md,
        "frames drawn: {} of {}. the last turn's scene: {}. probe calls per turn: \
         p50 {}, p95 {}.\n",
        drawn.len(),
        all.len(),
        run.scene,
        probes(50.0),
        probes(95.0)
    );
    if let Some(gpu) = run.monitor_gpu_micros {
        let _ = writeln!(
            md,
            "the loop's perf monitor after the last turn: gpu {gpu} us \
             (`PerfSnapshot::gpu_micros`, the last reading fed).\n"
        );
    }
    for note in &run.notes {
        let _ = writeln!(md, "{note}\n");
    }
    table(md, &all);
    if !drawn.is_empty() && drawn.len() < all.len() {
        let _ = writeln!(md, "the {} drawn turns only:\n", drawn.len());
        table(md, &drawn);
    }
}

fn table(md: &mut String, samples: &[&Sample]) {
    let _ = writeln!(
        md,
        "| stage | items are | calls p50 | calls p95 | items p50 | items p95 | time p50 | time p95 |"
    );
    let _ = writeln!(md, "| --- | --- |{}", " ---: |".repeat(6));
    for stage in Stage::ALL {
        let unit = stage.items();
        let item = |p| match unit {
            Some(_) => format!("{}", stat(samples, stage, p, items)),
            None => "-".to_string(),
        };
        let _ = writeln!(
            md,
            "| {} | {} | {} | {} | {} | {} | {} | {} |",
            stage.name(),
            unit.unwrap_or("-"),
            stat(samples, stage, 50.0, calls),
            stat(samples, stage, 95.0, calls),
            item(50.0),
            item(95.0),
            us(stat(samples, stage, 50.0, micros)),
            us(stat(samples, stage, 95.0, micros)),
        );
    }
    md.push('\n');
}
