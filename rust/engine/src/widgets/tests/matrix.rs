//! The matrix: every widget renders under every built-in theme, at a
//! phone width and a desktop width, without panicking and without
//! pushing a node outside a sane envelope. Nothing here checks pixels;
//! it checks that no widget assumes HOFF, a wide window or a mouse.

use crate::drawing::compositor::{Compositor, LayerId, SceneNode};
use crate::styling::theme::{Intent, Theme};
use accord::{Role, SemId, SnapshotBuilder};

use crate::widgets::form::field::Field;
use crate::widgets::overlay::popover::{Popover, PopoverTrigger, render_panel};
use crate::widgets::prelude::*;
use crate::widgets::selection::Toggle;
use crate::widgets::widget::Rect;

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
        .variant(ButtonVariant::Warning)
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
    // The kit's field, every part on: label and link, icon, eye, error.
    let mut field = Field::new("email")
        .label("Email")
        .link("Forgot password?")
        .icon("scan")
        .password(true)
        .with_text("brenner")
        .error("This email is already taken.");
    field.input.focus();
    field.render(&mut c, Rect::new(content.x, 4300.0, content.w, 0.0), theme);
    Checkbox::new(true)
        .label("on")
        .render(&mut c, row(6.0), theme);
    Switch::new(true).render(&mut c, row(7.0), theme);
    let mut toggle = Toggle::new("Bold").icon("eye").on(true);
    toggle.set_focused(true);
    toggle.render(&mut c, row(7.0), theme);
    Toggle::icon_only("eye", "Preview")
        .disabled(true)
        .render(&mut c, row(7.0), theme);
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
    let accordion = Accordion::new(AccordionMode::Single)
        .item("3D Objects", "Meshes and primitives on the canvas.")
        .item("Materials", "Surfaces and the maps that drive them.")
        .opened(0)
        .disabled_item(1);
    let h = accordion.height(content.w, theme);
    accordion.render(&mut c, Rect::new(content.x, 4300.0, content.w, h), theme);
    let mut collapsible = Collapsible::new("Explore").icon("house").current(true);
    collapsible.set_focused(true);
    collapsible.toggle();
    collapsible.wake(0.1);
    let h = collapsible.height(content.w, 80.0, theme);
    let at = Rect::new(content.x, 4700.0, content.w, h);
    collapsible.render(&mut c, at, 80.0, theme, |_, _| {});
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
        toasts.wake(1.0 / 60.0);
    }
    toasts.render(&mut c, LayerId::DEFAULT, theme, vw, vh);
    let mut tip = Tooltip::new("tip");
    tip.set_hover(true, row(0.0));
    tip.tick(10.0);
    tip.render(&mut c, LayerId::DEFAULT, theme, vw, vh);
    ProgressBar::new(0.5).render(&mut c, row(21.0), theme);
    Spinner::new().render(&mut c, row(22.0), theme);
    // task-104 front C (#156): a thread row and a quick comment
    {
        use crate::widgets::message::message::{Message, MessageKind};
        Message::new(
            "Luna Starfield",
            "2s",
            "a body long enough to wrap on a phone, at least once",
        )
        .divider(true)
        .render(&mut c, row(23.0), theme);
        Message::new("Randomdash", "1h", "clamped to three lines")
            .kind(MessageKind::Quick)
            .reaction("2")
            .render(&mut c, row(26.0), theme);
    }

    // task-104 front B: the popover's trigger, open, and its panel, placed
    // inside the content.
    let trigger = PopoverTrigger::new("100%");
    let anchor = Rect::new(content.right() - 88.0, row(23.0).y, 88.0, 40.0);
    trigger.render(&mut c, LayerId::DEFAULT, anchor, true, theme);
    let placed = Popover::new().placement(anchor, (204.0, 160.0), content, theme);
    render_panel(&mut c, LayerId::DEFAULT, placed.rect, theme);

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

/// The widgets with a SemId (phase a of task-59: text field, checkbox,
/// button) project their role and label, and bind the rect they draw
/// with, under every theme at both widths. The rect follows the layout
/// of each viewport; the id never changes.
#[test]
fn widgets_with_a_sem_id_project_and_bind_under_every_theme_at_phone_and_desktop_widths() {
    let id = |text: &str| SemId::parse(text).unwrap();
    let (name, agree, submit) = (id("form.name"), id("form.agree"), id("form.submit"));
    let mut rects = Vec::new();
    for (t, theme) in themes().into_iter().enumerate() {
        for (vw, vh) in VIEWPORTS {
            let gutter = theme.layout.gutter(theme.layout.breakpoint(vw));
            let row = |i: f32| Rect::new(gutter, gutter + i * 60.0, vw - gutter * 2.0, 44.0);
            let field = TextField::new("name")
                .with_text("brenner")
                .sem(name.clone());
            let check = Checkbox::new(true).label("on").sem(agree.clone());
            let button = Button::new("Commit").icon("check").sem(submit.clone());

            let mut c = Compositor::new();
            c.begin_frame();
            field.render(&mut c, row(0.0), &theme);
            check.render(&mut c, row(1.0), &theme);
            button.render(&mut c, row(2.0), &theme);

            let mut b = SnapshotBuilder::new();
            field.project(&mut b, &name, "name", None);
            check.project(&mut b, &agree, "agree", None);
            button.project(&mut b, &submit, None);
            let snap = b.finish().unwrap();

            let what = format!("theme {t} at {vw}x{vh}");
            for (at, (who, role, label)) in [
                (&name, Role::TextInput, "name"),
                (&agree, Role::Checkbox, "agree"),
                (&submit, Role::Button, "Commit"),
            ]
            .into_iter()
            .enumerate()
            {
                let node = snap.get(who).unwrap();
                assert_eq!((node.role, node.label.as_str()), (role, label), "{what}");
                let r = row(at as f32);
                let bound = c.binding(who).unwrap();
                assert_eq!(bound.rect, [r.x, r.y, r.w, r.h], "{what}: {who}");
                let (x, y) = r.center();
                assert_eq!(c.hit(x, y), Some(who), "{what}: {who}");
            }
            rects.push(c.binding(&name).unwrap().rect);
        }
    }
    assert!(
        rects.windows(2).any(|w| w[0] != w[1]),
        "the phone and the desktop lay the field out differently"
    );
}

/// What the phase b matrix checks for one id: its node (role and a
/// non-empty label), its binding too, or also that `hit` at the center of
/// that binding answers it.
#[derive(Clone, Copy, PartialEq)]
enum Probe {
    Node,
    Bound,
    Hit,
}

/// One widget's frame and the ids to probe in it.
type Frame = (Compositor, Vec<(SemId, Role, Probe)>);

/// Phase b of task-59: every other interactive widget, with a SemId,
/// projects its role and a label and binds the rects it hit-tests with,
/// under every theme at both widths, and the whole projection validates.
/// Each widget draws into a frame of its own, so an overlay (the modal,
/// the open select) never covers another widget's hit. A container is
/// checked for its binding; its children answer the hit.
#[test]
fn every_interactive_widget_projects_and_binds_under_every_theme_at_phone_and_desktop_widths() {
    let id = |text: &str| SemId::parse(text).unwrap();
    let frame = |overlay: bool| {
        let mut c = Compositor::new();
        let layer = if overlay {
            c.create_layer(100)
        } else {
            LayerId::DEFAULT
        };
        c.begin_frame();
        (c, layer)
    };
    let mut tab_rects = Vec::new();
    for (t, theme) in themes().into_iter().enumerate() {
        for (vw, vh) in VIEWPORTS {
            let what = format!("theme {t} at {vw}x{vh}");
            let gutter = theme.layout.gutter(theme.layout.breakpoint(vw));
            let row = |i: f32| Rect::new(gutter, gutter + i * 60.0, vw - gutter * 2.0, 44.0);
            let block = |i: f32| Rect::new(gutter, gutter + i * 60.0, vw - gutter * 2.0, 200.0);
            let label = |i: usize| format!("row {i}");
            let mut b = SnapshotBuilder::new();
            let mut frames: Vec<Frame> = Vec::new();

            let chip = Chip::new("tag").selectable(true).sem(id("m.chip"));
            let (mut c, _) = frame(false);
            chip.render(&mut c, row(0.0), &theme);
            chip.project(&mut b, &id("m.chip"), None);
            frames.push((c, vec![(id("m.chip"), Role::Toggle, Probe::Hit)]));

            let icon = IconButton::new("x").sem(id("m.icon"));
            let (mut c, _) = frame(false);
            icon.render(&mut c, row(1.0), &theme);
            icon.project(&mut b, &id("m.icon"), "close", None);
            frames.push((c, vec![(id("m.icon"), Role::Button, Probe::Hit)]));

            let submit = SubmitButton::new().active(true).sem(id("m.submit"));
            let (mut c, _) = frame(false);
            submit.render(&mut c, row(2.0), &theme);
            submit.project(&mut b, &id("m.submit"), "send", None);
            frames.push((c, vec![(id("m.submit"), Role::Button, Probe::Hit)]));

            let switch = Switch::new(true).sem(id("m.switch"));
            let (mut c, _) = frame(false);
            switch.render(&mut c, row(3.0), &theme);
            switch.project(&mut b, &id("m.switch"), "wifi", None);
            frames.push((c, vec![(id("m.switch"), Role::Toggle, Probe::Hit)]));

            let toggle = Toggle::icon_only("eye", "Preview").sem(id("m.toggle"));
            let (mut c, _) = frame(false);
            toggle.render(&mut c, row(3.0), &theme);
            toggle.project(&mut b, &id("m.toggle"), None);
            frames.push((c, vec![(id("m.toggle"), Role::Toggle, Probe::Hit)]));

            let slider = Slider::new(0.0, 100.0, 40.0).step(5.0).sem(id("m.slider"));
            let (mut c, _) = frame(false);
            slider.render(&mut c, row(4.0), &theme);
            slider.project(&mut b, &id("m.slider"), "volume", None);
            frames.push((c, vec![(id("m.slider"), Role::Slider, Probe::Hit)]));

            let mut select = Select::new(["a", "b", "c"], 1).sem(id("m.select"));
            select.open();
            let (mut c, overlay) = frame(true);
            select.render(&mut c, row(5.0), &theme);
            select.render_dropdown(&mut c, overlay, row(5.0), &theme);
            select.project(&mut b, &id("m.select"), "size", None);
            frames.push((
                c,
                vec![
                    (id("m.select"), Role::Select, Probe::Hit),
                    (id("m.select[0]"), Role::Option, Probe::Hit),
                    (id("m.select[2]"), Role::Option, Probe::Hit),
                ],
            ));

            let tabs = Tabs::new(["one", "two", "three"]).sem(id("m.tabs"));
            let (mut c, _) = frame(false);
            tabs.render(&mut c, row(6.0), &theme);
            tabs.project(&mut b, &id("m.tabs"), "view", None);
            tab_rects.push(c.binding(&id("m.tabs[1]")).map(|b| b.rect));
            frames.push((
                c,
                vec![
                    (id("m.tabs"), Role::TabList, Probe::Bound),
                    (id("m.tabs[1]"), Role::Tab, Probe::Hit),
                ],
            ));

            let link = NavLink::new("Cards")
                .icon("clipboard")
                .badge(Badge::count("3"))
                .active(true)
                .sem(id("m.link"));
            let (mut c, _) = frame(false);
            link.render(&mut c, row(7.0), &theme);
            link.project(&mut b, &id("m.link"), None);
            frames.push((c, vec![(id("m.link"), Role::Link, Probe::Hit)]));

            let path = Breadcrumb::new(["a", "b", "c", "d"]).sem(id("m.path"));
            let (mut c, _) = frame(false);
            path.render(&mut c, row(8.0), &theme);
            path.project(&mut b, &id("m.path"), "path", None);
            frames.push((
                c,
                vec![
                    (id("m.path"), Role::Group, Probe::Bound),
                    (id("m.path[0]"), Role::Link, Probe::Hit),
                    (id("m.path[3]"), Role::Label, Probe::Hit),
                ],
            ));

            let split = SplitPane::new(SplitDirection::Horizontal, 0.5, &theme).sem(id("m.split"));
            let (mut c, _) = frame(false);
            split.render(&mut c, row(9.0), &theme);
            split.project(&mut b, &id("m.split"), "panes", None);
            frames.push((c, vec![(id("m.split"), Role::Group, Probe::Hit)]));

            let mut list = VirtualList::new(24.0).sem(id("m.list"));
            list.set_item_count(100);
            let (mut c, _) = frame(false);
            list.render_with(&mut c, block(10.0), &theme, |_, _, _, _, _| {});
            list.project(&mut b, &id("m.list"), "files", None, label);
            frames.push((
                c,
                vec![
                    (id("m.list"), Role::List, Probe::Bound),
                    (id("m.list[0]"), Role::ListItem, Probe::Hit),
                ],
            ));

            let mut table = Table::new(
                vec![
                    Column::new("a", &theme),
                    Column::new("b", &theme).priority(0),
                ],
                &theme,
            )
            .sem(id("m.table"));
            table.set_row_count(50);
            let (mut c, _) = frame(false);
            table.render_with(&mut c, block(14.0), &theme, |_, _, _, _| {});
            table.project(&mut b, &id("m.table"), "grid", None, label);
            frames.push((
                c,
                vec![
                    (id("m.table"), Role::Table, Probe::Bound),
                    (id("m.table[0]"), Role::Row, Probe::Hit),
                ],
            ));

            let tree = Tree::new(vec![
                TreeNode::branch(1, "root", vec![TreeNode::leaf(2, "leaf")]).expanded(true),
            ])
            .sem(id("m.tree"));
            let (mut c, _) = frame(false);
            tree.render(&mut c, block(18.0), &theme);
            tree.project(&mut b, &id("m.tree"), "fs", None);
            frames.push((
                c,
                vec![
                    (id("m.tree"), Role::Tree, Probe::Bound),
                    (id("m.tree[1]"), Role::TreeItem, Probe::Hit),
                    (id("m.tree[2]"), Role::TreeItem, Probe::Hit),
                ],
            ));

            let menu = ContextMenu::new(vec![
                MenuEntry::item(1, "Open").icon("folder"),
                MenuEntry::Separator,
                MenuEntry::item(2, "Delete").intent(Intent::Destructive),
            ])
            .sem(id("m.menu"));
            let (mut c, overlay) = frame(true);
            menu.render(&mut c, overlay, &theme, gutter, gutter);
            menu.project(&mut b, &id("m.menu"), "actions", None);
            frames.push((
                c,
                vec![
                    (id("m.menu"), Role::Menu, Probe::Bound),
                    (id("m.menu[1]"), Role::MenuItem, Probe::Hit),
                    (id("m.menu[2]"), Role::MenuItem, Probe::Hit),
                ],
            ));

            let modal = Modal::new("Title", "Body body body", "OK", "Cancel")
                .intent(Intent::Destructive)
                .sem(id("m.modal"));
            let (mut c, overlay) = frame(true);
            modal.render(&mut c, overlay, &theme, vw, vh);
            modal.project(&mut b, &id("m.modal"), None);
            frames.push((
                c,
                vec![
                    (id("m.modal"), Role::Dialog, Probe::Bound),
                    (id("m.modal[confirm]"), Role::Button, Probe::Hit),
                    (id("m.modal[cancel]"), Role::Button, Probe::Hit),
                ],
            ));

            let mut toasts = ToastManager::new().sem(id("m.toasts"));
            toasts.push("hello", Intent::Informational, &theme);
            for _ in 0..60 {
                toasts.wake(1.0 / 60.0);
            }
            let (mut c, overlay) = frame(true);
            toasts.render(&mut c, overlay, &theme, vw, vh);
            toasts.project(&mut b, &id("m.toasts"), "notifications", None);
            frames.push((
                c,
                vec![
                    (id("m.toasts"), Role::Region, Probe::Node),
                    (id("m.toasts[0]"), Role::Status, Probe::Hit),
                ],
            ));

            // task-104 front C (#156): a message row and its like chip
            let message = crate::widgets::message::message::Message::new("Luna", "2s", "body")
                .kind(crate::widgets::message::message::MessageKind::Quick)
                .reaction("2")
                .sem(id("m.message"));
            let (mut c, _) = frame(false);
            message.render(&mut c, block(22.0), &theme);
            message.project(&mut b, &id("m.message"), None);
            frames.push((
                c,
                vec![
                    (id("m.message"), Role::ListItem, Probe::Bound),
                    (id("m.message.reaction"), Role::Button, Probe::Hit),
                ],
            ));

            let snap = b
                .finish()
                .unwrap_or_else(|e| panic!("{what}: the projection does not validate: {e:?}"));
            for (c, probes) in &frames {
                for (who, role, probe) in probes {
                    let node = snap
                        .get(who)
                        .unwrap_or_else(|| panic!("{what}: {who} is not projected"));
                    assert_eq!(node.role, *role, "{what}: {who}");
                    assert!(!node.label.trim().is_empty(), "{what}: {who} has no label");
                    if *probe == Probe::Node {
                        continue;
                    }
                    let bound = c
                        .binding(who)
                        .unwrap_or_else(|| panic!("{what}: {who} is not bound"));
                    let [x, y, w, h] = bound.rect;
                    assert!(
                        bound.rect.iter().all(|v| v.is_finite()) && w > 0.0 && h > 0.0,
                        "{what}: {who} bound to {:?}",
                        bound.rect
                    );
                    if *probe == Probe::Hit {
                        assert_eq!(c.hit(x + w / 2.0, y + h / 2.0), Some(who), "{what}");
                    }
                }
            }
        }
    }
    assert!(
        tab_rects.windows(2).any(|w| w[0] != w[1]),
        "the phone and the desktop lay the tabs out differently"
    );
}

/// task-81: a select with more options than `size.dropdown_max_h` fits
/// reaches its last option under every theme at both widths. Scrolled to
/// the end by the wheel, the first option is no longer bound, the last one
/// is bound inside the panel, `hit` at its center answers it, and a click
/// there picks it.
#[test]
fn a_long_select_reaches_its_last_option_under_every_theme_at_phone_and_desktop_widths() {
    let long = SemId::parse("m.long").unwrap();
    let last = long.key("19");
    for (t, theme) in themes().into_iter().enumerate() {
        for (vw, _) in VIEWPORTS {
            let what = format!("theme {t} at {vw}");
            let gutter = theme.layout.gutter(theme.layout.breakpoint(vw));
            let control = Rect::new(gutter, gutter, vw - gutter * 2.0, 44.0);
            let mut select =
                Select::new((0..20).map(|i| format!("option {i}")), 0).sem(long.clone());
            let (x, y) = control.center();
            select.handle_event(&WidgetEvent::MouseDown { x, y }, control, &theme);
            let panel = select.dropdown_rect(control, &theme);
            let (x, y) = panel.center();
            let end = WidgetEvent::Scroll {
                x,
                y,
                delta: 100_000.0,
            };
            assert!(select.handle_event(&end, control, &theme).changed, "{what}");

            let mut c = Compositor::new();
            let overlay = c.create_layer(100);
            c.begin_frame();
            select.render(&mut c, control, &theme);
            select.render_dropdown(&mut c, overlay, control, &theme);
            assert_eq!(c.binding(&long.key("0")), None, "{what}: scrolled out");
            let [bx, by, bw, bh] = c
                .binding(&last)
                .unwrap_or_else(|| panic!("{what}: the last option is not bound"))
                .rect;
            assert!(
                by >= panel.y && by + bh <= panel.y + panel.h,
                "{what}: the last row inside the panel"
            );
            let (x, y) = (bx + bw / 2.0, by + bh / 2.0);
            assert_eq!(c.hit(x, y), Some(&last), "{what}");
            let pick = select.handle_event(&WidgetEvent::MouseDown { x, y }, control, &theme);
            assert!(pick.clicked, "{what}");
            assert_eq!(select.selected, 19, "{what}");
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
        toasts.wake(1.0 / 60.0);
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
