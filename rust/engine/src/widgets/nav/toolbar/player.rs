//! The kit's video player (`components/VideoPlayer`): play and pause, the
//! timeline with what played and its head, the time, and repeat when the
//! player has it. On its own pill, or bare on a photo (`on_image`), where
//! its marks take the photo's tones. Every value is
//! `theme.navigation.tool.player`.

use accord::{Role, SemId, SnapshotBuilder};

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::styling::theme::Theme;
use crate::widgets::buttons::Press;
use crate::widgets::icons;
use crate::widgets::recipe::{rounded_rect, rounded_rect_stroke, shadow_drops};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

use super::super::tokens::PlayerTones;

/// Where each part of a player lands.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerLayout {
    pub play: Rect,
    pub track: Rect,
    pub time: Rect,
    pub repeat: Option<Rect>,
}

/// The video player.
#[derive(Clone, Debug)]
pub struct VideoPlayer {
    pub playing: bool,
    /// What played, 0 to 1.
    pub progress: f32,
    pub time: String,
    pub repeat: bool,
    pub on_image: bool,
    play: Press,
    repeat_press: Press,
    sem: Option<SemId>,
}

impl VideoPlayer {
    pub fn new(time: impl Into<String>, progress: f32) -> Self {
        Self {
            playing: false,
            progress: progress.clamp(0.0, 1.0),
            time: time.into(),
            repeat: false,
            on_image: false,
            play: Press::default(),
            repeat_press: Press::default(),
            sem: None,
        }
    }

    pub fn repeat(mut self, on: bool) -> Self {
        self.repeat = on;
        self
    }

    /// Bare, on a photo.
    pub fn on_image(mut self, on: bool) -> Self {
        self.on_image = on;
        self
    }

    /// The identity of the player; its play button is `<id>.play`, its
    /// repeat button `<id>.repeat`.
    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    fn tones(&self, theme: &Theme) -> PlayerTones {
        let p = &theme.navigation.tool.player;
        if self.on_image { p.on_image } else { p.solid }
    }

    /// The narrowest player: play, the track's least width, the time and
    /// repeat inside the paddings; as tall as the recipe.
    pub fn preferred_size(&self, theme: &Theme) -> (f32, f32) {
        let p = &theme.navigation.tool.player;
        let repeat = if self.repeat {
            p.repeat_gap + p.repeat - p.repeat_overhang
        } else {
            0.0
        };
        let w = p.pad_start
            + p.button
            + p.button_gap
            + p.track_min_w
            + p.time_gap
            + p.time_w
            + repeat
            + p.pad_end;
        (w, p.height)
    }

    /// The parts in a player as wide as `bounds` (the track takes what
    /// is left), as tall as the recipe, at `bounds`' top.
    pub fn layout(&self, bounds: Rect, theme: &Theme) -> PlayerLayout {
        let p = &theme.navigation.tool.player;
        let cy = bounds.y + p.height / 2.0;
        let play = Rect::new(
            bounds.x + p.pad_start,
            cy - p.button / 2.0,
            p.button,
            p.button,
        );
        let mut end = bounds.x + bounds.w - p.pad_end;
        let repeat = self.repeat.then(|| {
            end += p.repeat_overhang;
            let at = Rect::new(end - p.repeat, cy - p.repeat / 2.0, p.repeat, p.repeat);
            end = at.x - p.repeat_gap;
            at
        });
        let line = p.time.style(&theme.typography).line_height;
        let time = Rect::new(end - p.time_w, cy - line / 2.0, p.time_w, line);
        let track_x = play.x + play.w + p.button_gap;
        let track_w = (time.x - p.time_gap - track_x).max(0.0);
        let track = Rect::new(track_x, cy - p.track_h / 2.0, track_w, p.track_h);
        PlayerLayout {
            play,
            track,
            time,
            repeat,
        }
    }

    /// Hover and press on play and repeat; a click on play flips it.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        let at = self.layout(bounds, theme);
        let mut out = self.play.handle(event, at.play, false);
        if out.clicked {
            self.playing = !self.playing;
        }
        if let Some(r) = at.repeat {
            let rep = self.repeat_press.handle(event, r, false);
            out = EventResult {
                handled: out.handled || rep.handled,
                changed: out.changed || rep.changed,
                clicked: out.clicked || rep.clicked,
            };
        }
        out
    }

    pub fn render_to_layer(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        let p = &theme.navigation.tool.player;
        let t = self.tones(theme);
        let pill = Rect::new(bounds.x, bounds.y, bounds.w, p.height);
        if let Some(id) = &self.sem {
            c.bind_to_layer(layer, id, [pill.x, pill.y, pill.w, pill.h]);
        }
        if !self.on_image {
            shadow_drops(c, layer, pill, p.radius, &p.shadow);
            c.push_to_layer(
                layer,
                rounded_rect(pill.x, pill.y, pill.w, pill.h, p.radius, p.fill.0),
            );
            c.push_to_layer(
                layer,
                rounded_rect_stroke(
                    pill.x,
                    pill.y,
                    pill.w,
                    pill.h,
                    p.radius,
                    p.border.0,
                    p.border_width,
                ),
            );
        }
        let at = self.layout(bounds, theme);
        if let Some(id) = &self.sem {
            for (name, r) in [("play", Some(at.play)), ("repeat", at.repeat)] {
                if let (Ok(part), Some(r)) = (id.child(name), r) {
                    c.bind_to_layer(layer, &part, [r.x, r.y, r.w, r.h]);
                }
            }
        }
        let hot = self.play.hovered || self.play.pressed;
        let icon = if self.playing { "pause" } else { "play" };
        let tone = if hot { t.icon_hover } else { t.icon };
        let (x, y) = (
            at.play.x + (p.button - p.icon) / 2.0,
            at.play.y + (p.button - p.icon) / 2.0,
        );
        if let Some(node) = icons::icon_at(icon, p.icon, tone.0, x, y) {
            c.push_to_layer(layer, node);
        }
        let k = at.track;
        let radius = k.h / 2.0;
        c.push_to_layer(layer, rounded_rect(k.x, k.y, k.w, k.h, radius, t.track.0));
        let played = (k.w * self.progress).max(p.head_w);
        c.push_to_layer(
            layer,
            rounded_rect(k.x, k.y, played, k.h, radius, t.played.0),
        );
        let head_x = k.x + played - p.head_w;
        c.push_to_layer(
            layer,
            rounded_rect(head_x, k.y, p.head_w, k.h, radius, t.head.0),
        );
        let style = p.time.style(&theme.typography);
        c.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(&self.time, &style, None),
                x: at.time.x,
                y: at.time.y,
                color: t.time.0,
            },
        );
        if let Some(r) = at.repeat {
            let hot = self.repeat_press.hovered || self.repeat_press.pressed;
            let fill = if hot { t.repeat_hover } else { t.repeat_fill };
            c.push_to_layer(layer, rounded_rect(r.x, r.y, r.w, r.h, r.w / 2.0, fill.0));
            let (x, y) = (r.x + (r.w - p.icon) / 2.0, r.y + (r.h - p.icon) / 2.0);
            if let Some(node) = icons::icon_at("repeat", p.icon, t.icon.0, x, y) {
                c.push_to_layer(layer, node);
            }
        }
    }

    /// A `Role::Group` named `label`, its time and progress as its value;
    /// under it the play button (`playing`) and the repeat button.
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
        node.description(self.time.as_str());
        node.state("progress", f64::from(self.progress));
        if let Ok(play) = id.child("play") {
            let name = if self.playing { "Pause" } else { "Play" };
            let mut button = b.node(&play, Role::Toggle, name);
            button.parent(id);
            button.state("playing", self.playing);
            button.action("activate");
        }
        if self.repeat
            && let Ok(rep) = id.child("repeat")
        {
            let mut button = b.node(&rep, Role::Button, "Repeat");
            button.parent(id);
            button.action("activate");
        }
    }
}
