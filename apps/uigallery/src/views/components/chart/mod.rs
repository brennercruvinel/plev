//! Chart page, on the showcase mold of the kit's ButtonsPage
//! (`showcase`): groups with a title and a bottom rule, cards in rows, a
//! chip naming what each card shows. The charts are
//! `engine::widgets::charts` (the pure geometry and its scene nodes); the
//! kit has none, so their recipe (`theme.navigation.chart`) derives from
//! its palette and type.
//!
//! The page shows the forms and the ranges they meet: a line, bars with
//! the peak, two stacked series, a donut with its legend, a meter and
//! labeled bars, and a translated legend. A plot takes its card's width
//! and half of it in height, up to the recipe's cap; the card grows with
//! it. A click on a chart replays the reveal (1.5 s ease-out cubic),
//! which reduced motion settles at once.
//!
//! Projected: a group per title, `chart_page.group[<g>]`, and in it each
//! card's chart, `chart_page.group[<g>].card[<c>]`, an image named by its
//! title and described by what it shows, over a table of its data, one
//! row per datum. A chart is read, not operated: the replay is motion,
//! not state, and offers no action.

#[cfg(test)]
mod tests;

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::compose::animation::{Easing, Tween};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::{ControlSize, Theme};
use engine::widgets::charts::{draw, semantics};
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};

use super::showcase::{self, GroupLayout};
use super::{After, Component, child_id, root_id, under};

/// Reveal timing of the old makepad_charts demo.
const REVEAL_S: f32 = 1.5;

/// What a card plots.
enum Plot {
    Line(Vec<f32>),
    Bars(Vec<f32>),
    Area(Vec<f32>, Vec<f32>),
    Donut(Vec<(&'static str, f32)>),
    Meter(f32),
    Rows(Vec<(String, f64, String)>),
}

struct Card {
    title: &'static str,
    /// What a reader hears the chart is.
    name: &'static str,
    plot: Plot,
}

struct Group {
    title: &'static str,
    span: usize,
    cards: Vec<Card>,
}

pub struct ChartPage {
    root: Option<SemId>,
    groups: Vec<Group>,
    /// 0..1 draw progress. Constructed settled at 1.0 so an idle page
    /// re-renders identically; clicking a chart replays from 0.
    reveal: Tween<f32>,
}

/// `chart_page.group[<g>]`.
fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

/// `chart_page.group[<g>].card[<c>]`.
fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

fn card(title: &'static str, name: &'static str, plot: Plot) -> Card {
    Card { title, name, plot }
}

fn row(name: &str, value: f64, unit: &str) -> (String, f64, String) {
    (name.to_string(), value, format!("{value} {unit}"))
}

impl ChartPage {
    pub fn new() -> Self {
        let line = vec![
            0.3, 0.5, 0.4, 0.7, 0.6, 0.9, 0.75, 0.85, 0.65, 0.95, 0.8, 0.92,
        ];
        let bars = vec![0.6, 0.8, 0.45, 0.9, 0.55, 0.75, 0.65, 0.85];
        let a = vec![0.2, 0.35, 0.3, 0.5, 0.45, 0.6, 0.55, 0.7, 0.65, 0.75];
        let b = vec![0.1, 0.2, 0.15, 0.3, 0.25, 0.35, 0.3, 0.4, 0.35, 0.45];
        let sources = vec![
            ("Search", 35.0),
            ("Direct", 25.0),
            ("Social", 20.0),
            ("Email", 12.0),
            ("Ads", 8.0),
        ];
        let fontes = vec![
            ("Pesquisa orgânica", 35.0),
            ("Acesso direto", 25.0),
            ("Redes sociais", 20.0),
            ("Correio eletrônico", 12.0),
            ("Anúncios pagos", 8.0),
        ];
        let groups = vec![
            Group {
                title: "Chart",
                span: 2,
                cards: vec![
                    card(
                        "Line, reveal on click",
                        "Sessions per week",
                        Plot::Line(line),
                    ),
                    card("Bars, the peak", "Deploys per day", Plot::Bars(bars)),
                    card(
                        "Stacked area, two series",
                        "Bandwidth by region",
                        Plot::Area(a, b),
                    ),
                    card("Donut, legend", "Traffic sources", Plot::Donut(sources)),
                ],
            },
            Group {
                title: "Meter",
                span: 2,
                cards: vec![
                    card("Meter", "Index built", Plot::Meter(0.62)),
                    card(
                        "Labeled bars",
                        "Chunks per store",
                        Plot::Rows(vec![
                            row("vectors", 38412.0, "chunks"),
                            row("documents", 12840.0, "chunks"),
                            row("images", 2481.0, "chunks"),
                        ]),
                    ),
                ],
            },
            Group {
                title: "Long content",
                span: 1,
                cards: vec![card(
                    "Translated legend",
                    "Origem do tráfego",
                    Plot::Donut(fontes),
                )],
            },
        ];
        Self {
            root: root_id("chart_page"),
            groups,
            reveal: Tween::new(1.0, REVEAL_S, Easing::EaseOutCubic),
        }
    }

    /// A plot's size in a card `card_w` wide: the card's width less its
    /// inset, the recipe's share of it in height, up to its cap; a meter
    /// is a hairline's track, the labeled bars a row per item.
    fn plot_size(plot: &Plot, card_w: f32, theme: &Theme) -> (f32, f32) {
        let k = &theme.navigation.chart;
        let w = showcase::room(card_w, theme);
        let row_h = theme.control.height(ControlSize::Xs);
        let h = match plot {
            Plot::Meter(_) => theme.spacing.sm,
            Plot::Rows(items) => row_h * items.len() as f32,
            _ => (w * k.aspect).min(k.max_h),
        };
        (w, h)
    }

    fn layout(&self, content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let groups = self.groups.iter().map(|g| {
            let w = showcase::card_width(g.span, content, theme);
            let heights = g
                .cards
                .iter()
                .map(|c| showcase::card_height(Self::plot_size(&c.plot, w, theme).1, theme))
                .collect();
            (g.span, heights)
        });
        showcase::layout_sized(groups, content, theme)
    }

    fn plot_rect(plot: &Plot, card: Rect, theme: &Theme) -> Rect {
        showcase::centered(Self::plot_size(plot, card.w, theme), card)
    }

    /// Natural height of the laid-out groups.
    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        showcase::content_height(&self.layout(content, theme), content)
    }

    /// Clicking any card replays the reveal from zero.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        if let WidgetEvent::MouseDown { x, y } = *event
            && self
                .layout(content, theme)
                .iter()
                .flat_map(|g| &g.cards)
                .any(|r| r.contains(x, y))
        {
            self.reveal.reset(0.0);
            self.reveal.set_target(1.0);
            return EventResult::clicked();
        }
        EventResult::IGNORED
    }

    /// Advance the reveal. Returns `true` while it still needs frames.
    pub fn tick(&mut self, dt: f32) -> bool {
        self.reveal.tick(dt);
        self.reveal.is_animating()
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let r = self.reveal.get().clamp(0.0, 1.0);
        let layout = self.layout(content, theme);
        let last = self.groups.len().saturating_sub(1);
        for (g, (group, at)) in self.groups.iter().zip(&layout).enumerate() {
            showcase::title(c, group.title, content, at.title_y, theme);
            for (k, (card, rect)) in group.cards.iter().zip(&at.cards).enumerate() {
                showcase::card(c, *rect, card.title, theme);
                let p = Self::plot_rect(&card.plot, *rect, theme);
                if let Some(id) = self.root.as_ref().and_then(|root| card_id(root, g, k)) {
                    c.bind(&id, [p.x, p.y, p.w, p.h]);
                }
                Self::draw(c, &card.plot, p, theme, r);
            }
            if g != last {
                showcase::rule(c, content, at.rule_y, theme);
            }
        }
    }

    fn draw(c: &mut Compositor, plot: &Plot, p: Rect, theme: &Theme, r: f32) {
        match plot {
            Plot::Line(data) => draw::line(c, data, p, theme, r),
            Plot::Bars(data) => draw::bars(c, data, p, theme, r),
            Plot::Area(a, b) => draw::area(c, a, b, p, theme, r),
            Plot::Donut(items) => draw::donut(c, items, p, theme, r),
            Plot::Meter(v) => draw::meter(c, v * r, p, theme),
            Plot::Rows(items) => {
                let row_h = p.h / items.len().max(1) as f32;
                draw::hbars(c, items, p, theme, row_h);
            }
        }
    }

    /// What a reader hears the chart is, and its data row by row.
    fn summary(card: &Card) -> (String, Vec<(String, String)>) {
        let series = |data: &[f32]| -> Vec<(String, String)> {
            let names = data.iter().enumerate().map(|(i, _)| (i + 1).to_string());
            names.zip(data.iter().map(|v| format!("{v:.2}"))).collect()
        };
        let span = |data: &[f32]| {
            let lo = data.iter().copied().fold(f32::INFINITY, f32::min);
            let hi = data.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            format!("{} values from {lo:.2} to {hi:.2}", data.len())
        };
        match &card.plot {
            Plot::Line(d) => (format!("line chart, {}", span(d)), series(d)),
            Plot::Bars(d) => (format!("bar chart, {}", span(d)), series(d)),
            Plot::Area(a, b) => {
                let rows = series(a)
                    .into_iter()
                    .zip(series(b))
                    .map(|((n, x), (_, y))| (n, format!("{x} and {y}")))
                    .collect();
                (
                    format!("stacked area chart, two series of {}", a.len()),
                    rows,
                )
            }
            Plot::Donut(items) => {
                let total: f32 = items.iter().map(|(_, v)| v).sum();
                let rows = items
                    .iter()
                    .map(|(n, v)| (n.to_string(), format!("{:.0}%", v / total * 100.0)))
                    .collect();
                (format!("donut chart, {} parts", items.len()), rows)
            }
            Plot::Meter(v) => (
                format!("meter at {:.0}%", v * 100.0),
                vec![("done".into(), format!("{:.0}%", v * 100.0))],
            ),
            Plot::Rows(items) => {
                let rows = items
                    .iter()
                    .map(|(n, _, l)| (n.clone(), l.clone()))
                    .collect();
                (format!("bar list, {} items", items.len()), rows)
            }
        }
    }
}

impl Component for ChartPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        for (g, group) in self.groups.iter().enumerate() {
            let Some(at) = group_id(root, g) else {
                continue;
            };
            b.node(&at, Role::Group, group.title).parent(root);
            for (c, card) in group.cards.iter().enumerate() {
                let Some(id) = card_id(root, g, c) else {
                    continue;
                };
                let (summary, rows) = Self::summary(card);
                semantics::project(b, &id, Some(&at), card.name, &summary, &rows);
            }
        }
    }

    /// A chart offers nothing to run.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        Err(format!(
            "the chart page has no {} on {}",
            req.action, req.target
        ))
    }

    /// A reader lands on a chart or a row to hear it; nothing here draws
    /// a focus, so the page takes it and keeps none.
    fn focus(&mut self, id: &SemId) -> bool {
        self.root.as_ref().is_some_and(|root| under(root, id))
    }
}
