//! The dev mode without a GPU: its views by name, the page's share of the
//! window, the head's two layouts, what a press chooses, and every view
//! drawing from the frames a recorder keeps.
//!
//! A dev mode that exists turns the heap's counters on, a switch of the
//! process: every test that makes one holds `zones::test_lock`.

use super::*;
use crate::runtime::perf::stages::{Stage, StageCost, StageFrame};
use crate::runtime::perf::zones::{ThreadZones, ZoneFrame, ZoneRecord, test_lock};
use crate::runtime::perf::{MemoryStats, Message, PlotPoint};

const WINDOW: (f32, f32) = (1280.0, 800.0);

fn zone(name: &'static str, start_ns: u64, nanos: u64, depth: u32) -> ZoneRecord {
    ZoneRecord {
        name,
        start_ns,
        nanos,
        depth,
    }
}

/// A frame with two stages, two zones, a plot and a message.
fn report(index: u64) -> FrameReport {
    let cost = |nanos| StageCost {
        calls: 1,
        items: 4,
        nanos,
    };
    let stages = StageFrame::from_costs(&[
        (Stage::Frame, cost(2_000_000)),
        (Stage::SceneBuild, cost(1_000_000)),
    ]);
    let zones = ZoneFrame {
        threads: vec![ThreadZones {
            name: "main".to_owned(),
            zones: vec![
                zone("scene build", 100_000, 1_000_000 + index * 10_000, 1),
                zone("frame", 0, 2_000_000, 0),
            ],
        }],
        dropped: 0,
    };
    let mut report = FrameReport::new(
        index,
        stages,
        zones,
        &Compositor::new(),
        MemoryStats::default(),
    );
    report.plots = vec![PlotPoint {
        name: "scene nodes",
        value: 12.0 + index as f64,
    }];
    report.messages = vec![Message {
        at_ns: 500_000,
        thread: "main".to_owned(),
        text: format!("frame {index} said this"),
    }];
    report
}

fn recorder(frames: u64) -> Recorder {
    let mut recorder = Recorder::new();
    for index in 0..frames {
        recorder.push(report(index), Vec::new(), false);
    }
    recorder
}

fn open(tab: Tab) -> DevMode {
    let mut dev = DevMode::new(true);
    dev.set_open(true);
    dev.set_tab(tab);
    dev
}

/// The nodes `dev` draws on its own layer from `recorder`.
fn drawn(dev: &mut DevMode, recorder: &Recorder) -> usize {
    let mut c = Compositor::new();
    dev.draw(&mut c, &Theme::dark(), WINDOW, recorder, None);
    let layer = dev.layer.expect("an available dev mode makes its layer");
    let layers = c.layers();
    layers
        .iter()
        .find(|l| l.id == layer)
        .map_or(0, |l| l.nodes().len())
}

#[test]
fn every_view_has_a_name_that_parses_back() {
    for tab in Tab::ALL {
        assert_eq!(Tab::parse(&tab.name().to_uppercase()), Some(tab));
    }
    assert_eq!(Tab::ALL.len(), TABS);
    assert_eq!(Tab::parse("nope"), None);
}

#[test]
fn the_open_panel_takes_half_the_window_from_the_page() {
    let _lock = test_lock();
    let mut dev = DevMode::new(true);
    assert_eq!(dev.page_size(1280.0, 800.0), (1280.0, 800.0));
    dev.set_open(true);
    assert_eq!(dev.page_size(1280.0, 800.0), (1280.0, 400.0));
    dev.set_dock(Dock::Side);
    assert_eq!(dev.page_size(1280.0, 800.0), (640.0, 800.0));
    let mut none = DevMode::new(false);
    none.set_open(true);
    assert_eq!(none.page_size(1280.0, 800.0), (1280.0, 800.0));
}

#[test]
fn a_dev_mode_that_is_not_there_draws_nothing_and_takes_nothing() {
    let mut dev = DevMode::new(false);
    dev.set_open(true);
    let mut c = Compositor::new();
    let layers = c.layers().len();
    dev.draw(&mut c, &Theme::dark(), WINDOW, &recorder(2), None);
    assert_eq!(c.layers().len(), layers);
    let press = WidgetEvent::MouseDown { x: 640.0, y: 780.0 };
    let took = dev.event(&press, WINDOW.0, WINDOW.1, &Theme::dark(), &c);
    assert!(!took.taken && !took.changed && !took.resized);
}

#[test]
fn every_view_draws_from_the_frames_kept() {
    let _lock = test_lock();
    let (kept, none) = (recorder(6), Recorder::new());
    for tab in Tab::ALL {
        let with = drawn(&mut open(tab), &kept);
        let without = drawn(&mut open(tab), &none);
        if tab == Tab::Inspect {
            // its body is a control of the page, which no frame changes
            assert!(with >= without, "{tab:?}: {with} nodes, {without} without");
        } else {
            assert!(with > without, "{tab:?}: {with} nodes, {without} without");
        }
    }
}

#[test]
fn a_narrow_panel_puts_the_views_on_a_line_of_their_own() {
    let _lock = test_lock();
    let theme = Theme::dark();
    let mut dev = open(Tab::Frame);
    let wide = dev.layout(1280.0, 800.0, &theme);
    assert_eq!(wide.tabs[0].y, wide.dock.y, "one line when both fit");
    dev.set_dock(Dock::Side);
    let narrow = dev.layout(600.0, 800.0, &theme);
    assert!(
        narrow.tabs[0].y > narrow.dock.bottom(),
        "the views go under"
    );
    assert!(narrow.body.y > narrow.tabs[0].bottom());
    assert!(narrow.body.y > wide.body.y - wide.panel.unwrap().y);
}

#[test]
fn a_press_on_a_row_of_the_zones_table_chooses_that_zone() {
    let _lock = test_lock();
    let theme = Theme::dark();
    let kept = recorder(4);
    let mut dev = open(Tab::Zones);
    let mut c = Compositor::new();
    dev.draw(&mut c, &theme, WINDOW, &kept, None);
    assert_eq!(dev.zone, None, "the heaviest zone until one is chosen");
    let rows = dev.zone_rows.clone();
    assert_eq!(rows.len(), 2);
    let (row, name) = rows[1];
    let (x, y) = row.center();
    let took = dev.event(
        &WidgetEvent::MouseDown { x, y },
        WINDOW.0,
        WINDOW.1,
        &theme,
        &c,
    );
    assert!(took.taken && took.changed);
    assert_eq!(dev.zone, Some(name));
}

#[test]
fn the_frames_a_chart_covers_leave_the_tools_own_out() {
    let mut kept = recorder(3);
    kept.mark_tool();
    kept.push(report(3), Vec::new(), false);
    kept.push(report(4), Vec::new(), false);
    let indices: Vec<u64> = frames(&kept).iter().map(|f| f.index).collect();
    assert_eq!(indices, [0, 1, 2, 4]);
}

#[test]
fn the_panel_asks_for_a_frame_until_it_has_shown_the_latest() {
    let _lock = test_lock();
    let kept = recorder(2);
    let mut dev = open(Tab::Frame);
    assert!(dev.wants_frame(&kept), "the latest frame is not shown yet");
    let _ = drawn(&mut dev, &kept);
    assert!(
        !dev.wants_frame(&kept),
        "shown, and no GPU time to wait for"
    );
    dev.set_open(false);
    assert!(
        !dev.wants_frame(&recorder(3)),
        "a closed panel asks nothing"
    );
}
