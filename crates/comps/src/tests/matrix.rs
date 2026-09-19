//! The matrix: every widget renders under every built-in theme, at a
//! phone width and a desktop width, without panicking and without
//! pushing a node outside a sane envelope. Nothing here checks pixels;
//! it checks that no widget assumes HOFF, a wide window or a mouse.

use engine::compositor::{Compositor, LayerId, SceneNode};
use engine::theme::{Intent, Theme};

use crate::core::Rect;
use crate::prelude::*;

const THEMES: [&str; 12] = [
    "hoff",
    "plev",
    "catppuccin",
    "dracula",
    "tokyo-night",
    "rose-pine",
    "nord",
    "gruvbox",
    "github-dark",
    "one-dark",
    "kanagawa",
    "moonlight",
];

/// Phone portrait and a desktop window.
const VIEWPORTS: [(f32, f32); 2] = [(390.0, 844.0), (1440.0, 900.0)];

fn themes() -> Vec<Theme> {
    let mut v: Vec<Theme> = THEMES.iter().map(|n| Theme::named(n).unwrap()).collect();
    v.push(Theme::light());
    v
}

/// Every node's bounding box must be finite (a NaN width means a
/// division by a zero the tokens should have prevented).
fn assert_finite(nodes: &[SceneNode], what: &str) {
    for n in nodes {
        let ok = match n {
            SceneNode::Rect { x, y, w, h, .. }
            | SceneNode::RoundedRect { x, y, w, h, .. }
            | SceneNode::GradientRect { x, y, w, h, .. }
            | SceneNode::Shadow { x, y, w, h, .. }
            | SceneNode::BackdropBlur { x, y, w, h, .. }
            | SceneNode::PushClip { x, y, w, h }
            | SceneNode::Image { x, y, w, h, .. } => {
                x.is_finite() && y.is_finite() && w.is_finite() && h.is_finite()
            }
            SceneNode::Text { x, y, .. } => x.is_finite() && y.is_finite(),
            _ => true,
        };
        assert!(ok, "{what}: non-finite node {n:?}");
    }
}

fn render_all(theme: &Theme, vw: f32, vh: f32) -> Vec<SceneNode> {
    let mut c = Compositor::new();
    c.begin_frame();
    let bp = theme.layout.breakpoint(vw);
    let gutter = theme.layout.gutter(bp);
    let content = Rect::new(gutter, gutter, vw - gutter * 2.0, vh - gutter * 2.0);
    let row = |i: f32| Rect::new(content.x, content.y + i * 60.0, content.w, 44.0);

    let mut b = Button::new("Commit").icon("check");
    b.set_focused(true);
    b.render(&mut c, row(0.0), theme);
    Button::new("Danger")
        .variant(ButtonVariant::Danger)
        .disabled(true)
        .render(&mut c, row(1.0), theme);
    IconButton::new("x").render(&mut c, row(2.0), theme);
    Chip::new("tag")
        .selected(true)
        .render(&mut c, row(3.0), theme);
    Badge::count("12").render(&mut c, row(4.0), theme);
    Badge::tag("MODIFIED").render(&mut c, row(4.0), theme);

    let mut f = TextField::new("name").with_text("brenner");
    f.focus();
    f.render(&mut c, row(5.0), theme);
    Checkbox::new(true)
        .label("on")
        .render(&mut c, row(6.0), theme);
    Switch::new(true).render(&mut c, row(7.0), theme);
    Slider::new(0.0, 100.0, 40.0).render(&mut c, row(8.0), theme);
    let mut s = Select::new(["a", "b", "c"], 1);
    s.render(&mut c, row(9.0), theme);
    s.handle_event(
        &WidgetEvent::MouseDown {
            x: row(9.0).x + 5.0,
            y: row(9.0).y + 5.0,
        },
        row(9.0),
        theme,
    );
    s.render_dropdown(&mut c, LayerId::DEFAULT, row(9.0), theme);

    Tabs::new(["one", "two", "three"]).render(&mut c, row(10.0), theme);
    NavLink::new("Cards")
        .icon("clipboard")
        .badge(Badge::count("3"))
        .active(true)
        .render(&mut c, row(11.0), theme);
    PanelHeader::new("Header")
        .blurb("blurb")
        .badge(Badge::tag("9"))
        .render(&mut c, row(12.0), theme);
    Breadcrumb::new(["a", "b", "c", "d"]).render(&mut c, row(13.0), theme);
    let mut sp = SplitPane::new(SplitDirection::Horizontal, 0.5, theme);
    sp.handle_event(
        &WidgetEvent::MouseMove {
            x: content.x + content.w / 2.0,
            y: row(14.0).y + 10.0,
        },
        row(14.0),
    );
    sp.render(&mut c, row(14.0), theme);

    for (i, variant) in [
        CardVariant::Stat {
            value: "38k".into(),
            label: "chunks".into(),
            delta: Some(("+2%".into(), true)),
        },
        CardVariant::Profile {
            name: "Ann".into(),
            username: "@ann".into(),
            bio: "a bio that is long enough to wrap on a phone".into(),
            action: "Follow".into(),
            online: true,
            avatar: None,
        },
        CardVariant::Media {
            title: "Clip".into(),
            caption: "cap".into(),
            badge: Some("4K".into()),
            image: None,
        },
        CardVariant::List {
            title: "Rows".into(),
            rows: vec![
                CardListRow::new("a", "1").active(true),
                CardListRow::new("b", "2").progress(0.4),
                CardListRow::new("", "3"),
            ],
        },
        CardVariant::Chart {
            value: "12".into(),
            label: "x".into(),
            groups: vec![(0.2, 0.5), (0.8, 0.3)],
            highlight: 1,
        },
        CardVariant::Cta {
            title: "T".into(),
            body: "body text body text body text".into(),
            button: "Go".into(),
        },
    ]
    .into_iter()
    .enumerate()
    {
        let card = Card::new(variant).width(content.w.min(theme.size.card_w));
        let (w, h) = card.preferred_size(theme);
        assert!(
            w <= content.w.max(theme.size.card_min_w) + 0.5,
            "card wider than its column"
        );
        card.render(
            &mut c,
            Rect::new(content.x, 900.0 + i as f32 * 320.0, w, h),
            theme,
        );
    }
    Avatar::new("ann")
        .online(true)
        .render(&mut c, row(15.0), theme);
    Panel::new().label("STATS").render(&mut c, row(16.0), theme);
    Stat::new("1.3", "GB")
        .unit("GB")
        .delta("+1", Intent::Constructive)
        .render(&mut c, row(17.0), theme);
    CodeBlock::new("cargo test")
        .caption("sh")
        .render(&mut c, row(18.0), theme);
    Skeleton::new(SkeletonShape::Lines).render(&mut c, row(19.0), theme);
    Separator::Horizontal.render(&mut c, row(20.0), theme);
    EmptyState::new("Nothing", "message")
        .icon("search")
        .cta(Button::new("Go"))
        .render(
            &mut c,
            Rect::new(content.x, 3000.0, content.w, 300.0),
            theme,
        );
    let mut list = VirtualList::new(24.0);
    list.set_item_count(100);
    list.render_with(
        &mut c,
        Rect::new(content.x, 3400.0, content.w, 200.0),
        theme,
        |_, _, _, _, _| {},
    );
    Tree::new(vec![TreeNode::branch(
        1,
        "root",
        vec![TreeNode::leaf(2, "leaf")],
    )])
    .render(
        &mut c,
        Rect::new(content.x, 3700.0, content.w, 200.0),
        theme,
    );
    let mut table = Table::new(
        vec![Column::new("a", theme), Column::new("b", theme).priority(0)],
        theme,
    );
    table.set_row_count(50);
    table.render_with(
        &mut c,
        Rect::new(content.x, 4000.0, content.w, 200.0),
        theme,
        |_, _, _, _| {},
    );

    Modal::new("Title", "Body body body", "OK", "Cancel")
        .intent(Intent::Destructive)
        .render(&mut c, LayerId::DEFAULT, theme, vw, vh);
    ContextMenu::new(vec![
        MenuEntry::item(1, "Open").icon("folder"),
        MenuEntry::Separator,
        MenuEntry::item(2, "Delete").intent(Intent::Destructive),
    ])
    .render(&mut c, LayerId::DEFAULT, theme, 10.0, 10.0);
    let mut toasts = ToastManager::new();
    toasts.push("hello", Intent::Informational, theme);
    for _ in 0..60 {
        toasts.tick(1.0 / 60.0);
    }
    toasts.render(&mut c, LayerId::DEFAULT, theme, vw, vh);
    let mut tip = Tooltip::new("tip");
    tip.set_hover(true, row(0.0));
    tip.tick(10.0);
    tip.render(&mut c, LayerId::DEFAULT, theme, vw, vh);
    ProgressBar::new(0.5).render(&mut c, row(21.0), theme);
    Spinner::new().render(&mut c, row(22.0), theme);

    let mut shell = AppShell::new(
        Sidebar::new(vec![
            NavLink::new("A").icon("house"),
            NavLink::new("B").icon("eye"),
        ])
        .brand("plev", "tag"),
        PanelHeader::new("Screen"),
        vw,
        vh,
    );
    shell.sidebar.drawer_open = true;
    shell.render(&mut c, LayerId::DEFAULT, LayerId::DEFAULT, theme);

    c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec()
}

#[test]
fn every_widget_renders_under_every_theme_at_phone_and_desktop_widths() {
    for theme in themes() {
        for (vw, vh) in VIEWPORTS {
            let nodes = render_all(&theme, vw, vh);
            assert!(
                nodes.len() > 100,
                "{vw}x{vh}: too few nodes ({})",
                nodes.len()
            );
            assert_finite(&nodes, &format!("{vw}x{vh}"));
        }
    }
}

#[test]
fn modal_and_toast_fit_a_phone_width() {
    let theme = Theme::hoff();
    let (vw, vh) = (390.0, 844.0);
    let modal = Modal::new("T", "B", "OK", "No");
    let d = modal.dialog_rect(&theme, vw, vh);
    let gutter = theme.layout.gutter(theme.layout.breakpoint(vw));
    assert!(d.x >= gutter - 0.5 && d.x + d.w <= vw - gutter + 0.5);
    let mut toasts = ToastManager::new();
    toasts.push("hi", Intent::Neutral, &theme);
    for _ in 0..120 {
        toasts.tick(1.0 / 60.0);
    }
    let r = toasts.visible_rects(&theme, vw, vh)[0];
    assert!(
        r.x >= 0.0 && r.x + r.w <= vw + 0.5,
        "toast inside the phone"
    );
    // And on a desktop the toast keeps the token width.
    let r = toasts.visible_rects(&theme, 1440.0, 900.0)[0];
    assert_eq!(r.w, theme.size.toast_w);
}
