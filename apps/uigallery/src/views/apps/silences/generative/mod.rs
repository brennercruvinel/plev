//! The generated surface of the Silences page (task-69 phase 4b): the
//! proposing row of the Surface page (`surface_page::proposing`, shared,
//! under the region `silences_surface`), a close button, and the
//! `SurfaceHost` that holds the surface a proposal opened, beside the
//! fixed screen. The rules or the Claude turn an intent into a verified
//! IR over the app's registry (the fixed screen's projection, never the
//! row's), the surface opens or transforms in place by `SemId`, and
//! `close` dismisses it with the exit transition of phase 2. Every
//! capability of the row runs through the gallery's one dispatcher, lent
//! by the todo page to `page::Page`, the `Operable` over the whole page.
//!
//! Where things sit (`Halves`): the row on top, the fixed screen under it
//! at the whole width while no surface is open; with one, the fixed
//! screen takes the left half and the surface the right (wide), or the
//! surface goes under the fixed screen at its natural height (narrow).
//! The surface's images draw the frames the page holds (`jobs::PageFrames`)
//! and the page decodes the ones it lacks from its wake.
//!
//! Under the row, while the page's composer offers a sequence the person
//! repeated (task-70 phase 3), the band of `offer.rs` asks whether to make
//! it one action; the fixed screen moves down by its height, and only
//! then. Under it, while the person has it open (task-70 phase 4), the
//! band of `access.rs` turns their preferences of accessibility on and
//! off, for every surface the host shows.

pub(super) mod access;
pub(super) mod offer;
pub(super) mod page;

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::prelude::{Button, ButtonSize, EventResult, Rect, WidgetEvent};
use screen::SurfaceHost;

use crate::views::apps::surface::offering::{Asked, Proposing, Row};
use access::{Access, Pressed};
use offer::{Answer, Band, Offer};

/// The region the row projects under on this page.
pub(super) const REGION: &str = "silences_surface";

/// The block: the host, the surface it shows, the row and the close.
pub(super) struct Generated {
    pub host: SurfaceHost,
    /// The surface open (the IR's id), when one is.
    pub id: Option<SemId>,
    pub proposing: Proposing,
    close: Button,
    /// `silences_surface.close`; `None` only if the ids do not parse.
    pub close_id: Option<SemId>,
    /// The composite offered to the person, and the page's composer.
    pub offer: Offer,
    /// The person's accessibility band, and whether it is open.
    pub access: Access,
    /// A node of the block or of the surface that holds no focus of its
    /// own (the close, a band's button, a surface's checkbox) and that a
    /// screen reader moved the focus to: kept here and projected.
    pub focus: Option<SemId>,
}

/// Where each part goes in the page's content: the row, the close
/// button at its right, the fixed screen, and the surface when one is
/// open.
pub(super) struct Halves {
    pub row: Row,
    pub close: Rect,
    /// The offer's band, while one stands.
    pub offer: Option<Band>,
    /// The accessibility band, while the person has it open.
    pub access: Option<access::Band>,
    pub fixed: Rect,
    pub surface: Option<Rect>,
}

/// What the row asked, with the close.
pub(super) struct RowAsked {
    pub asked: Asked,
    pub close: bool,
    /// The person's answer to the offer, clicked on its band.
    pub answer: Option<Answer>,
    /// A button of the accessibility band, clicked.
    pub pressed: Option<Pressed>,
}

impl Generated {
    pub fn new() -> Self {
        let proposing = Proposing::new(REGION);
        let close_id = proposing
            .ids
            .as_ref()
            .and_then(|ids| ids.region.child("close").ok());
        let mut close = Button::new("close").size(ButtonSize::Sm).disabled(true);
        if let Some(id) = &close_id {
            close = close.sem(id.clone());
        }
        let offer = Offer::new(proposing.ids.as_ref().map(|ids| &ids.region));
        let access = Access::new(proposing.ids.as_ref().map(|ids| &ids.region));
        Self {
            host: SurfaceHost::new(),
            id: None,
            proposing,
            close,
            close_id,
            offer,
            access,
            focus: None,
        }
    }

    /// True while a surface is open.
    pub fn open(&self) -> bool {
        self.id.is_some()
    }

    /// True when `id` is one of the block's own nodes (the row's, the
    /// close), not the surface's.
    pub fn holds(&self, id: &SemId) -> bool {
        self.proposing.holds(id)
            || self.close_id.as_ref() == Some(id)
            || self.offer.holds(id)
            || self.access.holds(id)
    }

    pub fn set_inert(&mut self, inert: bool) {
        self.proposing.set_inert(inert);
        for s in self.host.iter_mut() {
            s.set_inert(inert);
        }
    }

    /// The row's nodes, the close button (enabled while a surface is
    /// open, its action `close`), and the surface, after the app's own
    /// projection.
    pub fn project(&self, b: &mut SnapshotBuilder) {
        self.host.project(b);
        self.proposing.project(b);
        let (Some(ids), Some(close)) = (&self.proposing.ids, &self.close_id) else {
            return;
        };
        let open = self.open();
        let mut node = b.node(close, Role::Button, "close the surface");
        node.parent(&ids.region).enabled(open);
        node.action("close").enabled(open);
        self.offer.project(b, &ids.region);
        self.access.project(b, &ids.region);
        if let Some(id) = &self.focus
            && b.declared().iter().any(|n| n.id == *id)
        {
            b.focus(id);
        }
    }

    /// `close` on the close button, when `req` asks it.
    pub fn is_close(&self, req: &Request) -> bool {
        req.action.as_str() == "close" && self.close_id.as_ref() == Some(&req.target)
    }

    /// The height the surface takes at `width`; a block's worth while
    /// none is open.
    pub fn surface_h(&self, width: f32, theme: &Theme) -> f32 {
        match self.id.as_ref().and_then(|id| self.host.get(id)) {
            Some(s) => s.height(width, theme),
            None => theme.size.card_min_w / 2.0,
        }
    }

    /// Where everything goes in `content`. `fixed_h` measures the fixed
    /// screen at a width (the narrow case stacks it over the surface).
    pub fn halves(&self, content: Rect, theme: &Theme, fixed_h: impl Fn(Rect) -> f32) -> Halves {
        let gap = theme.spacing.xl;
        let (cw, ch) = self.close.preferred_size(theme);
        let row_w = (content.w - cw - theme.spacing.sm).max(0.0);
        let row = self.proposing.row(content.x, content.y, row_w, theme);
        let close = Rect::new(
            content.right() - cw,
            row.intent.y + (row.intent.h - ch) / 2.0,
            cw,
            ch,
        );
        let offer = self
            .offer
            .band(content.x, row.band.bottom(), content.w, theme);
        let below = offer
            .as_ref()
            .map_or(row.band.bottom(), |o| o.band.bottom());
        let access = self.access.band(content.x, below, content.w, theme);
        let below = access.as_ref().map_or(below, |a| a.band.bottom());
        let h = (content.bottom() - below).max(0.0);
        if !self.open() {
            return Halves {
                row,
                close,
                offer,
                access,
                fixed: Rect::new(content.x, below, content.w, h),
                surface: None,
            };
        }
        if content.w >= theme.size.card_min_w * 2.0 + gap {
            let w = (content.w - gap) / 2.0;
            return Halves {
                row,
                close,
                offer,
                access,
                fixed: Rect::new(content.x, below, w, h),
                surface: Some(Rect::new(content.x + w + gap, below, w, h)),
            };
        }
        let fixed = Rect::new(content.x, below, content.w, 0.0);
        let fixed = Rect::new(fixed.x, fixed.y, fixed.w, fixed_h(fixed));
        Halves {
            row,
            close,
            offer,
            access,
            fixed,
            surface: Some(Rect::new(
                content.x,
                fixed.bottom() + gap,
                content.w,
                content.h,
            )),
        }
    }

    /// A person's pointer on the row and the close button.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        h: &Halves,
        theme: &Theme,
    ) -> (EventResult, RowAsked) {
        self.close.disabled = !self.open();
        if matches!(event, WidgetEvent::MouseDown { .. }) {
            self.focus = None;
        }
        let (r, asked) = self.proposing.handle_event(event, &h.row, theme);
        if asked.taken {
            return (
                r,
                RowAsked {
                    asked,
                    close: false,
                    answer: None,
                    pressed: None,
                },
            );
        }
        let close = self.close.handle_event(event, h.close);
        let (band, answer) = match &h.offer {
            Some(band) => self.offer.handle_event(event, band),
            None => (EventResult::IGNORED, None),
        };
        let (access, pressed) = match &h.access {
            Some(band) => self.access.handle_event(event, band),
            None => (EventResult::IGNORED, None),
        };
        (
            r.merge(close).merge(band).merge(access),
            RowAsked {
                asked,
                close: close.clicked,
                answer,
                pressed,
            },
        )
    }

    /// The row and the close button; the surface and the dropdown are
    /// drawn by the page (the surface needs its frames, the dropdown goes
    /// over everything).
    pub fn render_row(&mut self, c: &mut Compositor, h: &Halves, theme: &Theme) {
        self.close.disabled = !self.open();
        self.proposing.render(c, &h.row, theme);
        self.close.render(c, h.close, theme);
        if let Some(band) = &h.offer {
            self.offer.render(c, band, theme);
        }
        if let Some(band) = &h.access {
            let on = self.host.preferences();
            self.access.render(c, band, theme, on);
        }
    }
}
