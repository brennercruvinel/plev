//! The pipeline view over hand-built stage frames: phase totals, own
//! time, parts, the GPU phase's three sources and the frame's totals.

use super::super::pipeline::{Phase, Pipeline};
use super::super::stages::{Stage, StageCost, StageFrame};

fn cost(calls: u64, items: u64, nanos: u64) -> StageCost {
    StageCost {
        calls,
        items,
        nanos,
    }
}

fn timed(nanos: u64) -> StageCost {
    cost(1, 0, nanos)
}

/// A frame with every CPU phase and its parts.
fn full_frame() -> Vec<(Stage, StageCost)> {
    vec![
        (Stage::Frame, timed(10_000)),
        (Stage::Input, cost(1, 2, 500)),
        (Stage::HitTest, cost(2, 40, 100)),
        (Stage::Tick, timed(50)),
        (Stage::SceneBuild, cost(1, 300, 3_000)),
        (Stage::Projection, timed(200)),
        (Stage::Layout, cost(1, 30, 800)),
        (Stage::MeasureHit, cost(10, 0, 100)),
        (Stage::MeasureMiss, cost(2, 0, 400)),
        (Stage::Shape, cost(3, 90, 350)),
        (Stage::Bind, cost(12, 12, 0)),
        (Stage::Draw, timed(4_000)),
        (Stage::Resolve, timed(1_000)),
        (Stage::LayerHash, cost(3, 300, 150)),
        (Stage::Geometry, cost(1, 100, 400)),
        (Stage::TextResolve, cost(1, 64, 900)),
        (Stage::Upload, cost(4, 4096, 200)),
        (Stage::Encode, cost(1, 9, 1_500)),
    ]
}

#[test]
fn phases_come_in_frame_order_with_distinct_names() {
    let pipeline = Pipeline::of(&StageFrame::default());
    for (i, phase) in Phase::ALL.iter().enumerate() {
        assert_eq!(*phase as usize, i);
        assert_eq!(pipeline.phases[i].phase, *phase);
    }
    let names: std::collections::HashSet<_> = Phase::ALL.iter().map(|p| p.name()).collect();
    assert_eq!(names.len(), Phase::COUNT);
    assert!(Phase::ALL.iter().all(|p| p.is_cpu() != (*p == Phase::Gpu)));
}

#[test]
fn an_empty_frame_is_all_zero() {
    let pipeline = Pipeline::of(&StageFrame::default());
    assert!(
        pipeline
            .phases
            .iter()
            .all(|p| p.nanos == 0 && p.own_nanos == 0 && p.parts.is_empty())
    );
    assert_eq!(
        (pipeline.cpu_nanos, pipeline.gpu_nanos, pipeline.frame_nanos),
        (0, 0, 0)
    );
    assert_eq!(pipeline.budget_share(16_666_667), 0.0);
}

#[test]
fn each_cpu_phase_subtracts_its_timed_parts() {
    let pipeline = Pipeline::of(&StageFrame::from_costs(&full_frame()));

    let input = pipeline.phase(Phase::Input);
    assert_eq!((input.nanos, input.own_nanos, input.items), (500, 400, 2));
    assert_eq!(input.parts, [(Stage::HitTest, cost(2, 40, 100))]);

    let tick = pipeline.phase(Phase::Tick);
    assert_eq!((tick.nanos, tick.own_nanos), (50, 50));
    assert!(tick.parts.is_empty());

    // shaping is listed, not subtracted: 3000 - (200 + 800 + 100 + 400)
    let scene = pipeline.phase(Phase::Scene);
    assert_eq!(
        (scene.nanos, scene.own_nanos, scene.items),
        (3_000, 1_500, 300)
    );
    let parts: Vec<_> = scene.parts.iter().map(|(s, _)| *s).collect();
    assert_eq!(
        parts,
        [
            Stage::Projection,
            Stage::Layout,
            Stage::MeasureHit,
            Stage::MeasureMiss,
            Stage::Shape,
            Stage::Bind
        ]
    );

    // the resolve and the text resolve: 1900 - (150 + 400 + 900 + 200)
    let resolve = pipeline.phase(Phase::Resolve);
    assert_eq!((resolve.nanos, resolve.own_nanos), (1_900, 250));
    assert_eq!(resolve.items, 64, "glyphs");
    assert_eq!(resolve.parts.len(), 4);

    let encode = pipeline.phase(Phase::Encode);
    assert_eq!(
        (encode.nanos, encode.own_nanos, encode.items),
        (1_500, 1_500, 9)
    );

    assert_eq!(pipeline.cpu_nanos, 500 + 50 + 3_000 + 1_900 + 1_500);
    assert_eq!(pipeline.frame_nanos, 10_000);
    assert_eq!(pipeline.other_nanos, 10_000 - 6_950);
}

#[test]
fn parts_that_did_not_run_are_left_out_and_own_time_saturates() {
    let frame = StageFrame::from_costs(&[
        (Stage::Resolve, timed(100)),
        // hashed before the frame too (`needs_render`): more than the phase
        (Stage::LayerHash, cost(6, 600, 300)),
    ]);
    let resolve = Pipeline::of(&frame).phase(Phase::Resolve).clone();
    assert_eq!((resolve.nanos, resolve.own_nanos), (100, 0));
    assert_eq!(resolve.parts, [(Stage::LayerHash, cost(6, 600, 300))]);
}

#[test]
fn without_the_app_frame_the_frame_is_the_cpu_phases() {
    let frame = StageFrame::from_costs(&[(Stage::Tick, timed(300)), (Stage::Encode, timed(700))]);
    let pipeline = Pipeline::of(&frame);
    assert_eq!((pipeline.cpu_nanos, pipeline.frame_nanos), (1_000, 1_000));
    assert_eq!(pipeline.other_nanos, 0);
    assert_eq!(pipeline.budget_share(2_000), 0.5);
    assert_eq!(pipeline.budget_share(0), 0.0);
}

#[test]
fn the_gpu_phase_is_the_span_cut_into_layers_composite_and_the_rest() {
    let frame = StageFrame::from_costs(&[
        (Stage::GpuWait, timed(9_000)),
        (Stage::GpuSpan, timed(5_000)),
        (Stage::GpuLayers, cost(1, 3, 3_000)),
        (Stage::GpuComposite, timed(1_500)),
    ]);
    let pipeline = Pipeline::of(&frame);
    let gpu = pipeline.phase(Phase::Gpu);
    assert_eq!((gpu.nanos, gpu.items), (5_000, 3), "the span, not the wait");
    assert_eq!(pipeline.gpu_nanos, 5_000);
    assert_eq!(pipeline.gpu_other_nanos, 500);
    assert_eq!(gpu.own_nanos, 500);
    let parts: Vec<_> = gpu.parts.iter().map(|(s, _)| *s).collect();
    assert_eq!(parts, [Stage::GpuLayers, Stage::GpuComposite]);
    assert_eq!(pipeline.cpu_nanos, 0, "the gpu is not a cpu phase");
}

#[test]
fn two_readings_in_one_frame_read_as_their_mean() {
    let frame = StageFrame::from_costs(&[
        (Stage::GpuSpan, cost(2, 0, 8_000)),
        (Stage::GpuLayers, cost(2, 4, 4_000)),
        (Stage::GpuComposite, cost(2, 0, 2_000)),
    ]);
    let pipeline = Pipeline::of(&frame);
    assert_eq!(pipeline.gpu_nanos, 4_000);
    assert_eq!(pipeline.gpu_other_nanos, 1_000);
    assert_eq!(pipeline.phase(Phase::Gpu).items, 2, "passes per reading");
}

#[test]
fn without_timestamps_the_gpu_phase_is_the_wait_or_nothing() {
    let waited = Pipeline::of(&StageFrame::from_costs(&[(Stage::GpuWait, timed(7_000))]));
    let gpu = waited.phase(Phase::Gpu);
    assert_eq!((gpu.nanos, gpu.own_nanos), (7_000, 7_000));
    assert!(gpu.parts.is_empty());
    assert_eq!(waited.gpu_other_nanos, 0);

    let neither = Pipeline::of(&StageFrame::from_costs(&[(Stage::Encode, timed(10))]));
    assert_eq!(neither.gpu_nanos, 0);
}
