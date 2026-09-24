//! each component on its comps widget: the size it takes in the layout
//! and how it draws, from the node, the widget and the snapshot shown.
//!
//! every visual value is a token: the widgets read the theme, and what the
//! host draws itself (a container's fill, a text run, a dialog's title)
//! reads the field the IR token names. a prop a widget does not take (a
//! typography on a text field, a color on a control) is its default.

use comps::Rect;
use comps::action::Badge;
use comps::charts::draw as chart;
use comps::content::{EmptyState, Panel, Separator, Stat, Table};
use comps::feedback::ProgressBar;
use comps::form::TextField;
use comps::nav::Tabs;
use comps::recipe::{rounded_rect, shadow_stack};
use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::layout::TextMeasureSpec;
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::{ControlSize, Elevation, Theme};
use sem::{Snapshot, Value};
use uir::{Axis, ChartKind, Component, SpacingToken};

use crate::expand::Node;
use crate::layout::Leaf;
use crate::token;
use crate::widget::{Widget, dialog_buttons, list_of, text_of};

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
    node.props
        .typography
        .map_or_else(|| theme.typography.body(), |t| token::typography(t, theme))
}

fn stat(node: &Node, shown: &Snapshot) -> Stat {
    let value = node.value("value", shown).map(text_of).unwrap_or_default();
    Stat::new(value, node.label.clone().unwrap_or_default())
}

/// a dialog's padding, its title's style and the band above and below
/// its children.
fn dialog_metrics(node: &Node, theme: &Theme) -> (f32, TextStyle, f32) {
    let pad = token::spacing(node.props.padding.unwrap_or(SpacingToken::Lg), theme);
    (pad, theme.typography.title(), theme.spacing.sm)
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
        Some(Component::Badge) => fixed(Badge::tag(shown_text(node, shown)).preferred_size(theme)),
        Some(Component::Stat) => fixed(stat(node, shown).preferred_size(theme)),
        Some(Component::TextField) => fill(TextField::height(theme)),
        Some(Component::Slider) => fill(theme.control.height(ControlSize::Xs)),
        Some(Component::Select) => fill(theme.control.height(ControlSize::Md)),
        Some(Component::Tabs) => fill(Tabs::height(theme)),
        Some(Component::Table) => {
            let rows = list_of(node.value("rows", shown)).len() as f32;
            fill(Table::header_h(theme) + rows * Table::row_h(theme))
        }
        Some(Component::Chart(_) | Component::Image | Component::EmptyState) => {
            fill(block_h(theme))
        }
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
            let style = theme.typography.base_2r();
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
/// does not bind itself.
pub(crate) fn draw(
    c: &mut Compositor,
    node: &Node,
    widget: &mut Widget,
    rect: Rect,
    shown: &Snapshot,
    theme: &Theme,
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
    if !binds_itself && uir::role_of(node.component, &node.bind).is_some() {
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
        (Some(Component::Badge), _) => Badge::tag(shown_text(node, shown)).render(c, rect, theme),
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
        (Some(Component::Image), _) => image(c, node, rect, theme),
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
        (_, Widget::Slider(s)) => s.render(c, rect, theme),
        (_, Widget::Select(s)) => s.render(c, rect, theme),
        (_, Widget::Tabs(t)) => t.render(c, rect, theme),
        (_, Widget::Chip(b)) => b.render(c, rect, theme),
        (_, Widget::Table(Some(t))) => {
            let rows = list_of(node.value("rows", shown));
            let style = theme.typography.base_2r();
            let ink = theme.colors.text_mid.0;
            t.render_with(c, rect, theme, |c, row, cell, _| {
                if let Some(s) = rows.get(row) {
                    let y = cell.y + TextMeasurer::vertical_center(&style, cell.h);
                    text(c, s, &style, Rect::new(cell.x, y, cell.w, cell.h), ink);
                }
            });
        }
        (_, Widget::Table(None) | Widget::Dialog(_) | Widget::Static) => {}
    }
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

/// an image the surface cannot decode (the IR names a source; the engine
/// takes a GPU image handle): the card shell with its label in the middle.
fn image(c: &mut Compositor, node: &Node, rect: Rect, theme: &Theme) {
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
    let style = theme.typography.caption_r();
    let label = node.label.as_deref().unwrap_or_default();
    let (w, h) = TextMeasurer::measure_styled(label, &style, Some(rect.w));
    let at = Rect::new(
        rect.x + (rect.w - w) / 2.0,
        rect.y + (rect.h - h) / 2.0,
        w,
        h,
    );
    text(c, label, &style, at, theme.colors.text_dim.0);
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
