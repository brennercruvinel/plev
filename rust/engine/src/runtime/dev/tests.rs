//! The dev mode without a GPU: its views by name and icon, the page's
//! share of the window, what a press on each place does, the frame the
//! panel waits for, every view drawing from the frames a recorder keeps,
//! and the findings read from a frame.
//!
//! A dev mode that exists turns the heap's counters on and holds the
//! marks, switches of the process: every test that makes one holds
//! `zones::test_lock`.

use super::insights::Severity;
use super::*;
use crate::runtime::perf::stages::{Stage, StageCost, StageFrame};
use crate::runtime::perf::zones::{ThreadZones, ZoneFrame, ZoneRecord, test_lock};
use crate::runtime::perf::{FrameReport, GpuReading, LayerReport, MemoryStats, Message, PlotPoint};
use crate::widgets::icons;

const WINDOW: (f32, f32) = (1280.0, 800.0);

fn zone(name: &'static str, start_ns: u64, nanos: u64, depth: u32) -> ZoneRecord {
    ZoneRecord {
        name,
        start_ns,
        nanos,
        depth,
    }
}

/// A frame whose scene build took `scene_ns`, with two zones, a plot and
/// a message.
fn report_of(index: u64, scene_ns: u64) -> FrameReport {
    let cost = |nanos| StageCost {
        calls: 1,
        items: 4,
        nanos,
    };
    let stages = StageFrame::from_costs(&[
        (Stage::Frame, cost(scene_ns * 2)),
        (Stage::SceneBuild, cost(scene_ns)),
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
        name: "controls bound",
        value: 12.0 + index as f64,
    }];
    report.messages = vec![Message {
        at_ns: 500_000,
        thread: "main".to_owned(),
        level: Level::Info,
        text: format!("frame {index} said this"),
    }];
    report
}

fn report(index: u64) -> FrameReport {
    report_of(index, 1_000_000)
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

/// Where the last frame drawn offered `act`.
fn place(dev: &DevMode, act: &Act) -> Rect {
    dev.regions
        .iter()
        .rev()
        .find(|(_, offered)| offered == act)
        .map(|(rect, _)| *rect)
        .unwrap_or_else(|| panic!("no place for {act:?}"))
}

/// A press and its release in the middle of where `act` is offered.
fn press(dev: &mut DevMode, act: &Act) -> Took {
    let (x, y) = place(dev, act).center();
    let c = Compositor::new();
    let took = dev.event(&WidgetEvent::MouseDown { x, y }, WINDOW.0, WINDOW.1, &c);
    let up = dev.event(&WidgetEvent::MouseUp { x, y }, WINDOW.0, WINDOW.1, &c);
    assert!(up.taken, "the release of a press taken is not the page's");
    took
}

#[test]
fn every_view_has_a_name_that_parses_back_and_an_icon_that_is_there() {
    for tab in Tab::ALL {
        assert_eq!(Tab::parse(&tab.name().to_uppercase()), Some(tab));
        assert!(icons::has_icon(tab.icon()), "{tab:?}: {}", tab.icon());
    }
    assert_eq!(Tab::ALL.len(), TABS);
    assert_eq!(Tab::parse("nope"), None);
    // what the head, the log and the findings draw with
    for name in [
        "settings",
        "mouse-pointer-2",
        "panel-right",
        "panel-bottom",
        "x",
        "trash",
        "info",
        "alert-triangle",
        "circle-x",
        "circle-check",
    ] {
        assert!(icons::has_icon(name), "{name}");
    }
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
    let took = dev.event(&press, WINDOW.0, WINDOW.1, &c);
    assert!(!took.taken && !took.changed && !took.resized);
}

#[test]
fn the_gear_sits_at_the_pages_lower_right_and_a_press_opens_the_panel() {
    let _lock = test_lock();
    let mut dev = DevMode::new(true);
    let _ = drawn(&mut dev, &Recorder::new());
    let gear = place(&dev, &Act::Open);
    assert_eq!(gear, dev.gear(WINDOW.0, WINDOW.1));
    assert!(gear.w <= 32.0 && gear.h <= 32.0, "a small button: {gear:?}");
    assert!(gear.x > WINDOW.0 * 0.9 && gear.y > WINDOW.1 * 0.9);
    // a press beside it is the page's
    let c = Compositor::new();
    let beside = WidgetEvent::MouseDown { x: 640.0, y: 400.0 };
    assert!(!dev.event(&beside, WINDOW.0, WINDOW.1, &c).taken);

    let took = press(&mut dev, &Act::Open);
    assert!(took.taken && took.changed && took.resized);
    assert!(dev.is_open());
    // open, the gear is gone and the close is there
    let _ = drawn(&mut dev, &recorder(2));
    assert!(dev.regions.iter().all(|(_, act)| *act != Act::Open));
    let took = press(&mut dev, &Act::Close);
    assert!(took.resized && !dev.is_open());
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
fn a_press_on_a_tab_shows_its_view_and_a_narrow_panel_keeps_every_tab() {
    let _lock = test_lock();
    let kept = recorder(3);
    let mut dev = open(Tab::Frame);
    let _ = drawn(&mut dev, &kept);
    let wide = place(&dev, &Act::Tab(Tab::Log));
    let took = press(&mut dev, &Act::Tab(Tab::Log));
    assert!(took.changed && !took.resized);
    assert_eq!(dev.tab(), Tab::Log);

    // docked at the side of a narrow window the names go, the tabs stay,
    // and what stands beside them takes a line of its own
    dev.set_dock(Dock::Side);
    for window in [760.0, 480.0] {
        let mut c = Compositor::new();
        dev.draw(&mut c, &Theme::dark(), (window, 800.0), &kept, None);
        let panel = window / 2.0;
        for tab in Tab::ALL {
            let at = place(&dev, &Act::Tab(tab));
            assert!(
                at.x >= panel && at.right() <= window,
                "{tab:?} in a panel {panel} wide: {at:?}"
            );
        }
        let close = place(&dev, &Act::Close);
        assert!(close.x >= panel && close.right() <= window);
        let narrow = place(&dev, &Act::Tab(Tab::Frame));
        assert!(narrow.w < wide.w, "a tab not shown keeps its icon only");
        assert!(
            close.y > narrow.y,
            "the close goes under the tabs: {close:?} {narrow:?}"
        );
    }
}

#[test]
fn the_head_chooses_the_frame_shown_and_the_dock() {
    let _lock = test_lock();
    let kept = recorder(3);
    let mut dev = open(Tab::Frame);
    let _ = drawn(&mut dev, &kept);
    assert_eq!(dev.follow(), Follow::Latest);
    press(&mut dev, &Act::Follow(Follow::Slowest));
    assert_eq!(dev.follow(), Follow::Slowest);
    // a bar of the history shows its frame, and Live goes back
    let _ = drawn(&mut dev, &kept);
    press(&mut dev, &Act::Follow(Follow::Frame(1)));
    assert_eq!(dev.follow(), Follow::Frame(1));
    let _ = drawn(&mut dev, &kept);
    press(&mut dev, &Act::Follow(Follow::Latest));
    assert_eq!(dev.follow(), Follow::Latest);

    let took = press(&mut dev, &Act::Dock);
    assert!(took.resized);
    assert_eq!(dev.dock(), Dock::Side);
}

#[test]
fn a_drag_of_the_panels_edge_gives_it_more_or_less_of_the_window() {
    let _lock = test_lock();
    let mut dev = open(Tab::Frame);
    let _ = drawn(&mut dev, &recorder(2));
    let c = Compositor::new();
    let event = |dev: &mut DevMode, ev| dev.event(&ev, WINDOW.0, WINDOW.1, &c);
    let down = event(&mut dev, WidgetEvent::MouseDown { x: 640.0, y: 400.0 });
    assert!(down.taken, "the edge is where the page ends");
    let moved = event(&mut dev, WidgetEvent::MouseMove { x: 640.0, y: 200.0 });
    assert!(moved.resized && moved.taken);
    assert_eq!(dev.page_size(WINDOW.0, WINDOW.1), (1280.0, 200.0));
    // never the whole window, never nothing
    event(&mut dev, WidgetEvent::MouseMove { x: 640.0, y: 0.0 });
    assert_eq!(dev.page_size(WINDOW.0, WINDOW.1).1, 160.0);
    event(&mut dev, WidgetEvent::MouseMove { x: 640.0, y: 800.0 });
    assert_eq!(dev.page_size(WINDOW.0, WINDOW.1).1, 640.0);
    assert!(event(&mut dev, WidgetEvent::MouseUp { x: 640.0, y: 800.0 }).taken);
    let after = event(&mut dev, WidgetEvent::MouseMove { x: 640.0, y: 300.0 });
    assert!(!after.resized, "released, the edge stays");
}

#[test]
fn the_pointer_over_a_place_asks_for_a_frame_once() {
    let _lock = test_lock();
    let mut dev = open(Tab::Frame);
    let _ = drawn(&mut dev, &recorder(2));
    let c = Compositor::new();
    let (x, y) = place(&dev, &Act::Tab(Tab::Gpu)).center();
    let over = WidgetEvent::MouseMove { x, y };
    assert!(dev.event(&over, WINDOW.0, WINDOW.1, &c).changed);
    let again = WidgetEvent::MouseMove { x: x + 1.0, y };
    let took = dev.event(&again, WINDOW.0, WINDOW.1, &c);
    assert!(
        took.taken && !took.changed,
        "the same place: nothing to draw"
    );
}

#[test]
fn a_press_on_a_row_of_the_zones_table_chooses_that_zone() {
    let _lock = test_lock();
    let kept = recorder(4);
    let mut dev = open(Tab::Zones);
    let _ = drawn(&mut dev, &kept);
    assert_eq!(dev.zone, None, "the heaviest zone until one is chosen");
    let took = press(&mut dev, &Act::Zone("frame"));
    assert!(took.taken && took.changed);
    assert_eq!(dev.zone, Some("frame"));
}

#[test]
fn the_log_filters_to_a_level_and_clears_what_was_said() {
    let _lock = test_lock();
    let mut kept = recorder(3);
    let mut warned = report(3);
    warned.messages[0].level = Level::Warn;
    kept.push(warned, Vec::new(), false);
    let mut dev = open(Tab::Log);
    let _ = drawn(&mut dev, &kept);
    assert_eq!(dev.log.listed, 4, "a line a frame");
    press(&mut dev, &Act::Level(Some(Level::Warn)));
    let _ = drawn(&mut dev, &kept);
    assert_eq!(dev.log.listed, 1, "the warning alone");
    press(&mut dev, &Act::Level(None));
    press(&mut dev, &Act::ClearLog);
    let _ = drawn(&mut dev, &kept);
    assert_eq!(dev.log.listed, 0, "nothing said since the clear");
    // a line said after the clear shows
    kept.push(report(4), Vec::new(), false);
    let _ = drawn(&mut dev, &kept);
    assert_eq!(dev.log.listed, 1);
}

#[test]
fn the_picker_arms_the_pointer_and_a_press_on_the_page_is_not_the_pages() {
    let _lock = test_lock();
    let mut dev = open(Tab::Frame);
    let _ = drawn(&mut dev, &recorder(2));
    press(&mut dev, &Act::Pick);
    assert_eq!(dev.tab(), Tab::Inspect, "picking opens the view it fills");
    let c = Compositor::new();
    let on_page = WidgetEvent::MouseDown { x: 300.0, y: 100.0 };
    let took = dev.event(&on_page, WINDOW.0, WINDOW.1, &c);
    assert!(took.taken, "the press chooses, it does not reach the page");
    let up = WidgetEvent::MouseUp { x: 300.0, y: 100.0 };
    assert!(dev.event(&up, WINDOW.0, WINDOW.1, &c).taken);
    // one press, then the page has its pointer back
    assert!(!dev.event(&on_page, WINDOW.0, WINDOW.1, &c).taken);
}

/// A frame with a control bound over a ground and a label, and an icon
/// beside it that nothing binds.
fn page() -> Compositor {
    let mut c = Compositor::new();
    build(&mut c);
    c
}

/// The frame of [`page`], built into `c` as an app builds every frame.
fn build(c: &mut Compositor) {
    use crate::drawing::compositor::{SceneNode, TextNodeKey};
    c.begin_frame();
    let fill = |x, y, w, h| SceneNode::Rect {
        x,
        y,
        w,
        h,
        color: [1.0; 4],
    };
    c.push(fill(0.0, 0.0, 1280.0, 400.0));
    let button = SemId::parse("page.button").unwrap();
    c.bind(&button, [100.0, 100.0, 120.0, 40.0]);
    c.push(fill(100.0, 100.0, 120.0, 40.0));
    c.push(SceneNode::Text {
        key: TextNodeKey::new("Save", 14.0, 20.0, None),
        x: 140.0,
        y: 110.0,
        color: [0.0, 0.0, 0.0, 1.0],
    });
    c.push(fill(400.0, 100.0, 24.0, 24.0));
}

#[test]
fn the_picker_chooses_the_control_or_the_node_whichever_is_more_particular() {
    let _lock = test_lock();
    let c = page();
    let mut dev = open(Tab::Inspect);
    // the control's own ground: the control
    dev.pick_at(&c, 104.0, 104.0);
    assert_eq!(dev.picked_kind(), Some("Control"));
    assert_eq!(dev.picked().map(SemId::as_str), Some("page.button"));
    // its label, smaller than it: the text node
    dev.pick_at(&c, 150.0, 120.0);
    assert_eq!(dev.picked_kind(), Some("Text"));
    assert_eq!(dev.picked(), None);
    // what the page gave no id is chosen all the same
    dev.pick_at(&c, 410.0, 110.0);
    assert_eq!(dev.picked_kind(), Some("Rect"));
    // the page's ground under everything
    dev.pick_at(&c, 900.0, 300.0);
    assert_eq!(dev.picked_kind(), Some("Rect"));
    dev.pick_at(&c, 2000.0, 2000.0);
    assert_eq!(dev.picked_kind(), None, "nothing is drawn there");
}

#[test]
fn a_node_chosen_draws_what_it_is_and_never_the_panel_itself() {
    let _lock = test_lock();
    let mut c = page();
    let mut dev = open(Tab::Inspect);
    dev.draw(&mut c, &Theme::dark(), WINDOW, &recorder(2), None);
    let empty = c.layer(dev.layer.unwrap()).unwrap().nodes().len();
    // a point of the open panel: the picker leaves its own layer out and
    // answers with what the page drew under it, or nothing
    dev.pick_at(&c, 640.0, 600.0);
    assert_eq!(dev.picked_kind(), None, "the panel is not a thing to pick");
    dev.pick_at(&c, 150.0, 120.0);
    build(&mut c);
    dev.draw(&mut c, &Theme::dark(), WINDOW, &recorder(2), None);
    let with = c.layer(dev.layer.unwrap()).unwrap().nodes().len();
    assert!(
        with > empty,
        "the text's lines and its ring: {with} > {empty}"
    );
}

#[test]
fn a_press_on_a_line_of_the_log_opens_it_whole_and_another_closes_it() {
    let _lock = test_lock();
    let mut kept = recorder(1);
    let mut long = report(1);
    long.messages[0].text = "a line of the log that says a great deal ".repeat(12);
    kept.push(long, Vec::new(), false);
    let mut dev = open(Tab::Log);
    let _ = drawn(&mut dev, &kept);
    let line = Act::Line(1);
    let shut = place(&dev, &line);
    press(&mut dev, &line);
    let _ = drawn(&mut dev, &kept);
    let whole = place(&dev, &line);
    assert!(
        whole.h > shut.h * 1.9,
        "its text takes the lines it needs: {} then {}",
        shut.h,
        whole.h
    );
    press(&mut dev, &line);
    let _ = drawn(&mut dev, &kept);
    assert_eq!(place(&dev, &line).h, shut.h, "one row again");
}

#[test]
fn a_parent_of_the_tree_folds_the_lines_under_it() {
    use accord::{Role, SnapshotBuilder};
    let _lock = test_lock();
    let mut b = SnapshotBuilder::new();
    b.node("list", Role::Region, "A list");
    b.node("list.one", Role::Button, "One").parent("list");
    b.node("list.two", Role::Button, "Two").parent("list");
    let snapshot = b.finish().expect("a well formed tree");
    let id = |text: &str| SemId::parse(text).unwrap();
    let mut dev = open(Tab::Inspect);
    let mut c = Compositor::new();
    dev.draw(
        &mut c,
        &Theme::dark(),
        WINDOW,
        &recorder(1),
        Some(&snapshot),
    );
    let lines = |dev: &DevMode| {
        let shown = |(_, act): &(Rect, Act)| matches!(act, Act::Control(_));
        dev.regions.iter().filter(|r| shown(r)).count()
    };
    assert_eq!(lines(&dev), 3);
    assert!(
        dev.regions
            .iter()
            .all(|(_, act)| *act != Act::Fold(id("list.one"))),
        "a line that holds nothing has no fold"
    );
    press(&mut dev, &Act::Fold(id("list")));
    c.begin_frame();
    dev.draw(
        &mut c,
        &Theme::dark(),
        WINDOW,
        &recorder(1),
        Some(&snapshot),
    );
    assert_eq!(lines(&dev), 1, "the two under it are folded away");
    press(&mut dev, &Act::Fold(id("list")));
    c.begin_frame();
    dev.draw(
        &mut c,
        &Theme::dark(),
        WINDOW,
        &recorder(1),
        Some(&snapshot),
    );
    assert_eq!(lines(&dev), 3);
}

#[test]
fn a_launch_asks_for_the_gear_or_for_a_view() {
    let _lock = test_lock();
    assert!(!DevMode::from_request("").is_available());
    assert!(!DevMode::from_request("0").is_available());
    let gear = DevMode::from_request("1");
    assert!(gear.is_available() && !gear.is_open());
    let zones = DevMode::from_request("zones");
    assert!(zones.is_open());
    assert_eq!(zones.tab(), Tab::Zones);
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

#[test]
fn the_panel_asks_for_the_frame_that_shows_the_gpu_time_it_waited_for() {
    let _lock = test_lock();
    let reading = |frame| GpuReading {
        frame,
        span: Some(1_000),
        ..GpuReading::default()
    };
    let mut kept = Recorder::new();
    kept.set_on(true);
    kept.push(report(0), Vec::new(), true);
    let mut dev = open(Tab::Gpu);
    let _ = drawn(&mut dev, &kept);
    assert!(dev.wants_frame(&kept), "frame 0 waits for its timestamps");
    // the tool's frame reads them back: what the panel drew is now stale
    kept.mark_tool();
    kept.push(report(1), vec![reading(0)], true);
    assert!(
        dev.wants_frame(&kept),
        "the time came in after the panel was drawn"
    );
    let _ = drawn(&mut dev, &kept);
    assert!(!dev.wants_frame(&kept), "shown whole: nothing more to ask");
    kept.set_on(false);
}

/// The findings about the latest frame of `kept`, as the Frame view
/// reads them.
fn found(kept: &Recorder) -> Vec<insights::Insight> {
    let shown = kept.latest().expect("a frame");
    insights::of(kept, shown, LayerId(u32::MAX))
}

#[test]
fn the_findings_open_with_the_frame_against_the_budget() {
    let mut inside = Recorder::new();
    inside.push(report_of(0, 1_000_000), Vec::new(), false);
    let first = &found(&inside)[0];
    assert!(first.title.starts_with("Inside the budget"), "{first:?}");
    assert!(found(&inside).iter().all(|i| i.severity != Severity::Bad));

    let mut over = Recorder::new();
    over.push(report_of(0, 40_000_000), Vec::new(), false);
    let first = &found(&over)[0];
    assert_eq!(first.severity, Severity::Bad, "the worst is first");
    assert!(first.title.starts_with("Over budget"), "{first:?}");
    assert!(first.detail.starts_with("scene is 100%"), "{first:?}");
}

#[test]
fn the_findings_say_what_was_drawn_for_nothing_and_what_the_log_holds() {
    let layer = |id, redrawn, nodes| LayerReport {
        id: LayerId(id),
        z_order: id as i32,
        visible: true,
        opacity: 1.0,
        redrawn,
        nodes,
        glyphs: 0,
        draws: 0,
        effects: 0,
    };
    let mut frame = report(0);
    frame.layers = vec![layer(0, true, 12), layer(1, true, 0), layer(2, true, 0)];
    frame.messages[0].level = Level::Warn;
    let mut kept = Recorder::new();
    kept.push(frame, Vec::new(), false);
    let all = found(&kept);
    let hollow = all
        .iter()
        .find(|i| i.title.contains("hold nothing"))
        .expect("two layers redrawn with no node");
    assert_eq!((hollow.severity, hollow.tab), (Severity::Warn, Tab::Gpu));
    assert!(hollow.title.starts_with("2 "), "{hollow:?}");
    let log = all
        .iter()
        .find(|i| i.tab == Tab::Log)
        .expect("a warning was said");
    assert_eq!(log.title, "1 warning in the log");
    assert_eq!(log.detail, "frame 0 said this");

    // nothing redrawn is said too, as what it is: the engine at rest
    let mut idle = report(1);
    idle.layers = vec![layer(0, false, 12), layer(1, false, 0)];
    let mut kept = Recorder::new();
    kept.push(idle, Vec::new(), false);
    let rest = found(&kept);
    let reused = rest
        .iter()
        .find(|i| i.title == "Nothing redrawn")
        .expect("every layer reused");
    assert_eq!(reused.severity, Severity::Good);
}
