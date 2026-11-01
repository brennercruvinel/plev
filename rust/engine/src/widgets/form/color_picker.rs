//! Color picker: the kit's ColorPicker (`components/ColorPicker/
//! index.tsx`), one box with a swatch, the color's hex typed beside it
//! and its opacity in percent at the end, behind a divider. The swatch
//! paints the color typed over its base, the opacity showing it (the kit
//! leaves its swatch blank: painting it is this widget's).
//!
//! The two texts are bare text fields ([`Look::Bare`], `picker.hex` and
//! `picker.alpha`): typing, the editing keys, the clipboard and the input
//! method reach the one with the caret as any field. The hex takes six hex
//! digits, the opacity three digits; a hex of six digits (or three, as
//! css writes `#FFF`) and an opacity up to 100 are the color as soon as
//! they are typed, the color kept showing while they are not one, and a
//! box left with something else shows the color kept again. A press on the box puts the caret in the part pressed (the
//! swatch is the hex's, the `%` the opacity's), Tab goes from the hex to
//! the opacity and Shift+Tab back ([`ColorPicker::tab`]), Enter keeps
//! what was typed, and Escape puts back the color the caret came in to,
//! and lets go. Every value is `theme.form.color`'s, the texts the bare
//! box's.
//!
//! Projected ([`ColorPicker::project`]): a group named `label`, its
//! `value` the color as `#RRGGBBAA`, with its two text fields in it,
//! `<id>.hex` ("Hex") and `<id>.alpha` ("Opacity", described as a percent);
//! `set_value` on either sets that part, refused when it is not one.

use accord::{Role, SemId, SnapshotBuilder};

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::TextMeasurer;
use crate::styling::color::Color;
use crate::styling::theme::Theme;
use crate::widgets::form::text_field::{EditKey, TextField};
use crate::widgets::form::text_input::{Accept, Chars};
use crate::widgets::form::tokens::Look;
use crate::widgets::recipe::{rounded_rect, rounded_rect_stroke};
use crate::widgets::wake::Wake;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent, with_alpha};

/// The opacity's sign, after its number.
const PERCENT: &str = "%";

/// The widest opacity, what its part makes room for.
const FULL: &str = "100";

/// One of the two texts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorPart {
    Hex,
    Alpha,
}

/// Where the parts go.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorPickerLayout {
    pub whole: Rect,
    pub swatch: Rect,
    pub hex: Rect,
    /// The opacity's part, from its divider to the box's padding.
    pub alpha_part: Rect,
    pub divider: Rect,
    pub alpha: Rect,
    pub suffix: Rect,
}

/// A color typed as hex with its opacity.
#[derive(Clone, Debug)]
pub struct ColorPicker {
    pub hex: TextField,
    pub alpha: TextField,
    rgb: [u8; 3],
    /// Percent, 0 to 100.
    opacity: u8,
    hovered: bool,
    /// The color when the caret came in, for Escape.
    before: Option<([u8; 3], u8)>,
    sem: Option<SemId>,
}

/// `text` as a color: six hex digits, or three (each doubled), with or
/// without a `#`.
pub fn parse_hex(text: &str) -> Option<[u8; 3]> {
    let digits = text.trim().trim_start_matches('#');
    if !digits.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let wide: String = match digits.len() {
        6 => digits.to_owned(),
        3 => digits.chars().flat_map(|c| [c, c]).collect(),
        _ => return None,
    };
    let byte = |i: usize| u8::from_str_radix(&wide[i..i + 2], 16).ok();
    Some([byte(0)?, byte(2)?, byte(4)?])
}

/// `text` as an opacity in percent: a whole number up to 100, with or
/// without its `%`.
pub fn parse_opacity(text: &str) -> Option<u8> {
    let n: u32 = text.trim().trim_end_matches('%').trim().parse().ok()?;
    u8::try_from(n).ok().filter(|n| *n <= 100)
}

fn hex_text(rgb: [u8; 3]) -> String {
    format!("{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2])
}

impl ColorPicker {
    /// The color `hex` (black when it is not one) at `opacity` percent.
    pub fn new(hex: &str, opacity: u8) -> Self {
        let rgb = parse_hex(hex).unwrap_or([0; 3]);
        let opacity = opacity.min(100);
        let hex = TextField::new("Hex").look(Look::Bare).accept(Accept {
            chars: Chars::Hex,
            max: Some(6),
        });
        let alpha = TextField::new(FULL)
            .look(Look::Bare)
            .accept(Accept::digits(FULL.len()));
        Self {
            hex: hex.with_text(&hex_text(rgb)),
            alpha: alpha.with_text(&opacity.to_string()),
            rgb,
            opacity,
            hovered: false,
            before: None,
            sem: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.hex.disabled = disabled;
        self.alpha.disabled = disabled;
        self
    }

    pub fn is_disabled(&self) -> bool {
        self.hex.disabled
    }

    /// The id it projects as and binds the box to; the texts are
    /// `<id>.hex` and `<id>.alpha`.
    pub fn sem(mut self, id: SemId) -> Self {
        if let (Ok(hex), Ok(alpha)) = (id.child("hex"), id.child("alpha")) {
            self.hex = self.hex.sem(hex);
            self.alpha = self.alpha.sem(alpha);
        }
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    /// The id of the text `part`.
    pub fn text_id(&self, part: ColorPart) -> Option<&SemId> {
        self.field(part).sem_id()
    }

    pub fn field(&self, part: ColorPart) -> &TextField {
        match part {
            ColorPart::Hex => &self.hex,
            ColorPart::Alpha => &self.alpha,
        }
    }

    fn field_mut(&mut self, part: ColorPart) -> &mut TextField {
        match part {
            ColorPart::Hex => &mut self.hex,
            ColorPart::Alpha => &mut self.alpha,
        }
    }

    /// The color as it shows: what is typed where it is one, else the
    /// color before it.
    pub fn rgb(&self) -> [u8; 3] {
        parse_hex(self.hex.text()).unwrap_or(self.rgb)
    }

    /// The opacity as it shows, in percent.
    pub fn opacity(&self) -> u8 {
        parse_opacity(self.alpha.text()).unwrap_or(self.opacity)
    }

    /// The color with its opacity.
    pub fn color(&self) -> Color {
        let [r, g, b] = self.rgb().map(|v| f32::from(v) / 255.0);
        Color::rgba(r, g, b, f32::from(self.opacity()) / 100.0)
    }

    /// The color as css writes it with its opacity, `#RRGGBBAA`.
    pub fn value(&self) -> String {
        let a = (f32::from(self.opacity()) * 255.0 / 100.0).round() as u8;
        format!("#{}{a:02X}", hex_text(self.rgb()))
    }

    /// The text with the caret.
    pub fn focused(&self) -> Option<ColorPart> {
        [ColorPart::Hex, ColorPart::Alpha]
            .into_iter()
            .find(|p| self.field(*p).is_focused())
    }

    /// The text the keys go to: the one with the caret, else the hex.
    pub fn text(&self) -> &TextField {
        self.field(self.focused().unwrap_or(ColorPart::Hex))
    }

    pub fn text_mut(&mut self) -> &mut TextField {
        self.field_mut(self.focused().unwrap_or(ColorPart::Hex))
    }

    /// The caret to `part`, off the other (which shows the color again
    /// if it was left with something else). `false` when disabled.
    pub fn focus(&mut self, part: ColorPart) -> bool {
        if self.is_disabled() {
            return false;
        }
        if self.before.is_none() {
            self.before = Some((self.rgb(), self.opacity()));
        }
        let other = match part {
            ColorPart::Hex => ColorPart::Alpha,
            ColorPart::Alpha => ColorPart::Hex,
        };
        self.field_mut(other).unfocus();
        self.field_mut(part).focus();
        self.settle();
        true
    }

    /// The caret off both, what was typed kept where it is a color.
    pub fn blur(&mut self) -> bool {
        let had = self.focused().is_some();
        self.hex.unfocus();
        self.alpha.unfocus();
        self.before = None;
        self.settle();
        had
    }

    /// Tab (`forward`) from the hex to the opacity, Shift+Tab back.
    /// `false` past either end: the focus leaves the picker.
    pub fn tab(&mut self, forward: bool) -> bool {
        match (self.focused(), forward) {
            (Some(ColorPart::Hex), true) => self.focus(ColorPart::Alpha),
            (Some(ColorPart::Alpha), false) => self.focus(ColorPart::Hex),
            _ => false,
        }
    }

    /// Enter: what was typed is kept (the box with the caret shows the
    /// color it makes), the caret stays. `false` without the caret.
    pub fn enter(&mut self) -> bool {
        let Some(part) = self.focused() else {
            return false;
        };
        self.commit();
        let text = match part {
            ColorPart::Hex => hex_text(self.rgb),
            ColorPart::Alpha => self.opacity.to_string(),
        };
        let field = self.field_mut(part);
        field.set_text(&text);
        field.edit(EditKey::End);
        true
    }

    /// Escape: the color the caret came in to, and the caret off.
    /// `false` without the caret.
    pub fn escape(&mut self) -> bool {
        if self.focused().is_none() {
            return false;
        }
        if let Some((rgb, opacity)) = self.before {
            self.rgb = rgb;
            self.opacity = opacity;
            self.hex.set_text(&hex_text(rgb));
            self.alpha.set_text(&opacity.to_string());
        }
        self.blur()
    }

    /// The hex set whole (an agent's, a reader's), with or without its
    /// `#`. `false`, the color as it was, when it is not one or disabled.
    pub fn set_hex(&mut self, text: &str) -> bool {
        let Some(rgb) = parse_hex(text).filter(|_| !self.is_disabled()) else {
            return false;
        };
        self.rgb = rgb;
        self.hex.set_text(&hex_text(rgb));
        true
    }

    /// The opacity set whole, a number up to 100, with or without its
    /// `%`. `false`, as it was, when it is not one or disabled.
    pub fn set_opacity(&mut self, text: &str) -> bool {
        let Some(opacity) = parse_opacity(text).filter(|_| !self.is_disabled()) else {
            return false;
        };
        self.opacity = opacity;
        self.alpha.set_text(&opacity.to_string());
        true
    }

    /// What is typed becomes the color where it is one.
    fn commit(&mut self) {
        self.rgb = self.rgb();
        self.opacity = self.opacity();
    }

    /// A text without the caret shows the color, as it is kept.
    fn settle(&mut self) {
        self.commit();
        if !self.hex.is_focused() {
            self.hex.set_text(&hex_text(self.rgb));
        }
        if !self.alpha.is_focused() {
            self.alpha.set_text(&self.opacity.to_string());
        }
    }

    /// The box's height: its border, its padding and the taller of the
    /// swatch and a text.
    pub fn height(&self, theme: &Theme) -> f32 {
        let k = &theme.form.color;
        (k.border_width + k.pad) * 2.0 + k.swatch.max(self.hex.box_height(theme))
    }

    /// Its size with `room` across: the kit's width, never more.
    pub fn size(&self, room: f32, theme: &Theme) -> (f32, f32) {
        (theme.form.color.width.min(room), self.height(theme))
    }

    /// Where the parts go in `bounds` (its height is not read: the box
    /// is [`height`](Self::height) tall).
    pub fn layout(&self, bounds: Rect, theme: &Theme) -> ColorPickerLayout {
        let k = &theme.form.color;
        let style = &theme.typography.run(theme.form.bare.text);
        let whole = Rect::new(bounds.x, bounds.y, bounds.w, self.height(theme));
        let edge = k.border_width + k.pad;
        let inner = Rect::new(
            whole.x + edge,
            whole.y + edge,
            (whole.w - edge * 2.0).max(0.0),
            (whole.h - edge * 2.0).max(0.0),
        );
        let mid = inner.y + inner.h / 2.0;
        let swatch_side = k.swatch.min(inner.w);
        let swatch = Rect::new(inner.x, mid - swatch_side / 2.0, swatch_side, swatch_side);
        let measure = |t: &str| TextMeasurer::measure_styled(t, style, None).0.ceil();
        let suffix_w = measure(PERCENT);
        // The number's room: the widest opacity and a caret after it.
        let number_w = measure(FULL) + theme.control.focus_ring_width;
        let divider_w = k.border_width;
        // The kit's width, never under what the widest opacity, its `%`
        // and the part's insets measure (a derived theme's 0: as wide as
        // that), so `100 %` shows whole.
        let measured = divider_w + k.alpha_pad * 2.0 + number_w + k.alpha_gap + suffix_w;
        let alpha_w = k
            .alpha_w
            .max(measured)
            .min((inner.right() - swatch.right() - k.swatch_gap).max(0.0));
        let alpha_part = Rect::new(inner.right() - alpha_w, inner.y, alpha_w, inner.h);
        let text_h = self.hex.box_height(theme);
        let line_h = style.line_height.min(inner.h);
        let divider = Rect::new(alpha_part.x, mid - line_h / 2.0, divider_w, line_h);
        let alpha_x = divider.right() + k.alpha_pad;
        let alpha_room = (alpha_part.right() - k.alpha_pad - alpha_x).max(0.0);
        let suffix_w = suffix_w.min(alpha_room);
        let number = (alpha_room - k.alpha_gap - suffix_w).max(0.0);
        let alpha = Rect::new(alpha_x, mid - text_h / 2.0, number, text_h);
        let suffix = Rect::new(
            alpha_part.right() - k.alpha_pad - suffix_w,
            alpha.y,
            suffix_w,
            text_h,
        );
        let hex_x = (swatch.right() + k.swatch_gap).min(alpha_part.x);
        let hex = Rect::new(
            hex_x,
            mid - text_h / 2.0,
            (alpha_part.x - hex_x).max(0.0),
            text_h,
        );
        ColorPickerLayout {
            whole,
            swatch,
            hex,
            alpha_part,
            divider,
            alpha,
            suffix,
        }
    }

    /// The pointer: hover lights the box; a press in it puts the caret in
    /// the part pressed, at the glyph pressed on a text and at the end
    /// elsewhere; a press off it lets the caret go.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        let l = self.layout(bounds, theme);
        if self.is_disabled() {
            let was = self.hovered;
            self.hovered = false;
            return if self.blur() || was {
                EventResult::changed()
            } else {
                EventResult::IGNORED
            };
        }
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let over = l.whole.contains(x, y);
                let r = self
                    .hex
                    .handle_event(event, l.hex, theme)
                    .merge(self.alpha.handle_event(event, l.alpha, theme));
                if over != self.hovered {
                    self.hovered = over;
                    return r.merge(EventResult::changed());
                }
                r
            }
            WidgetEvent::MouseDown { x, y } if l.whole.contains(x, y) => {
                let (part, rect) = if x >= l.alpha_part.x {
                    (ColorPart::Alpha, l.alpha)
                } else {
                    (ColorPart::Hex, l.hex)
                };
                self.focus(part);
                let field = self.field_mut(part);
                if rect.contains(x, y) {
                    field.handle_event(event, rect, theme);
                } else {
                    field.edit(EditKey::End);
                }
                EventResult::clicked()
            }
            WidgetEvent::MouseDown { .. } if self.blur() => EventResult::changed(),
            _ => EventResult::IGNORED,
        }
    }

    /// The caret blink of the text with the caret.
    pub fn wake(&mut self, dt: f32) -> Wake {
        match self.focused() {
            Some(part) => self.field_mut(part).wake(dt),
            None => Wake::idle(),
        }
    }

    /// Where the input method's window goes: the caret's text's caret.
    pub fn ime_area(&self, bounds: Rect, theme: &Theme) -> Option<Rect> {
        let l = self.layout(bounds, theme);
        Some(match self.focused()? {
            ColorPart::Hex => self.hex.ime_area(l.hex, theme),
            ColorPart::Alpha => self.alpha.ime_area(l.alpha, theme),
        })
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        let scale = &theme.typography;
        let layer = LayerId::DEFAULT;
        let k = &theme.form.color;
        let l = self.layout(bounds, theme);
        let faded = if self.is_disabled() {
            theme.form.bare.disabled_alpha
        } else {
            1.0
        };
        let fade = |color: Color| with_alpha(color, color.0[3] * faded);
        if let Some(id) = &self.sem {
            let r = l.whole;
            c.bind_to_layer(layer, id, [r.x, r.y, r.w, r.h]);
        }
        let surface = if self.focused().is_some() {
            &k.focus
        } else if self.hovered {
            &k.hover
        } else {
            &k.rest
        };
        let Rect { x, y, w, h } = l.whole;
        let radius = k.radius.min(h / 2.0);
        c.push(rounded_rect(x, y, w, h, radius, fade(surface.fill)));
        c.push(rounded_rect_stroke(
            x,
            y,
            w,
            h,
            radius,
            fade(surface.border),
            k.border_width,
        ));
        // The swatch: its base, the color over it at its opacity, its
        // border.
        let Rect { x, y, w, h } = l.swatch;
        let radius = k.swatch_radius.min(h / 2.0);
        c.push(rounded_rect(x, y, w, h, radius, fade(k.swatch_base)));
        c.push(rounded_rect(x, y, w, h, radius, fade(self.color())));
        c.push(rounded_rect_stroke(
            x,
            y,
            w,
            h,
            radius,
            fade(k.swatch_border),
            k.border_width,
        ));
        self.hex.render(c, l.hex, theme);
        let d = l.divider;
        c.push(SceneNode::Rect {
            x: d.x,
            y: d.y,
            w: d.w,
            h: d.h,
            color: fade(k.divider),
        });
        self.alpha.render(c, l.alpha, theme);
        let bare = &theme.form.bare;
        c.push(SceneNode::Text {
            key: TextNodeKey::from_style(PERCENT, &scale.run(bare.text), None),
            x: l.suffix.x,
            y: l.suffix.y + TextMeasurer::vertical_center(&scale.run(bare.text), l.suffix.h),
            color: fade(bare.text_color),
        });
    }

    /// Declares it as `id` (see the module docs). It only reads the
    /// widget: the owner runs `set_value` on a text ([`set_hex`],
    /// [`set_opacity`]).
    ///
    /// [`set_hex`]: Self::set_hex
    /// [`set_opacity`]: Self::set_opacity
    #[track_caller]
    pub fn project(
        &self,
        b: &mut SnapshotBuilder,
        id: &SemId,
        label: &str,
        parent: Option<&SemId>,
    ) {
        let mut node = b.node(id, Role::Group, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.state("value", self.value())
            .enabled(!self.is_disabled());
        if let Some(hex) = self.hex.sem_id() {
            self.hex.project(b, hex, "Hex", Some(id));
        }
        if let Some(alpha) = self.alpha.sem_id() {
            self.alpha
                .project(b, alpha, "Opacity", Some(id))
                .description("percent");
        }
    }
}
