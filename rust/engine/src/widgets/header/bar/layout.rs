//! Where a header's parts land: the row at its parts' widths, the parts
//! that leave first, the search that gives way last and, below the
//! narrowest useful field, the search as its icon button (or alone across
//! the row while it has the caret).

use super::{Header, HeaderItem, HeaderLayout, HeaderSlot, LogoKind, Spacing};
use crate::styling::theme::Theme;
use crate::widgets::widget::Rect;

/// How the search sits in the row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SearchFit {
    /// Its box, shrunk as far as the row asks.
    Box,
    /// Its icon button.
    Icon,
    /// Its box across the row, alone (it has the caret).
    Alone,
}

impl Header {
    pub fn height(&self, theme: &Theme) -> f32 {
        theme.header.height
    }

    fn gap(&self, theme: &Theme) -> f32 {
        match self.spacing {
            Spacing::Wide => theme.header.gap,
            Spacing::Tight => theme.header.gap_tight,
        }
    }

    fn item_size(item: &HeaderItem, theme: &Theme) -> (f32, f32) {
        match item {
            HeaderItem::Logo(l) => l.preferred_size(theme),
            HeaderItem::Action(a) => a.size(theme),
            HeaderItem::Search(g) => (theme.header.search_w, g.height(theme)),
            HeaderItem::Button(b) => b.preferred_size(theme),
            HeaderItem::Avatar(a) => a.size(theme),
        }
    }

    /// The row's width with every part at its own width, the bar's
    /// padding included: what the bar asks of its host to show it whole.
    pub fn preferred_width(&self, theme: &Theme) -> f32 {
        let h = &theme.header;
        let mut w = h.pad_x * 2.0;
        for (i, s) in self.slots.iter().enumerate() {
            if i > 0 {
                w += if s.joined {
                    h.pair_gap
                } else {
                    self.gap(theme)
                };
            }
            w += Self::item_size(&s.item, theme).0;
        }
        w
    }

    /// Where each part goes in `bounds`, the bar.
    pub fn layout(&self, bounds: Rect, theme: &Theme) -> HeaderLayout {
        let l = self.layout_as(bounds, theme, SearchFit::Box);
        let search = self
            .slots
            .iter()
            .zip(&l.slots)
            .find_map(|(s, r)| match s.item {
                HeaderItem::Search(_) => Some(r.map_or(0.0, |r| r.w)),
                _ => None,
            });
        let least = theme.size.field_min_w.min(theme.header.search_w);
        match search {
            Some(w) if w < least => {
                let typing = self.search().is_some_and(|g| g.input.is_focused());
                let fit = if typing {
                    SearchFit::Alone
                } else {
                    SearchFit::Icon
                };
                self.layout_as(bounds, theme, fit)
            }
            _ => l,
        }
    }

    fn layout_as(&self, bounds: Rect, theme: &Theme, fit: SearchFit) -> HeaderLayout {
        let h = &theme.header;
        let room = (bounds.w - h.pad_x * 2.0).max(0.0);
        let gap = self.gap(theme);
        let alone = fit == SearchFit::Alone;
        // `mark`: a logo shows its symbol alone (the minimal header's).
        let measure = |mark: bool| -> Vec<(f32, f32)> {
            self.slots
                .iter()
                .map(|s| match (&s.item, fit) {
                    (HeaderItem::Search(_), SearchFit::Icon) => self.find.size(theme),
                    (HeaderItem::Search(g), SearchFit::Alone) => (room, g.height(theme)),
                    (HeaderItem::Logo(l), _) if mark => {
                        l.clone().kind(LogoKind::Symbol).preferred_size(theme)
                    }
                    (item, _) => Self::item_size(item, theme),
                })
                .collect()
        };
        let shown = |s: &HeaderSlot| !alone || matches!(s.item, HeaderItem::Search(_));
        let width = |sizes: &[(f32, f32)], keep: &dyn Fn(&HeaderSlot) -> bool| {
            let mut w = 0.0;
            let mut first = true;
            for (s, (sw, _)) in self.slots.iter().zip(sizes) {
                if !keep(s) {
                    continue;
                }
                if !first {
                    w += if s.joined { h.pair_gap } else { gap };
                }
                w += sw;
                first = false;
            }
            w
        };
        // The parts leave tier by tier until the row fits; past the first
        // tier only once the search is its button (its box gives way
        // first).
        let stays = |s: &HeaderSlot, tier: u8| shown(s) && (s.leaves == 0 || s.leaves > tier);
        let last = self.slots.iter().map(|s| s.leaves).max().unwrap_or(0);
        let mut sizes = measure(false);
        // Last of all, past every tier, the logo keeps its symbol only.
        let mark = fit == SearchFit::Icon && width(&sizes, &|s| stays(s, last)) > room;
        if mark {
            sizes = measure(true);
        }
        let mut tier = 0;
        while tier < last
            && (tier == 0 || fit != SearchFit::Box)
            && width(&sizes, &|s| stays(s, tier)) > room
        {
            tier += 1;
        }
        let keep = |s: &HeaderSlot| stays(s, tier);
        let used = width(&sizes, &keep);
        // The search gives way last, down to what is left of it.
        let shrink = (used - room).max(0.0);
        let free = (room - used).max(0.0);
        let mid = bounds.y + bounds.h / 2.0;
        let mut x = bounds.x + h.pad_x;
        let mut first = true;
        let mut slots = Vec::with_capacity(self.slots.len());
        let (mut left_end, mut right_start) = (x, bounds.right() - h.pad_x);
        let mut grown = false;
        for (s, (sw, sh)) in self.slots.iter().zip(&sizes) {
            if !keep(s) {
                slots.push(None);
                continue;
            }
            if !first {
                x += if s.joined { h.pair_gap } else { gap };
            }
            first = false;
            let w = match (&s.item, fit) {
                (HeaderItem::Search(_), SearchFit::Box) => (sw - shrink).max(0.0),
                _ => *sw,
            };
            if grown && right_start > x {
                right_start = x;
            }
            slots.push(Some(Rect::new(x, mid - sh / 2.0, w, *sh)));
            x += w;
            if s.grow {
                left_end = x;
                x += free;
                grown = true;
            }
        }
        let thumbs = self.thumbs.as_ref().filter(|_| !alone).and_then(|t| {
            let (tw, th) = t.size(theme);
            let r = Rect::new(bounds.x + (bounds.w - tw) / 2.0, mid - th / 2.0, tw, th);
            (r.x >= left_end + gap && r.right() <= right_start - gap).then_some(r)
        });
        HeaderLayout {
            slots,
            thumbs,
            compact: fit == SearchFit::Icon,
            mark,
        }
    }
}
