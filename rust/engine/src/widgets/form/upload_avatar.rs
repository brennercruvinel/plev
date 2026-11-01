//! Upload avatar: the kit's UploadAvatar (`components/UploadAvatar/
//! index.tsx`), a round photo that, hovered, darkens under a plus: a
//! press on it asks for a file to replace the photo. Without a photo the
//! disc shows the kit's stand-in (`/images/no-avatar.png`). A photo is an
//! image the owner gives ([`UploadAvatar::chosen`]); where there is none
//! to give, the theme's stand-in for the kit's photo stands in the disc
//! (`theme.form.upload.photo`, a gradient of its colors).
//!
//! The widget asks; the owner chooses: a press, Enter or Space on it, or
//! a reader's or an agent's `activate`, turn it to choosing
//! ([`UploadAvatar::activate`]); the owner opens its file chooser, takes
//! the ask ([`UploadAvatar::take_ask`]) and answers with what was chosen.
//! The keyboard focus shows the plus as the hover does, with the focus
//! ring around the disc. Every value is `theme.form.upload`'s.
//!
//! Projected ([`UploadAvatar::project`]): a `Role::Button` named by its
//! label ("Replace avatar", or "Upload avatar" without a photo), its
//! `photo` state whether one is in, with an `activate`.

use accord::{Role, SemId, SnapshotBuilder};

use crate::drawing::compositor::{Compositor, LayerId, SceneNode};
use crate::drawing::gpu::image::ImageHandle;
use crate::styling::color::Color;
use crate::styling::theme::Theme;
use crate::widgets::icons;
use crate::widgets::recipe::{focus_ring, rounded_rect};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent, with_alpha};

/// The kit's plus in a ring (UploadAvatar :24, `icons` `kit-plus-circle`).
const PLUS: &str = "kit-plus-circle";

/// A round photo that asks to be replaced.
#[derive(Clone, Debug)]
pub struct UploadAvatar {
    /// A photo is in: the image given, or the theme's stand-in for one.
    photo: bool,
    image: Option<ImageHandle>,
    hovered: bool,
    focused: bool,
    disabled: bool,
    /// Asked for a file, until the owner takes the ask.
    asking: bool,
    sem: Option<SemId>,
}

impl Default for UploadAvatar {
    fn default() -> Self {
        Self::new()
    }
}

impl UploadAvatar {
    /// Without a photo: the stand-in.
    pub fn new() -> Self {
        Self {
            photo: false,
            image: None,
            hovered: false,
            focused: false,
            disabled: false,
            asking: false,
            sem: None,
        }
    }

    /// With a photo: `image`, or the theme's stand-in when `None`.
    pub fn with_photo(mut self, image: Option<ImageHandle>) -> Self {
        self.photo = true;
        self.image = image;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    pub fn has_photo(&self) -> bool {
        self.photo
    }

    /// What a reader hears: replacing a photo, or giving one.
    pub fn label(&self) -> &'static str {
        if self.photo {
            "Replace avatar"
        } else {
            "Upload avatar"
        }
    }

    pub fn set_focused(&mut self, on: bool) {
        self.focused = on && !self.disabled;
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Ask for a file (a press, Enter, Space, an `activate`). `false`
    /// when disabled.
    pub fn activate(&mut self) -> bool {
        if self.disabled {
            return false;
        }
        self.asking = true;
        true
    }

    pub fn is_asking(&self) -> bool {
        self.asking
    }

    /// The owner takes the ask, to open its file chooser: `true` once per
    /// ask.
    pub fn take_ask(&mut self) -> bool {
        std::mem::take(&mut self.asking)
    }

    /// What was chosen: `image`, or the theme's stand-in for a photo when
    /// the owner has none to show.
    pub fn chosen(&mut self, image: Option<ImageHandle>) {
        self.photo = true;
        self.image = image;
    }

    /// The disc in `bounds`: the kit's side, never wider than the room,
    /// from the top left (its height is not read).
    pub fn disc(&self, bounds: Rect, theme: &Theme) -> Rect {
        let side = theme.form.upload.size.min(bounds.w).max(0.0);
        Rect::new(bounds.x, bounds.y, side, side)
    }

    /// Its size with `room` across.
    pub fn size(&self, room: f32, theme: &Theme) -> (f32, f32) {
        let side = theme.form.upload.size.min(room);
        (side, side)
    }

    /// Whether `(x, y)` is on the disc (round: its corners are not).
    pub fn contains(&self, x: f32, y: f32, bounds: Rect, theme: &Theme) -> bool {
        let d = self.disc(bounds, theme);
        let (cx, cy) = d.center();
        let r = d.w / 2.0;
        (x - cx).powi(2) + (y - cy).powi(2) <= r * r
    }

    /// The pointer: hover shows the plus, a press asks for a file.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        if self.disabled {
            return if std::mem::take(&mut self.hovered) {
                EventResult::changed()
            } else {
                EventResult::IGNORED
            };
        }
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let over = self.contains(x, y, bounds, theme);
                if over != self.hovered {
                    self.hovered = over;
                    EventResult::changed()
                } else {
                    EventResult::IGNORED
                }
            }
            WidgetEvent::MouseDown { x, y } if self.contains(x, y, bounds, theme) => {
                self.activate();
                EventResult::clicked()
            }
            _ => EventResult::IGNORED,
        }
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        let layer = LayerId::DEFAULT;
        let t = &theme.form.upload;
        let d = self.disc(bounds, theme);
        let r = d.w / 2.0;
        let faded = if self.disabled {
            theme.form.input.disabled_alpha
        } else {
            1.0
        };
        let fade = |color: Color| with_alpha(color, color.0[3] * faded);
        if let Some(id) = &self.sem {
            c.bind_to_layer(layer, id, [d.x, d.y, d.w, d.h]);
        }
        c.push(rounded_rect(d.x, d.y, d.w, d.h, r, fade(t.fill)));
        match (self.photo, self.image) {
            (true, Some(image)) => c.push(SceneNode::Image {
                x: d.x,
                y: d.y,
                w: d.w,
                h: d.h,
                image,
                corner_radius: r,
            }),
            (photo, _) => {
                let [top, bottom] = if photo { t.photo } else { t.empty };
                // css `to bottom`: the first stop at the top.
                c.push(SceneNode::GradientRect {
                    x: d.x,
                    y: d.y,
                    w: d.w,
                    h: d.h,
                    color: fade(top),
                    color2: fade(bottom),
                    angle_deg: 180.0,
                    corner_radius: r,
                    border_width: 0.0,
                    border_color: [0.0; 4],
                });
            }
        }
        if (self.hovered || self.focused) && !self.disabled {
            c.push(rounded_rect(d.x, d.y, d.w, d.h, r, t.scrim.0));
            let side = t.icon.min(d.w);
            let (cx, cy) = d.center();
            let at = (cx - side / 2.0, cy - side / 2.0);
            if let Some(node) = icons::icon_at(PLUS, side, t.icon_color.0, at.0, at.1) {
                c.push(node);
            }
        }
        if self.focused {
            c.push(focus_ring(d, r, theme));
        }
    }

    /// Declares it as `id` (see the module docs). It only reads the
    /// widget: the owner runs `activate` ([`UploadAvatar::activate`]) and
    /// answers the ask.
    #[track_caller]
    pub fn project(&self, b: &mut SnapshotBuilder, id: &SemId, parent: Option<&SemId>) {
        let mut node = b.node(id, Role::Button, self.label());
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.state("photo", self.photo)
            .enabled(!self.disabled)
            .focusable(true);
        node.action("activate");
    }
}
