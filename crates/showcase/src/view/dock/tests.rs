//! Headless dock-view tests: geometry at narrow and wide viewports, and
//! hit areas asserted against the very nodes `render` pushes (one source
//! of truth for rects). No GPU: scenes build into a plain compositor.

use super::*;
use plev::compositor::LayerId;
use showcase::model::dock::DockState;

const DT: f32 = 1.0 / 60.0;

fn content(vw: f32) -> Rect {
    Rect::new(288.0, 118.0, (vw - 368.0).max(200.0), 682.0)
}

fn step(section: &mut DockSection, frames: usize) {
    for _ in 0..frames {
        section.tick(DT);
    }
}

fn scene(section: &mut DockSection, content: Rect) -> Vec<SceneNode> {
    let mut c = Compositor::new();
    c.begin_frame();
    section.render(&mut c, content, &Theme::hoff());
    c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec()
}

#[test]
fn dock_geometry_holds_at_narrow_and_wide_viewports() {
    for vw in [600.0_f32, 1500.0] {
        let mut section = DockSection::new();
        let content = content(vw);
        assert_eq!(section.content_height(content), content.h);
        let collapsed = section.model.dock_rect(content);
        assert!(
            collapsed.x >= content.x,
            "{vw}: dock starts inside the stage"
        );
        assert!(collapsed.x + collapsed.w <= content.x + content.w + 0.5);
        assert!(collapsed.y + collapsed.h <= content.y + content.h);
        let (cx, _) = collapsed.center();
        assert!((cx - (content.x + content.w / 2.0)).abs() < 0.5, "centered");

        let (ax, ay) = section.model.avatar_rect(1, content).center();
        let r = section.handle_event(&WidgetEvent::MouseDown { x: ax, y: ay }, content);
        assert!(r.clicked, "{vw}: clicking an avatar expands");
        step(&mut section, 120);
        let expanded = section.model.dock_rect(content);
        assert!(expanded.w <= content.w + 0.5, "{vw}: morph never overflows");
        assert!(expanded.w >= collapsed.w - 0.5);
        if vw > 1000.0 {
            assert!(expanded.w > collapsed.w + 100.0, "wide stage opens visibly");
        }
        let send = section.send_rect(content);
        let field = section.input_rect(content);
        for r in [send, field] {
            assert!(r.x >= expanded.x - 0.5, "{vw}: control inside the dock");
            assert!(r.x + r.w <= expanded.x + expanded.w + 0.5);
        }
        assert!(
            field.x > section.model.avatar_rect(1, content).x,
            "field sits right of the selected avatar"
        );
    }
}

/// Hit areas and pixels share one source: every avatar circle and the
/// send circle appear in the scene at exactly the hit-test rects.
#[test]
fn hit_areas_match_drawn_geometry() {
    let mut section = DockSection::new();
    let content = content(1500.0);
    let nodes = scene(&mut section, content);
    let rects: Vec<Rect> = nodes
        .iter()
        .filter_map(|n| match n {
            SceneNode::RoundedRect { x, y, w, h, .. } => Some(Rect::new(*x, *y, *w, *h)),
            _ => None,
        })
        .collect();
    let drawn = |r: Rect| {
        rects.iter().any(|d| {
            (d.x - r.x).abs() < 0.01
                && (d.y - r.y).abs() < 0.01
                && (d.w - r.w).abs() < 0.01
                && (d.h - r.h).abs() < 0.01
        })
    };
    for i in 0..AVATARS {
        assert!(
            drawn(section.model.avatar_rect(i, content)),
            "avatar {i} drawn as hit"
        );
    }
    assert!(
        drawn(section.send_rect(content)),
        "send button drawn as hit"
    );
    assert!(
        nodes
            .iter()
            .any(|n| matches!(n, SceneNode::BackdropBlur { .. })),
        "the dock pill frosts its backdrop (real glass)"
    );
}

#[test]
fn hover_lifts_the_avatar_and_settles_back_at_rest() {
    let mut section = DockSection::new();
    let content = content(1500.0);
    let (x, y) = section.model.avatar_rect(2, content).center();
    assert!(
        section
            .handle_event(&WidgetEvent::MouseMove { x, y }, content)
            .changed
    );
    step(&mut section, 30);
    assert!(section.model.avatar_lift(2) > 4.0, "hover lifts the avatar");
    let same = section.handle_event(&WidgetEvent::MouseMove { x, y }, content);
    assert!(!same.changed, "unchanged hover must not request frames");
    let off = WidgetEvent::MouseMove {
        x: content.x + 4.0,
        y: content.y + 4.0,
    };
    assert!(section.handle_event(&off, content).changed);
    step(&mut section, 60);
    assert!(section.model.avatar_lift(2) < 0.01);
    assert!(!section.tick(DT), "fully at rest after the lift returns");
    let miss = WidgetEvent::MouseDown {
        x: content.x + 4.0,
        y: content.y + 4.0,
    };
    assert_eq!(section.handle_event(&miss, content), EventResult::IGNORED);
}

#[test]
fn click_expands_send_flashes_and_the_dock_settles_shut() {
    let mut section = DockSection::new();
    let content = content(1500.0);
    let (x, y) = section.model.avatar_rect(0, content).center();
    assert!(
        section
            .handle_event(&WidgetEvent::MouseDown { x, y }, content)
            .clicked
    );
    step(&mut section, 60);
    assert_eq!(section.model.state(), DockState::Expanded(0));
    assert!(section.tick(DT), "blinking caret keeps frames coming");

    // A faded-out avatar under the message field must not steal clicks.
    let (gx, gy) = section.model.avatar_rect(2, content).center();
    let r = section.handle_event(&WidgetEvent::MouseDown { x: gx, y: gy }, content);
    assert!(
        r.handled && !r.clicked,
        "ghost avatar consumed by the dock body"
    );
    assert_eq!(section.model.state(), DockState::Expanded(0));

    let (sx, sy) = section.send_rect(content).center();
    assert!(
        section
            .handle_event(&WidgetEvent::MouseDown { x: sx, y: sy }, content)
            .clicked
    );
    assert!(section.model.flash_alpha() > 0.9, "send flash fires");
    step(&mut section, 180);
    assert_eq!(section.model.state(), DockState::Idle);
    assert!(!section.tick(DT), "no busy loop after the send completes");
}

#[test]
fn expanded_scene_shows_field_placeholder_and_caret() {
    let mut section = DockSection::new();
    let content = content(1500.0);
    let (x, y) = section.model.avatar_rect(1, content).center();
    section.handle_event(&WidgetEvent::MouseDown { x, y }, content);
    step(&mut section, 66); // expanded, caret in its visible half
    let nodes = scene(&mut section, content);
    let field = section.input_rect(content);
    assert!(
        nodes
            .iter()
            .any(|n| matches!(n, SceneNode::RoundedRect { x, w, .. }
            if (*x - field.x).abs() < 0.01 && (*w - field.w).abs() < 0.01)),
        "message field drawn at its hit rect"
    );
    assert!(
        nodes.iter().any(|n| matches!(n, SceneNode::Text { key, .. }
            if key.text == "Message Bea")),
        "placeholder follows the selected contact"
    );
    assert!(
        nodes
            .iter()
            .any(|n| matches!(n, SceneNode::Rect { w, h, .. }
            if *w == 2.0 && *h > 8.0)),
        "blinking caret visible"
    );
}
