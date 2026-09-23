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

pub fn markdown(cfg: &Config, runs: &[Run], build: &str) -> String {
    let v = cfg.viewport;
    let mut md = String::new();
    let _ = writeln!(
        md,
        "{} measured turns per scenario after {} warm-up turns, fixed tick of {:.2} ms, \
         viewport {}x{} logical at scale {} ({}x{} physical), theme {}, {build} build, \
         stages {}. time is microseconds; p50 and p95 are nearest-rank over the turns.\n",
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
    for run in runs {
        scenario(&mut md, run);
    }
    md
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
