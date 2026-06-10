//! Cards section: the HOFF card deck — every `CardVariant` with sample
//! data, laid out as a content-driven masonry grid.

use plev::compositor::Compositor;
use plev::gpu::image::{ImageHandle, load_image_rgba};
use plev::theme::Theme;
use plev::ui::widgets::{Card, CardListRow, CardVariant, EventResult, Rect, WidgetEvent};

use super::group_label;

const GAP: f32 = 16.0;
const LABEL_H: f32 = 24.0;
/// Minimum readable card width — the column count derives from it.
const CARD_MIN_W: f32 = 320.0;
/// Maximum card width, so columns do not degenerate on ultra-wide windows.
const CARD_MAX_W: f32 = 520.0;

/// Procedural cover art: a vertical graphite-to-white wash with a soft
/// diagonal highlight — monochrome, like the deck's PNG previews.
fn cover_image(w: u32, h: u32, seed: f32) -> Option<ImageHandle> {
    let mut pixels = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let fx = x as f32 / w as f32;
            let fy = y as f32 / h as f32;
            // Top-lit wash + a diagonal band, all in the HOFF grays.
            let band = (1.0 - ((fx - fy * 0.8 - seed).abs() * 3.0)).clamp(0.0, 1.0);
            let glow = (1.0 - fy) * 0.22 + band * 0.10;
            let base = 40.0 / 255.0;
            let v = ((base + glow) * 255.0) as u8;
            pixels.extend_from_slice(&[v, v, v, 255]);
        }
    }
    load_image_rgba(w, h, pixels).ok()
}

/// Avatar: a radial-ish bright disc on graphite.
fn avatar_image() -> Option<ImageHandle> {
    let (w, h) = (88u32, 88u32);
    let mut pixels = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let dx = x as f32 / w as f32 - 0.5;
            let dy = y as f32 / h as f32 - 0.38;
            let d = (dx * dx + dy * dy).sqrt();
            let lum = (0.85 - d * 1.4).clamp(0.16, 0.85);
            let v = (lum * 255.0) as u8;
            pixels.extend_from_slice(&[v, v, v, 255]);
        }
    }
    load_image_rgba(w, h, pixels).ok()
}

pub struct CardsSection {
    cards: Vec<(&'static str, Card)>,
}

impl CardsSection {
    pub fn new() -> Self {
        let cards = vec![
            (
                "STAT",
                Card::new(CardVariant::Stat {
                    value: "1,632".into(),
                    label: "Clicks".into(),
                    delta: Some(("+12.4%".into(), true)),
                }),
            ),
            (
                "CHART",
                Card::new(CardVariant::Chart {
                    value: "$408.36".into(),
                    label: "Last month".into(),
                    groups: vec![(0.42, 0.60), (0.55, 0.88), (0.30, 0.48), (0.62, 0.74)],
                    highlight: 1,
                }),
            ),
            (
                "PROFILE",
                Card::new(CardVariant::Profile {
                    name: "Artur Hoff".into(),
                    username: "@hoff".into(),
                    bio: "Building dark glass interfaces. Every surface is \
                          #F8F8F8 at six alphas."
                        .into(),
                    action: "Follow".into(),
                    online: true,
                    avatar: avatar_image(),
                }),
            ),
            (
                "MEDIA",
                Card::new(CardVariant::Media {
                    title: "4K Video Streaming".into(),
                    caption: "Buffer-free playback on every panel.".into(),
                    badge: Some("4K".into()),
                    image: cover_image(352, 248, 0.2),
                }),
            ),
            (
                "LIST",
                Card::new(CardVariant::List {
                    title: "Expense Tracker".into(),
                    rows: vec![
                        CardListRow::new("Starter", "$88.00"),
                        CardListRow::new("Professional", "$128.00").active(true),
                        CardListRow::new("Syncing workspace", "70%").progress(0.7),
                    ],
                }),
            ),
            (
                "STAT — NEGATIVE",
                Card::new(CardVariant::Stat {
                    value: "68%".into(),
                    label: "Bounce rate".into(),
                    delta: Some(("-3.1%".into(), false)),
                }),
            ),
            (
                "MEDIA — GLASS",
                Card::new(CardVariant::Media {
                    title: "Comparative Tools".into(),
                    caption: "The video box recipe without artwork.".into(),
                    badge: Some("HD".into()),
                    image: None,
                }),
            ),
            (
                "CTA",
                Card::new(CardVariant::Cta {
                    title: "Detailed Analytics".into(),
                    body: "Track every click with the comparative toolkit and \
                           the full HOFF glass recipe — one shell, any preview."
                        .into(),
                    button: "Discover".into(),
                }),
            ),
        ];
        Self { cards }
    }

    /// Content-driven masonry: as many columns as `content.w` affords (at
    /// `CARD_MIN_W` each), stretched to fill the row; each card lands in
    /// the currently shorter column.
    fn layout(&self, content: Rect) -> Vec<Rect> {
        let cols = (((content.w + GAP) / (CARD_MIN_W + GAP)).floor() as usize).max(1);
        let col_w = ((content.w - (cols as f32 - 1.0) * GAP) / cols as f32).min(CARD_MAX_W);
        let mut col_y = vec![content.y; cols];
        self.cards
            .iter()
            .map(|(_, card)| {
                // Height measured at the stretched width (CTA bodies
                // re-wrap their text against it).
                let mut sized = card.clone();
                sized.width = col_w;
                let (_, h) = sized.preferred_size();
                let col = (0..cols)
                    .min_by(|a, b| col_y[*a].total_cmp(&col_y[*b]))
                    .unwrap_or(0);
                let rect = Rect::new(
                    content.x + col as f32 * (col_w + GAP),
                    col_y[col] + LABEL_H,
                    col_w,
                    h,
                );
                col_y[col] = rect.y + rect.h + GAP + 8.0;
                rect
            })
            .collect()
    }

    /// Natural height of the laid-out deck (page scrolling needs it).
    pub fn content_height(&self, content: Rect) -> f32 {
        self.layout(content)
            .iter()
            .map(|r| r.y + r.h)
            .fold(content.y, f32::max)
            - content.y
            + GAP
    }

    pub fn handle_event(&mut self, event: &WidgetEvent, content: Rect) -> EventResult {
        let rects = self.layout(content);
        let mut result = EventResult::IGNORED;
        for ((_, card), rect) in self.cards.iter_mut().zip(rects) {
            result = result.merge(card.handle_event(event, rect));
        }
        result
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let rects = self.layout(content);
        for ((label, card), rect) in self.cards.iter().zip(rects) {
            group_label(c, label, rect.x, rect.y - LABEL_H + 2.0, theme);
            card.render(c, rect, theme);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Distinct column x positions of a laid-out deck.
    fn column_xs(rects: &[Rect]) -> Vec<f32> {
        let mut xs: Vec<f32> = rects.iter().map(|r| r.x).collect();
        xs.sort_by(f32::total_cmp);
        xs.dedup();
        xs
    }

    /// Wide viewport (~1600px window): the deck must redistribute into
    /// three or more stretched columns instead of two fixed 368px ones.
    #[test]
    fn cards_redistribute_into_three_plus_columns_in_wide_content() {
        let section = CardsSection::new();
        let content = Rect::new(288.0, 80.0, 1272.0, 700.0);
        let rects = section.layout(content);

        let cols = column_xs(&rects);
        assert!(
            cols.len() >= 3,
            "wide content must yield >= 3 columns, got {}",
            cols.len()
        );

        // Columns stretch past the old fixed card width but stay clamped.
        let col_w = rects[0].w;
        assert!(rects.iter().all(|r| r.w == col_w));
        assert!(col_w > 368.0, "columns must stretch, got {col_w}");
        assert!(col_w <= CARD_MAX_W);

        // Nothing reaches past the content rect.
        let right = content.x + content.w;
        assert!(rects.iter().all(|r| r.x + r.w <= right + 0.5));
    }

    /// Narrow viewport: a single column that follows the content width.
    #[test]
    fn cards_collapse_to_one_stretched_column_in_narrow_content() {
        let section = CardsSection::new();
        let content = Rect::new(288.0, 80.0, 400.0, 700.0);
        let rects = section.layout(content);

        assert_eq!(column_xs(&rects).len(), 1);
        assert!(rects.iter().all(|r| r.x == content.x));
        assert!(rects.iter().all(|r| r.w == content.w));
    }
}
