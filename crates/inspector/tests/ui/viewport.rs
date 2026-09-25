//! the screen at a phone's width and at a desktop's: every part inside
//! the content rect, none over another, the frame at its own aspect,
//! nothing drawn past the window, and a click on the frame still picks
//! the control under it.

use comps::content::CodeBlock;
use comps::{Rect, Theme};
use engine::compositor::{LayerId, SceneNode};
use engine::theme::{Breakpoint, SidebarMode};

use crate::support::{Fallible, click_control, draw, id, open};

fn inside(inner: Rect, outer: Rect) -> bool {
    let e = 0.5;
    inner.x >= outer.x - e
        && inner.y >= outer.y - e
        && inner.right() <= outer.right() + e
        && inner.bottom() <= outer.bottom() + e
}

fn apart(a: Rect, b: Rect) -> bool {
    a.right() <= b.x + 0.5
        || b.right() <= a.x + 0.5
        || a.bottom() <= b.y + 0.5
        || b.bottom() <= a.y + 0.5
}

fn check(width: f32, height: f32, breakpoint: Breakpoint) -> Fallible {
    let mut view = open("a2ppa", width, height)?;
    let theme = Theme::default();
    // before the commit, the button on screen.
    view.seek_at(105_000);
    let layout = view.layout();
    assert_eq!(layout.shell.breakpoint, breakpoint);
    let content = layout.shell.content;
    for part in [layout.frame, layout.strip, layout.panel] {
        assert!(inside(part, content), "{width}: {part:?} in {content:?}");
        assert!(part.w > 0.0 && part.h > 0.0, "{width}: {part:?}");
    }
    assert!(apart(layout.frame, layout.strip), "{width}");
    assert!(apart(layout.frame, layout.panel), "{width}");
    assert!(apart(layout.strip, layout.panel), "{width}");
    assert!(
        layout.panel.h >= CodeBlock::row_h(&theme) * 3.0,
        "{width}: the panel has room for rows: {:?}",
        layout.panel
    );
    assert_eq!(
        layout.strip.h,
        view.strip().height(layout.strip.w, &theme),
        "{width}"
    );

    let fit = view.fit().ok_or("a frame")?;
    let frame = fit.frame();
    assert!(inside(frame, layout.frame), "{width}");
    assert!(
        ((frame.w / frame.h) - 1280.0 / 800.0).abs() < 0.01,
        "{width}: the aspect"
    );
    let g = view.strip().geometry(layout.strip, &theme);
    for r in g.chips.iter().chain(&g.lanes).chain([&g.scrub, &g.track]) {
        assert!(inside(*r, layout.strip), "{width}: {r:?}");
    }

    let submit = id("commit.submit");
    click_control(&mut view, &submit)?;
    assert_eq!(view.picked(), Some(&submit), "{width}");

    // nothing on the screen's own layer passes the window.
    let c = draw(&mut view);
    let window = Rect::new(0.0, 0.0, width, height);
    let screen = c.layer(LayerId::DEFAULT).ok_or("the default layer")?;
    for node in screen.nodes() {
        let r = match node {
            SceneNode::Rect { x, y, w, h, .. }
            | SceneNode::RoundedRect { x, y, w, h, .. }
            | SceneNode::GradientRect { x, y, w, h, .. } => Rect::new(*x, *y, *w, *h),
            SceneNode::Text { x, y, .. } => Rect::new(*x, *y, 0.0, 0.0),
            _ => continue,
        };
        assert!(inside(r, window), "{width}: {node:?}");
    }
    // the wrapped rows fit the panel.
    let style = CodeBlock::text_style(&theme);
    for row in view.shown_rows() {
        let w = engine::text::TextMeasurer::measure_styled(&row.text, &style, None).0;
        assert!(w <= layout.panel.w, "{width}: {:?}", row.text);
    }
    Ok(())
}

#[test]
fn a_phone_width_stacks_the_frame_the_strip_and_the_panel() -> Fallible {
    check(390.0, 844.0, Breakpoint::Compact)?;
    let view = open("a2ppa", 390.0, 844.0)?;
    let layout = view.layout();
    assert_eq!(layout.shell.sidebar_mode, SidebarMode::Drawer);
    assert!(
        layout.shell.menu_button.is_some(),
        "the list is behind the menu"
    );
    assert!(layout.frame.bottom() <= layout.strip.y && layout.strip.bottom() <= layout.panel.y);
    Ok(())
}

#[test]
fn a_desktop_width_puts_the_panel_beside_the_frame() -> Fallible {
    check(1440.0, 900.0, Breakpoint::Expanded)?;
    let view = open("a2ppa", 1440.0, 900.0)?;
    let layout = view.layout();
    assert_eq!(layout.shell.sidebar_mode, SidebarMode::Full);
    assert!(layout.frame.right() <= layout.panel.x);
    assert!(layout.frame.bottom() <= layout.strip.y);
    Ok(())
}

#[test]
fn a_resize_moves_every_part_with_the_window() -> Fallible {
    let mut view = open("a2ppa", 1440.0, 900.0)?;
    view.seek_at(105_000);
    let wide = view.fit().ok_or("a frame")?.frame();
    view.resize(390.0, 844.0);
    assert_eq!(view.size(), (390.0, 844.0));
    let narrow = view.fit().ok_or("a frame")?.frame();
    assert!(narrow.w < wide.w, "{narrow:?} {wide:?}");
    click_control(&mut view, &id("commit.submit"))?;
    assert_eq!(view.picked(), Some(&id("commit.submit")));
    Ok(())
}
