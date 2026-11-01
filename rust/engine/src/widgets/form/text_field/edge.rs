//! The field's border between its states (task-104 front M, Brenner's
//! decision B): at rest the kit's pale gray, a step stronger under the
//! pointer, and in use (focused) strong enough to find the field (3:1,
//! WCAG 1.4.11, `Theme::hold_edges`). The kit's `transition-all` fades
//! between them; here the border fades over the theme's `duration.fast`,
//! and at once with reduced motion (`animation::reduced_motion`).
//!
//! The fade is measured on the clock, not on ticks, so any frame draws the
//! border where it should be; `TextField::wake` asks for frames while it
//! runs. A frame drawn when the state changed starts the fade from the
//! border last drawn.

use std::cell::Cell;

use web_time::Instant;

use crate::compose::animation;
use crate::styling::color::Color;
use crate::widgets::form::tokens::InputRecipe;
use crate::widgets::widget::mix;

/// Where the border stands: 0 at rest, 1 hovered, 2 focused.
#[derive(Clone, Copy, Debug, Default)]
struct Fade {
    from: f32,
    to: f32,
    start: Option<Instant>,
    secs: f32,
    /// A frame was drawn: the next state fades from it.
    seen: bool,
}

impl Fade {
    fn at(&self, now: Instant) -> f32 {
        match self.start {
            Some(start) if self.secs > 0.0 => {
                let t = (now - start).as_secs_f32() / self.secs;
                if t >= 1.0 {
                    self.to
                } else {
                    // ease out: quick to answer, gentle to land
                    let e = 1.0 - (1.0 - t) * (1.0 - t);
                    self.from + (self.to - self.from) * e
                }
            }
            _ => self.to,
        }
    }

    fn running(&self, now: Instant) -> bool {
        self.start
            .is_some_and(|start| (now - start).as_secs_f32() < self.secs)
    }
}

/// The border's fade, kept across frames (`render` takes `&self`).
#[derive(Clone, Debug, Default)]
pub(super) struct EdgeFade(Cell<Fade>);

impl EdgeFade {
    /// The border to draw for a field in `target` (0 rest, 1 hovered, 2
    /// focused), fading over `secs` from the border last drawn.
    pub(super) fn border(&self, recipe: &InputRecipe, target: f32, secs: f32) -> Color {
        let now = Instant::now();
        let mut f = self.0.get();
        if !f.seen {
            // the first frame draws the state it finds
            f = Fade {
                from: target,
                to: target,
                start: None,
                secs,
                seen: true,
            };
        } else if f.to != target {
            let from = f.at(now);
            f = if animation::reduced_motion() || secs <= 0.0 {
                Fade {
                    from: target,
                    to: target,
                    start: None,
                    secs,
                    seen: true,
                }
            } else {
                Fade {
                    from,
                    to: target,
                    start: Some(now),
                    secs,
                    seen: true,
                }
            };
        }
        self.0.set(f);
        let at = f.at(now);
        let [rest, hover, focus] = [recipe.rest.border, recipe.hover.border, recipe.focus.border];
        let (a, b, t) = if at <= 1.0 {
            (rest, hover, at)
        } else {
            (hover, focus, at - 1.0)
        };
        Color(mix(a.0, b.0, t.clamp(0.0, 1.0)))
    }

    /// Whether the fade still runs (a frame is wanted).
    pub(super) fn running(&self) -> bool {
        self.0.get().running(Instant::now())
    }
}

#[cfg(test)]
mod tests {
    use super::EdgeFade;
    use crate::compose::animation::set_reduced_motion;
    use crate::styling::color::Color;
    use crate::styling::theme::Theme;
    use crate::styling::theme::legibility::{MARK_AA, contrast};

    /// Decision B in the four themes: at rest the theme's border, focused
    /// one a person finds (3:1 on the card), hovered a step between.
    #[test]
    fn the_field_border_strengthens_from_rest_to_focus() {
        let kit = Theme::hoff_core();
        assert_eq!(kit.form.input.rest.border, Color::hex(0xE2E2E2));
        for t in [kit, Theme::hoff(), Theme::dark(), Theme::light()] {
            let card = [t.colors.bg, t.colors.surface];
            let i = &t.form.input;
            let [rest, hover, focus] =
                [i.rest.border, i.hover.border, i.focus.border].map(|b| contrast(b, &card));
            assert!(focus >= MARK_AA, "focused at {focus:.2}");
            assert!(rest <= hover && hover <= focus, "{rest} {hover} {focus}");
        }
    }

    /// The border fades over the theme's time, and jumps with reduced
    /// motion; the first frame draws the state it finds.
    #[test]
    fn the_border_fades_and_jumps_with_reduced_motion() {
        let t = Theme::hoff_core();
        let i = &t.form.input;
        let secs = t.duration.fast;
        let fade = EdgeFade::default();
        assert_eq!(fade.border(i, 0.0, secs), i.rest.border);
        // focused: the fade starts from the rest and runs
        let start = fade.border(i, 2.0, secs);
        assert!(fade.running());
        assert_ne!(start, i.focus.border);
        set_reduced_motion(true);
        let jumped = EdgeFade::default();
        jumped.border(i, 0.0, secs);
        assert_eq!(jumped.border(i, 2.0, secs), i.focus.border);
        assert!(!jumped.running());
        set_reduced_motion(false);
    }
}
