//! Breadcrumb: the path to where the user is. Crumbs in base-2r, the
//! last one in the primary tone, chevrons between them in the faint
//! tone; every crumb but the last is a target (click fires on release
//! inside, like a button). When the trail is wider than its bounds the
//! leading crumbs collapse into an ellipsis crumb, so the current place
//! always shows.

use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::{IconSize, Theme};

use crate::core::{EventResult, Rect, WidgetEvent};
use crate::icons;

const ELLIPSIS: &str = "\u{2026}";

#[derive(Clone, Debug)]
pub struct Breadcrumb {
    pub crumbs: Vec<String>,
    hovered: Option<usize>,
    pressed: Option<usize>,
}

impl Breadcrumb {
    pub fn new(crumbs: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            crumbs: crumbs.into_iter().map(Into::into).collect(),
            hovered: None,
            pressed: None,
        }
    }

    pub fn hovered(&self) -> Option<usize> {
        self.hovered
    }

    pub fn text_style(theme: &Theme) -> TextStyle {
        theme.typography.base_2r()
    }

    /// Gap around the chevron: the `xs` step each side.
    fn gap(theme: &Theme) -> f32 {
        theme.spacing.xs
    }

    fn chevron(theme: &Theme) -> f32 {
        theme.control.icon(IconSize::Sm)
    }

    /// Visible crumbs for `width`: indices into `crumbs`, with `None`
    /// standing for the ellipsis. Leading crumbs collapse first; the
    /// first and the last stay as long as they fit.
    pub fn visible(&self, width: f32, theme: &Theme) -> Vec<Option<usize>> {
        let n = self.crumbs.len();
        if n == 0 {
            return Vec::new();
        }
        let style = Self::text_style(theme);
        let sep = Self::gap(theme) * 2.0 + Self::chevron(theme);
        let w = |s: &str| TextMeasurer::measure_styled(s, &style, None).0;
        let total = |items: &[Option<usize>]| -> f32 {
            items
                .iter()
                .map(|i| match i {
                    Some(i) => w(&self.crumbs[*i]),
                    None => w(ELLIPSIS),
                })
                .sum::<f32>()
                + sep * (items.len().saturating_sub(1)) as f32
        };
        let all: Vec<Option<usize>> = (0..n).map(Some).collect();
        if total(&all) <= width {
            return all;
        }
        // Collapse from the second crumb onward, keeping the first.
        for keep_tail in (1..n).rev() {
            let mut items = vec![Some(0), None];
            items.extend((n - keep_tail..n).map(Some));
            if n - keep_tail <= 1 {
                continue;
            }
            if total(&items) <= width {
                return items;
            }
        }
        // Nothing but the last fits.
        vec![None, Some(n - 1)]
    }

    /// Crumb rects for the visible trail (parallel to `visible`).
    pub fn crumb_rects(&self, bounds: Rect, theme: &Theme) -> Vec<(Option<usize>, Rect)> {
        let style = Self::text_style(theme);
        let sep = Self::gap(theme) * 2.0 + Self::chevron(theme);
        let mut x = bounds.x;
        self.visible(bounds.w, theme)
            .into_iter()
            .map(|i| {
                let text = match i {
                    Some(i) => self.crumbs[i].as_str(),
                    None => ELLIPSIS,
                };
                let (tw, _) = TextMeasurer::measure_styled(text, &style, None);
                let r = Rect::new(x, bounds.y, tw, bounds.h);
                x += tw + sep;
                (i, r)
            })
            .collect()
    }

    fn crumb_at(&self, x: f32, y: f32, bounds: Rect, theme: &Theme) -> Option<usize> {
        let last = self.crumbs.len().saturating_sub(1);
        self.crumb_rects(bounds, theme)
            .into_iter()
            .find_map(|(i, r)| match i {
                Some(i) if i != last && r.contains(x, y) => Some(i),
                _ => None,
            })
    }

    /// Returns the clicked crumb index on activation.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> (EventResult, Option<usize>) {
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let hit = self.crumb_at(x, y, bounds, theme);
                if hit != self.hovered {
                    self.hovered = hit;
                    (EventResult::changed(), None)
                } else {
                    (EventResult::IGNORED, None)
                }
            }
            WidgetEvent::MouseDown { x, y } => {
                self.pressed = self.crumb_at(x, y, bounds, theme);
                if self.pressed.is_some() {
                    (EventResult::changed(), None)
                } else {
                    (EventResult::IGNORED, None)
                }
            }
            WidgetEvent::MouseUp { x, y } => {
                let Some(pressed) = self.pressed.take() else {
                    return (EventResult::IGNORED, None);
                };
                if self.crumb_at(x, y, bounds, theme) == Some(pressed) {
                    (EventResult::clicked(), Some(pressed))
                } else {
                    (EventResult::changed(), None)
                }
            }
            WidgetEvent::Scroll { .. } => (EventResult::IGNORED, None),
        }
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.render_to_layer(c, LayerId::DEFAULT, bounds, theme);
    }

    pub fn render_to_layer(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        let style = Self::text_style(theme);
        let glass = &theme.glass;
        let last = self.crumbs.len().saturating_sub(1);
        let chevron = Self::chevron(theme);
        let gap = Self::gap(theme);
        let rects = self.crumb_rects(bounds, theme);
        for (k, (i, r)) in rects.iter().enumerate() {
            let (text, color) = match i {
                Some(i) if *i == last => (self.crumbs[*i].as_str(), theme.colors.text.0),
                Some(i) if self.hovered == Some(*i) => {
                    (self.crumbs[*i].as_str(), glass.text_active.0)
                }
                Some(i) => (self.crumbs[*i].as_str(), theme.colors.text_mid.0),
                None => (ELLIPSIS, glass.text_faint.0),
            };
            c.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(text, &style, None),
                    x: r.x,
                    y: r.y + TextMeasurer::vertical_center(&style, r.h),
                    color,
                },
            );
            if k + 1 < rects.len()
                && let Some(node) = icons::icon_at(
                    "chevron-right",
                    chevron,
                    glass.text_faint.0,
                    r.x + r.w + gap,
                    r.y + (r.h - chevron) / 2.0,
                )
            {
                c.push_to_layer(layer, node);
            }
        }
    }
}
