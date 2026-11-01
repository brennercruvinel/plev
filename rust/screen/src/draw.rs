//! each component on its comps widget: the size it takes in the layout
//! and how it draws, from the node, the widget and the snapshot shown.
//!
//! every visual value is a token: the widgets read the theme, and what the
//! host draws itself (a container's fill, a text run, a dialog's title)
//! reads the field the IR token names. a prop a widget does not take (a
//! typography on a text field, a color on a control) is its default. a
//! stat reads its `typography` as the size of its value (`StatSize`),
//! and a badge its `background` as the intent it is filled with (the
//! filled shape instead of the glass tag): the tokens `emphasize` moves.
//! the text the host draws itself for a slider (its label and value), a
//! table (its rows) and an image (its label) is in the node's `color`
//! when it has one, else in the host's own ink for it (task-70 phase 4:
//! the token "more contrast" moves).
//!
//! an image draws the picture the app holds for its bound `source`
//! (`Frames`), fitted inside its shell and never scaled up; without one,
//! the shell with its label. its size is the block's, or, with a
//! `control_size`, that control height at the frame's usual aspect (a
//! thumbnail in a row).

use accord::{Snapshot, Value};
use engine::compose::layout::TextMeasureSpec;
use engine::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::drawing::gpu::image::ImageHandle;
use engine::drawing::text::{TextMeasurer, TextStyle};
use engine::styling::theme::TypeRole;
use engine::styling::theme::{ControlSize, Elevation, Theme};
use engine::widgets::Rect;
use engine::widgets::buttons::Badge;
use engine::widgets::charts::draw as chart;
use engine::widgets::content::{EmptyState, Panel, Separator, Stat, Table};
use engine::widgets::feedback::ProgressBar;
use engine::widgets::form::{Slider, TextField};
use engine::widgets::nav::Tabs;
use engine::widgets::recipe::{rounded_rect, shadow_stack};
use plevir::{Axis, ChartKind, Component, SpacingToken};

use crate::code;
use crate::expand::Node;
use crate::host::Frames;
use crate::layout::Leaf;
use crate::token;
use crate::widget::{Widget, dialog_buttons, list_of, slider_h, slider_track, text_of};

/// the aspect of an image sized by a control height, width over height:
/// the frame of a video, the common one, before or without a picture.
/// pure geometry, not a token: it is the shape of the content, not of
/// the design system.
const FRAME_ASPECT: f32 = 16.0 / 9.0;

/// the picture an image shows now: the frame the app holds for the text
/// its `source` is bound to. none for any other node, for an unbound
/// source, or while the app has no frame for it.
pub(crate) fn picture(node: &Node, shown: &Snapshot, frames: &dyn Frames) -> Option<ImageHandle> {
    if node.component != Some(Component::Image) {
        return None;
    }
    let source = node.value("source", shown).map(text_of)?;
    frames.frame(&source)
}

/// the frame ref an image shows, when its `source` is bound.
pub(crate) fn source(node: &Node, shown: &Snapshot) -> Option<String> {
    if node.component != Some(Component::Image) {
        return None;
    }
    node.value("source", shown).map(text_of)
}

/// the text a text or a badge shows: its bound `text`, or its label.
pub(crate) fn shown_text(node: &Node, shown: &Snapshot) -> String {
    node.value("text", shown)
        .map(text_of)
        .or_else(|| node.label.clone())
        .unwrap_or_default()
}

/// whether a dialog is closed now: bound `open` is false. an unbound
/// dialog is always open.
pub(crate) fn closed(node: &Node, shown: &Snapshot) -> bool {
    node.component == Some(Component::Dialog)
        && matches!(node.value("open", shown), Some(Value::Bool(false)))
}

fn text_style(node: &Node, theme: &Theme) -> TextStyle {
    node.props.typography.map_or_else(
        || theme.typography.role(TypeRole::BodyLg),
        |t| token::typography(t, theme),
    )
}

fn stat(node: &Node, shown: &Snapshot) -> Stat {
    let value = node.value("value", shown).map(text_of).unwrap_or_default();
    let stat = Stat::new(value, node.label.clone().unwrap_or_default());
    match node.props.typography {
        Some(t) => stat.size(token::stat_size(t)),
        None => stat,
    }
}

/// a badge: the glass tag, or filled with the intent its `background`
/// names.
fn badge(node: &Node, shown: &Snapshot) -> Badge {
    let text = shown_text(node, shown);
    match node.props.background.and_then(token::badge_intent) {
        Some(intent) => Badge::count(text).intent(intent),
        None => Badge::tag(text),
    }
}

/// a dialog's padding, its title's style and the band above and below
/// its children.
fn dialog_metrics(node: &Node, theme: &Theme) -> (f32, TextStyle, f32) {
    let pad = token::spacing(node.props.padding.unwrap_or(SpacingToken::Lg), theme);
    (
        pad,
        theme.typography.role(TypeRole::Title),
        theme.spacing.sm,
    )
}

/// the height a chart, an image or an empty state takes when nothing else
/// says: half the narrowest card, the gallery's placeholder height.
fn block_h(theme: &Theme) -> f32 {
    theme.size.card_min_w / 2.0
}

/// what `node` takes in its parent (laid out along `axis`).
pub(crate) fn leaf(
    node: &Node,
    widget: Option<&Widget>,
    shown: &Snapshot,
    axis: Axis,
    theme: &Theme,
) -> Leaf {
    let fill = |height: f32| Leaf::Fill {
        height,
        min_w: theme.size.field_min_w,
    };
    let fixed = |(w, h): (f32, f32)| Leaf::Fixed(w, h);
    let no_inset = Leaf::Container { inset: [0.0; 4] };
    match node.component {
        None | Some(Component::Card | Component::List) => no_inset,
        Some(Component::Dialog) if closed(node, shown) => Leaf::Hidden,
        Some(Component::Dialog) => {
            let (pad, title, gap) = dialog_metrics(node, theme);
            let buttons = match widget {
                Some(Widget::Dialog(d)) => d.confirm.preferred_size(theme).1,
                _ => 0.0,
            };
            let own = if node.props.padding.is_some() {
                0.0
            } else {
                pad
            };
            Leaf::Container {
                inset: [own + title.line_height + gap, own, own + buttons + gap, own],
            }
        }
        Some(Component::Text) => Leaf::Text(TextMeasureSpec {
            content: shown_text(node, shown),
            style: text_style(node, theme),
            max_width: None,
        }),
        Some(Component::Badge) => fixed(badge(node, shown).preferred_size(theme)),
        Some(Component::Stat) => fixed(stat(node, shown).preferred_size(theme)),
        Some(Component::TextField) => fill(TextField::height(theme)),
        Some(Component::Slider) => fill(slider_h(theme)),
        Some(Component::Select) => fill(theme.control.height(ControlSize::Md)),
        Some(Component::Tabs) => fill(Tabs::height(theme)),
        Some(Component::Table) => {
            let rows = list_of(node.value("rows", shown)).len() as f32;
            fill(Table::header_h(theme) + rows * Table::row_h(theme))
        }
        Some(Component::Chart(_) | Component::EmptyState) => fill(block_h(theme)),
        Some(Component::Code) => fill(match widget {
            Some(Widget::Code(c)) => c.height(theme),
            _ => code::block(node, shown).height_for(0.0, theme),
        }),
        Some(Component::Image) => match node.props.control_size {
            Some(size) => {
                let h = theme.control.height(token::control(size));
                Leaf::Fixed((h * FRAME_ASPECT).round(), h)
            }
            None => fill(block_h(theme)),
        },
        Some(Component::Progress) => fill(theme.control.progress_track),
        Some(Component::Separator) => match axis {
            Axis::Horizontal => Leaf::Fixed(
                Separator::thickness(theme),
                theme.control.height(ControlSize::Xs),
            ),
            Axis::Vertical => Leaf::Fill {
                height: Separator::thickness(theme),
                min_w: 0.0,
            },
        },
        Some(Component::Switch) => {
            let [w, h] = theme.control.switch_size;
            Leaf::Fixed(w, h)
        }
        Some(Component::Checkbox) => {
            let style = theme.typography.role(TypeRole::Body);
            let label = node.label.as_deref().unwrap_or_default();
            let (tw, _) = TextMeasurer::measure_styled(label, &style, None);
            let side = theme.control.box_size;
            Leaf::Fixed(
                (side + theme.control.inline_gap + tw).ceil(),
                side.max(style.line_height),
            )
        }
        Some(Component::Button | Component::IconButton | Component::Chip) => match widget {
            Some(Widget::Button(b)) => {
                // the step of its ladder the props ask for, before the
                // first frame sets it on the widget
                let mut b = b.clone();
                if let Some(size) = node.props.control_size {
                    b.size = token::button_size(size, theme);
                }
                fixed(b.preferred_size(theme))
            }
            Some(Widget::Submit(b)) => fixed(b.preferred_size(theme)),
            Some(Widget::IconButton(b)) => fixed(b.preferred_size(theme)),
            Some(Widget::Chip(c)) => fixed(c.preferred_size(theme)),
            _ => Leaf::Fixed(0.0, 0.0),
        },
    }
}

/// a container's own drawing: its shadow stack and its fill, by token.
fn container(c: &mut Compositor, node: &Node, rect: Rect, theme: &Theme) {
    let radius = node.props.shape.map_or(0.0, |s| token::shape(s, theme));
    if let Some(e) = node.props.elevation {
        shadow_stack(
            c,
            LayerId::DEFAULT,
            rect,
            radius,
            token::elevation(e),
            theme,
        );
    }
    if let Some(bg) = node.props.background {
        c.push(rounded_rect(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            radius,
            token::color(bg, theme),
        ));
    }
}

/// a text run: measured and drawn with one style, wrapped to its rect.
fn text(c: &mut Compositor, s: &str, style: &TextStyle, rect: Rect, color: [f32; 4]) {
    if s.is_empty() {
        return;
    }
    c.push(SceneNode::Text {
        key: TextNodeKey::from_style(s, style, Some(rect.w.max(1.0))),
        x: rect.x,
        y: rect.y,
        color,
    });
}

/// draws `node` at `rect`, and binds its id to that rect when its widget
/// does not bind itself. `picture` is the frame an image shows, when the
/// app holds one (`picture`).
pub(crate) fn draw(
    c: &mut Compositor,
    node: &Node,
    widget: &mut Widget,
    rect: Rect,
    shown: &Snapshot,
    theme: &Theme,
    picture: Option<ImageHandle>,
) {
    let binds_itself = matches!(
        widget,
        Widget::Button(_)
            | Widget::Submit(_)
            | Widget::IconButton(_)
            | Widget::TextField(_)
            | Widget::Checkbox(_)
            | Widget::Switch(_)
            | Widget::Slider(_)
            | Widget::Chip(_)
    );
    if !binds_itself && plevir::role_of(node.component, &node.bind).is_some() {
        c.bind(&node.id, [rect.x, rect.y, rect.w, rect.h]);
    }
    match (node.component, widget) {
        (None | Some(Component::List), _) => container(c, node, rect, theme),
        (Some(Component::Card), _) => {
            let elevation = node
                .props
                .elevation
                .map_or(Elevation::Base, token::elevation);
            Panel::new().elevation(elevation).render(c, rect, theme);
        }
        (Some(Component::Dialog), Widget::Dialog(d)) => {
            let elevation = node
                .props
                .elevation
                .map_or(Elevation::Overlay, token::elevation);
            Panel::new().elevation(elevation).render(c, rect, theme);
            let (pad, style, _) = dialog_metrics(node, theme);
            let label = node.label.as_deref().unwrap_or_default();
            let band = Rect::new(
                rect.x + pad,
                rect.y + pad,
                rect.w - pad * 2.0,
                style.line_height,
            );
            text(c, label, &style, band, theme.colors.text.0);
            let (cancel, confirm) = dialog_buttons(d, rect, theme);
            d.cancel.render(c, cancel, theme);
            d.confirm.render(c, confirm, theme);
        }
        (Some(Component::Text), _) => {
            let color = node
                .props
                .color
                .map_or(theme.colors.text.0, |t| token::color(t, theme));
            text(
                c,
                &shown_text(node, shown),
                &text_style(node, theme),
                rect,
                color,
            );
        }
        (Some(Component::Badge), _) => badge(node, shown).render(c, rect, theme),
        (Some(Component::Stat), _) => stat(node, shown).render(c, rect, theme),
        (Some(Component::Separator), _) => {
            let dir = if rect.w < rect.h {
                Separator::Vertical
            } else {
                Separator::Horizontal
            };
            dir.render(c, rect, theme);
        }
        (Some(Component::EmptyState), _) => {
            let message = node.description.clone().unwrap_or_default();
            EmptyState::new(node.label.clone().unwrap_or_default(), message).render(c, rect, theme);
        }
        (Some(Component::Progress), _) => progress(node, shown).render(c, rect, theme),
        (Some(Component::Image), _) => image(c, node, rect, theme, picture),
        (Some(Component::Chart(kind)), _) => {
            let data = list_of(node.value("data", shown));
            draw_chart(c, kind, &data, rect, theme);
        }
        (_, Widget::Button(b)) => b.render(c, rect, theme),
        (_, Widget::Submit(b)) => b.render(c, rect, theme),
        (_, Widget::IconButton(b)) => b.render(c, rect, theme),
        (_, Widget::TextField(f)) => f.render(c, rect, theme),
        (_, Widget::Checkbox(b)) => b.render(c, rect, theme),
        (_, Widget::Switch(s)) => s.render(c, rect, theme),
        (_, Widget::Slider(s)) => slider(c, node, s, rect, theme),
        (_, Widget::Select(s)) => s.render(c, rect, theme),
        (_, Widget::Tabs(t)) => t.render(c, rect, theme),
        (_, Widget::Chip(b)) => b.render(c, rect, theme),
        (_, Widget::Table(Some(t))) => {
            let rows = list_of(node.value("rows", shown));
            let style = theme.typography.role(TypeRole::Body);
            let ink = ink(node, theme).unwrap_or(theme.colors.text_mid.0);
            // a table bounded in height (`size.max`, task-96) holds more
            // rows than its rect shows, and the list under it draws its
            // overscan past its bounds: its rows are clipped to it then. a
            // table as tall as its rows draws as it always did.
            let full = Table::header_h(theme) + rows.len() as f32 * Table::row_h(theme);
            let clipped = full > rect.h + 0.5;
            if clipped {
                c.push(SceneNode::PushClip {
                    x: rect.x,
                    y: rect.y,
                    w: rect.w,
                    h: rect.h,
                });
            }
            t.render_with(c, rect, theme, |c, row, cell, _| {
                if let Some(s) = rows.get(row) {
                    let y = cell.y + TextMeasurer::vertical_center(&style, cell.h);
                    text(c, s, &style, Rect::new(cell.x, y, cell.w, cell.h), ink);
                }
            });
            if clipped {
                c.push(SceneNode::PopClip);
            }
        }
        (_, Widget::Code(v)) => v.draw(c, rect, theme),
        (_, Widget::Table(None) | Widget::Dialog(_) | Widget::Static) => {}
    }
}

/// the ink the node's `color` names, when it has one.
fn ink(node: &Node, theme: &Theme) -> Option<[f32; 4]> {
    node.props.color.map(|t| token::color(t, theme))
}

/// a slider as the fixed screens draw one: its label on the left and its
/// value on the right, on a line above the track, and the track under
/// them. the value is the widget's, so a drag shows where it is going.
fn slider(c: &mut Compositor, node: &Node, s: &Slider, rect: Rect, theme: &Theme) {
    let label_style = theme.typography.run(TypeRole::Caption.strong());
    let value_style = theme.typography.run(TypeRole::Caption.regular());
    let own = ink(node, theme);
    let value = text_of(&Value::Num(f64::from(s.value())));
    let (vw, _) = TextMeasurer::measure_styled(&value, &value_style, None);
    let vw = vw.ceil();
    text(
        c,
        &value,
        &value_style,
        Rect::new(rect.x + rect.w - vw, rect.y, vw, value_style.line_height),
        own.unwrap_or(theme.colors.text.0),
    );
    let label = node.label.as_deref().unwrap_or_default();
    let room = (rect.w - vw - theme.spacing.sm).max(1.0);
    text(
        c,
        label,
        &label_style,
        Rect::new(rect.x, rect.y, room, label_style.line_height),
        own.unwrap_or(theme.colors.text_mid.0),
    );
    s.render(c, slider_track(rect, theme), theme);
}

/// a progress bar over its value: a fraction of the range the node
/// holding the value declares, or the value itself clamped to 0..=1.
fn progress(node: &Node, shown: &Snapshot) -> ProgressBar {
    let value = match node.value("value", shown) {
        Some(Value::Num(n)) => *n,
        Some(Value::Int(i)) => *i as f64,
        _ => 0.0,
    };
    let range = node
        .bind
        .state
        .get("value")
        .and_then(|at| shown.get(&at.node))
        .and_then(|n| n.constraints.range);
    let fraction = match range {
        Some((lo, hi)) if hi > lo => (value - lo) / (hi - lo),
        _ => value,
    };
    ProgressBar::new(fraction as f32)
}

/// an image: its shell, and inside it the picture the app holds for its
/// source, fitted and centered, never scaled up; without one, its label
/// in the middle (the surface cannot decode: the IR names a source, the
/// app holds the frames).
fn image(c: &mut Compositor, node: &Node, rect: Rect, theme: &Theme, picture: Option<ImageHandle>) {
    let radius = node
        .props
        .shape
        .map_or(theme.shape.card, |s| token::shape(s, theme));
    c.push(rounded_rect(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        radius,
        theme.glass.surface.0,
    ));
    if let Some(handle) = picture {
        let (iw, ih) = (handle.width as f32, handle.height as f32);
        if iw > 0.0 && ih > 0.0 {
            let scale = (rect.w / iw).min(rect.h / ih).min(1.0);
            let (w, h) = (iw * scale, ih * scale);
            c.draw_image(
                rect.x + (rect.w - w) / 2.0,
                rect.y + (rect.h - h) / 2.0,
                w,
                h,
                handle,
                radius,
            );
            return;
        }
    }
    let style = theme.typography.run(TypeRole::Caption.regular());
    let label = node.label.as_deref().unwrap_or_default();
    let (w, h) = TextMeasurer::measure_styled(label, &style, Some(rect.w));
    let at = Rect::new(
        rect.x + (rect.w - w) / 2.0,
        rect.y + (rect.h - h) / 2.0,
        w,
        h,
    );
    let color = ink(node, theme).unwrap_or(theme.colors.text_dim.0);
    text(c, label, &style, at, color);
}

/// a chart over the numbers of its `data` list; a donut item may carry
/// its name before a `:` (`"rust: 42"`).
fn draw_chart(c: &mut Compositor, kind: ChartKind, data: &[String], rect: Rect, theme: &Theme) {
    let parsed: Vec<(&str, f32)> = data
        .iter()
        .filter_map(|item| {
            let (name, value) = item.rsplit_once(':').unwrap_or(("", item.as_str()));
            value.trim().parse::<f32>().ok().map(|v| (name.trim(), v))
        })
        .collect();
    let values: Vec<f32> = parsed.iter().map(|&(_, v)| v).collect();
    match kind {
        ChartKind::Line => chart::line(c, &values, rect, theme, 1.0),
        ChartKind::Bar => chart::bars(c, &values, rect, theme, 1.0),
        ChartKind::Area => chart::area(c, &values, &vec![0.0; values.len()], rect, theme, 1.0),
        ChartKind::Donut => chart::donut(c, &parsed, rect, theme, 1.0),
    }
}

#[cfg(test)]
mod tests {
    use accord::{Role, Snapshot, SnapshotBuilder};
    use engine::drawing::compositor::{Compositor, LayerId, SceneNode};
    use engine::styling::theme::Theme;
    use engine::widgets::Rect;
    use engine::widgets::content::Table;
    use plevir::{Capabilities, GenerationPolicy, verify_json};

    use crate::surface::Surface;

    /// a log of `rows` lines.
    fn app(rows: usize) -> Snapshot {
        let mut b = SnapshotBuilder::new();
        let lines: Vec<String> = (0..rows).map(|i| format!("line {i}")).collect();
        b.node("log", Role::Region, "log").state("lines", lines);
        b.finish().unwrap()
    }

    /// the log's lines in a table bounded by the one height token.
    const IR: &str = r#"{"plev_ir": 1, "surface": {"id": "s", "title": "t", "root": {
        "sem": "s.root",
        "children": [{
            "sem": "s.lines", "component": "table", "label": "lines",
            "size": {"max": "dropdown_max_h"},
            "bind": {"state": {"rows": "log:lines"}}
        }]
    }}}"#;

    /// the table's rect, and every clip the frame pushed, for `rows`.
    fn drawn(rows: usize) -> ([f32; 4], Vec<[f32; 4]>) {
        let app = app(rows);
        let caps = Capabilities::from_snapshot(&app);
        let ir = verify_json(IR, &caps, &GenerationPolicy::default()).unwrap();
        let mut surface = Surface::materialize(ir, &app);
        let mut c = Compositor::new();
        surface.render(&mut c, Rect::new(0.0, 0.0, 400.0, 2000.0), &Theme::hoff());
        let table = c.binding(&"s.lines".parse().unwrap()).unwrap().rect;
        let clips = c
            .layer(LayerId::DEFAULT)
            .unwrap()
            .nodes()
            .iter()
            .filter_map(|n| match *n {
                SceneNode::PushClip { x, y, w, h } => Some([x, y, w, h]),
                _ => None,
            })
            .collect();
        (table, clips)
    }

    #[test]
    fn a_table_bounded_in_height_clips_its_rows_and_one_that_holds_them_does_not() {
        let theme = Theme::hoff();
        let (table, clips) = drawn(40);
        let full = Table::header_h(&theme) + 40.0 * Table::row_h(&theme);
        assert!(table[3] < full, "bounded: {table:?}");
        assert_eq!(clips, [table], "the rows clipped to the table");
        let (table, clips) = drawn(3);
        let full = Table::header_h(&theme) + 3.0 * Table::row_h(&theme);
        assert!(
            (table[3] - full).abs() < 0.5,
            "as tall as its rows: {table:?}"
        );
        assert!(clips.is_empty(), "drawn as it always was: {clips:?}");
    }
}
