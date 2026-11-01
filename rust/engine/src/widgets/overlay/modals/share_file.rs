//! The shared file (`components/ShareFile`): who to invite and what they
//! may do (the kit's Invite, an input group with its select) beside the
//! invite button; the general access; the people with access, each with
//! what they can do (a minimal select) or the owner's mark, and a remove
//! at the row's end while the row is hovered; the link and its copy
//! button in the foot.
//!
//! Projected: a `Role::Group` named "Share file", the invite field
//! (`<id>.invite`, its select `<id>.invite.choice`), the invite button
//! (`<id>.send`), the general access as labels (`<id>.access[<i>]`), each
//! person's select (`<id>.person[<i>].access`) and remove
//! (`<id>.person[<i>].remove`), the owner as a label, the copy button
//! (`<id>.copy`).

use accord::{Role, SemId, SnapshotBuilder, Value};

use crate::drawing::compositor::{Compositor, LayerId, SceneNode};
use crate::styling::theme::Theme;
use crate::widgets::buttons::{Button, ButtonSize, ButtonVariant};
use crate::widgets::form::Select;
use crate::widgets::form::input_group::InputGroup;
use crate::widgets::form::tokens::{Look, SelectLook};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

use super::{
    Controls, Item, ItemRole, Kind, ModalBody, boxed, child, glyph, hairline, middle, panel_with,
    picture, raise, text, text_w, tokens,
};

/// The general access rows: the glyph, the title, the line.
const ACCESS: [(&str, &str, &str); 2] = [
    ("users", "Only those invited", "4 people"),
    ("link", "Link access", "Only users have shared the link"),
];

/// What a person may do.
const CAN: [&str; 2] = ["can view", "can edit"];

/// A person with access: the name, the email, what they can do (`None`
/// for the owner).
pub struct Person {
    pub name: String,
    pub email: String,
    pub can: Option<usize>,
}

impl Person {
    pub fn new(name: &str, email: &str, can: Option<usize>) -> Self {
        Self {
            name: name.into(),
            email: email.into(),
            can,
        }
    }
}

/// The invite field, the invite button, then a select and a remove per
/// person who is not the owner, then the copy button.
const INVITE: usize = 0;
const SEND: usize = 1;

pub struct ShareFile {
    pub link: String,
    people: Vec<Person>,
    controls: Controls,
    /// The person row under the pointer: its remove shows.
    hovered: Option<usize>,
    access: Option<SemId>,
}

/// Where the parts go, top to bottom.
struct Layout {
    controls: Vec<Rect>,
    rules: [f32; 3],
    access_label: f32,
    access_rows: Vec<Rect>,
    people_label: f32,
    person_rows: Vec<Rect>,
    foot: Rect,
}

impl ShareFile {
    pub fn new(people: Vec<Person>, link: &str, id: Option<&SemId>) -> Self {
        let mut controls = Controls::default();
        let invite = InputGroup::new("Email, name...")
            .look(Look::Inset)
            .end_select(Select::new(CAN, 0));
        controls.push(
            Kind::Group(invite, "Invite by email or name".into()),
            child(id, "invite"),
        );
        let send = Button::new("Invite").variant(ButtonVariant::Secondary);
        controls.push(Kind::Button(send), child(id, "send"));
        let family = child(id, "person");
        for (i, p) in people.iter().enumerate() {
            let Some(can) = p.can else {
                continue;
            };
            let row = family.as_ref().map(|f| f.key(i.to_string()));
            let select = Select::new(CAN, can).look(SelectLook::Minimal).small(true);
            let label = format!("What {} can do", p.name);
            controls.push(Kind::Select(select, label), child(row.as_ref(), "access"));
            let remove = Item::new(format!("Remove {}", p.name), ItemRole::Button);
            controls.push(Kind::Item(remove), child(row.as_ref(), "remove"));
        }
        let copy = Button::new("Copy link")
            .variant(ButtonVariant::Primary)
            .size(ButtonSize::Sm)
            .icon("link");
        controls.push(Kind::Button(copy), child(id, "copy"));
        Self {
            link: link.into(),
            people,
            controls,
            hovered: None,
            access: child(id, "access"),
        }
    }

    /// The control index of person `i`'s select (its remove follows).
    fn person_control(&self, i: usize) -> Option<usize> {
        self.people.get(i)?.can?;
        let before = self.people[..i].iter().filter(|p| p.can.is_some()).count();
        Some(2 + before * 2)
    }

    fn row_h(theme: &Theme) -> f32 {
        let scale = &theme.typography;
        let f = &tokens(theme).share_file;
        let lines = scale.run(f.access.title).line_height + scale.run(f.access.line).line_height;
        let a = &f.access;
        a.box_size.max(lines) + a.row_pad_y * 2.0
    }

    fn person_h(theme: &Theme) -> f32 {
        let scale = &theme.typography;
        let f = &tokens(theme).share_file;
        let lines = scale.run(f.access.title).line_height + scale.run(f.access.line).line_height;
        f.person
            .avatar
            .max(lines)
            .max(theme.form.select.small_height)
            + f.person.pad_y * 2.0
    }

    fn label_h(theme: &Theme) -> f32 {
        let f = &tokens(theme).share_file;
        theme.typography.run(f.label).line_height + f.label_pad_y * 2.0
    }

    fn layout(&self, rect: Rect, theme: &Theme) -> Layout {
        let f = &tokens(theme).share_file;
        let ink = &tokens(theme).ink;
        let (x, w) = (rect.x + f.pad, rect.w - f.pad * 2.0);
        let mut controls = vec![Rect::new(0.0, 0.0, 0.0, 0.0); self.controls.0.len()];
        // The invite row.
        let (send_w, send_h) = match &self.controls.0[SEND].kind {
            Kind::Button(b) => b.preferred_size(theme),
            _ => (0.0, 0.0),
        };
        let group_h = match &self.controls.0[INVITE].kind {
            Kind::Group(g, _) => g.height(theme),
            _ => send_h,
        };
        let row_h = group_h.max(send_h);
        let y = rect.y + f.pad;
        let group_w = (w - f.invite_gap - send_w).max(0.0);
        controls[INVITE] = Rect::new(x, y + (row_h - group_h) / 2.0, group_w, group_h);
        controls[SEND] = Rect::new(
            x + group_w + f.invite_gap,
            y + (row_h - send_h) / 2.0,
            send_w,
            send_h,
        );
        // The general access.
        let rule0 = y + row_h + f.pad;
        let access_label = rule0 + ink.rule_w + f.section_pad_y;
        let mut at = access_label + Self::label_h(theme);
        let mut access_rows = Vec::new();
        for _ in ACCESS {
            access_rows.push(Rect::new(x, at, w, Self::row_h(theme)));
            at += Self::row_h(theme);
        }
        // The people.
        let rule1 = at + f.section_pad_y;
        let people_label = rule1 + ink.rule_w + f.section_pad_y;
        at = people_label + Self::label_h(theme);
        let mut person_rows = Vec::new();
        let ph = Self::person_h(theme);
        for (i, _) in self.people.iter().enumerate() {
            let row = Rect::new(rect.x, at, rect.w, ph);
            if let Some(k) = self.person_control(i) {
                let select = match &self.controls.0[k].kind {
                    Kind::Select(s, _) => s,
                    _ => continue,
                };
                let sw = select.width_in(theme).max(f.person.select_w);
                let sh = select.height_in(theme);
                controls[k] = Rect::new(row.right() - f.pad - sw, at + (ph - sh) / 2.0, sw, sh);
                let p = &f.person;
                controls[k + 1] = Rect::new(row.right() - p.remove_w, at, p.remove_w, ph);
            }
            person_rows.push(row);
            at += ph;
        }
        // The foot.
        let rule2 = at + f.section_pad_y;
        let last = self.controls.0.len() - 1;
        let (copy_w, copy_h) = match &self.controls.0[last].kind {
            Kind::Button(b) => b.preferred_size(theme),
            _ => (0.0, 0.0),
        };
        let foot = Rect::new(rect.x, rule2 + ink.rule_w, rect.w, copy_h + f.pad * 2.0);
        controls[last] = Rect::new(
            foot.right() - f.pad - copy_w,
            foot.y + f.pad,
            copy_w,
            copy_h,
        );
        Layout {
            controls,
            rules: [rule0, rule1, rule2],
            access_label,
            access_rows,
            people_label,
            person_rows,
            foot,
        }
    }

    fn draw_person(&self, c: &mut Compositor, i: usize, row: Rect, select_x: f32, theme: &Theme) {
        let scale = &theme.typography;
        let t = tokens(theme);
        let (f, ink) = (&t.share_file, &t.ink);
        let (p, a) = (&f.person, &f.access);
        let person = &self.people[i];
        let x = row.x + f.pad;
        let avatar = Rect::new(x, row.y + (row.h - p.avatar) / 2.0, p.avatar, p.avatar);
        let colors = t.pictures.avatars[(i + 1) % t.pictures.avatars.len()];
        picture(c, avatar, p.avatar / 2.0, colors);
        let tx = avatar.right() + a.gap;
        let lines = scale.run(a.title).line_height + scale.run(a.line).line_height;
        let ty = row.y + (row.h - lines) / 2.0;
        let room = (select_x - a.gap - tx).max(0.0);
        text(c, &person.name, &scale.run(a.title), tx, ty, room, ink.ink);
        text(
            c,
            &person.email,
            &scale.run(a.line),
            tx,
            ty + scale.run(a.title).line_height,
            room,
            ink.muted,
        );
    }
}

impl ModalBody for ShareFile {
    fn title(&self) -> &str {
        "Share file"
    }

    fn size(&self, w: f32, theme: &Theme) -> (f32, f32) {
        let w = tokens(theme).share_file.panel.width.min(w);
        let l = self.layout(Rect::new(0.0, 0.0, w, 0.0), theme);
        (w, l.foot.bottom())
    }

    fn handle_event(&mut self, event: &WidgetEvent, rect: Rect, theme: &Theme) -> EventResult {
        let l = self.layout(rect, theme);
        let mut r = EventResult::IGNORED;
        if let WidgetEvent::MouseMove { x, y } = *event {
            let over = l.person_rows.iter().position(|row| row.contains(x, y));
            let over = over.filter(|i| self.people[*i].can.is_some());
            if over != self.hovered {
                self.hovered = over;
                r = EventResult::changed();
            }
        }
        r.merge(self.controls.handle_event(event, &l.controls, theme).1)
    }

    fn render(&self, c: &mut Compositor, rect: Rect, theme: &Theme) {
        let scale = &theme.typography;
        let t = tokens(theme);
        let (f, ink) = (&t.share_file, &t.ink);
        let l = self.layout(rect, theme);
        // The foot's band to the sheet's bottom corners, under its edge.
        let foot = l.foot;
        panel_with(c, rect, &f.panel, theme, |c| {
            c.push(SceneNode::PushClip {
                x: foot.x,
                y: foot.y,
                w: foot.w,
                h: foot.h,
            });
            boxed(c, rect, f.panel.radius, ink.well, None);
            c.push(SceneNode::PopClip);
        });
        for y in l.rules {
            hairline(c, rect.x, y, rect.w, ink.rule_w, ink.rule);
        }
        let x = rect.x + f.pad;
        let w = rect.w - f.pad * 2.0;
        let label_y = |top: f32| top + f.label_pad_y;
        text(
            c,
            "General access",
            &scale.run(f.label),
            x,
            label_y(l.access_label),
            w,
            ink.muted,
        );
        text(
            c,
            "People with access",
            &scale.run(f.label),
            x,
            label_y(l.people_label),
            w,
            ink.muted,
        );
        let a = &f.access;
        for ((icon, title, line), row) in ACCESS.iter().zip(&l.access_rows) {
            let b = Rect::new(
                row.x,
                row.y + (row.h - a.box_size) / 2.0,
                a.box_size,
                a.box_size,
            );
            raise(c, b, a.box_radius, &ink.raised);
            boxed(c, b, a.box_radius, ink.well, Some((ink.edge, ink.rule_w)));
            glyph(c, icon, a.icon, ink.ink, b);
            let tx = b.right() + a.gap;
            let lines = scale.run(a.title).line_height + scale.run(a.line).line_height;
            let ty = row.y + (row.h - lines) / 2.0;
            text(
                c,
                title,
                &scale.run(a.title),
                tx,
                ty,
                row.right() - tx,
                ink.ink,
            );
            text(
                c,
                line,
                &scale.run(a.line),
                tx,
                ty + scale.run(a.title).line_height,
                row.right() - tx,
                ink.muted,
            );
        }
        let p = &f.person;
        for (i, row) in l.person_rows.iter().enumerate() {
            let select_x = match self.person_control(i) {
                Some(k) => l.controls[k].x,
                None => {
                    // The owner: the word and the shield at the row's end.
                    let owner_w = text_w("Owner", &scale.run(a.title)) + a.gap + p.shield;
                    let end = row.right() - f.pad - p.owner_pad_x;
                    let ox = end - owner_w;
                    text(
                        c,
                        "Owner",
                        &scale.run(a.title),
                        ox,
                        middle(&scale.run(a.title), row.y, row.h),
                        owner_w,
                        ink.ink,
                    );
                    let shield = Rect::new(end - p.shield, row.y, p.shield, row.h);
                    glyph(c, "shield-check", p.shield, p.owner, shield);
                    ox
                }
            };
            self.draw_person(c, i, *row, select_x, theme);
        }
        // The link, faded in the kit, in the secondary text here.
        let last = l.controls.len() - 1;
        let copy = l.controls[last];
        let room = copy.x - f.foot_gap - x;
        text(
            c,
            &self.link,
            &scale.run(f.link),
            x,
            middle(&scale.run(f.link), l.foot.y, l.foot.h),
            room,
            ink.muted,
        );
        self.controls.render(c, &l.controls, theme);
        // A hovered or focused row's remove, over its row's end.
        for (i, _) in self.people.iter().enumerate() {
            let Some(k) = self.person_control(i).map(|k| k + 1) else {
                continue;
            };
            if self.hovered != Some(i) && !self.controls.0[k].is_focused() {
                continue;
            }
            let r = l.controls[k];
            // `rounded-l-lg`: the end corners square under the clip.
            c.push(SceneNode::PushClip {
                x: r.x,
                y: r.y,
                w: r.w,
                h: r.h,
            });
            let wide = Rect::new(r.x, r.y, r.w + p.remove_radius, r.h);
            boxed(c, wide, p.remove_radius, p.remove_fill, None);
            c.push(SceneNode::PopClip);
            glyph(c, "x", p.remove_icon, p.remove_ink, r);
            if self.controls.0[k].is_focused() {
                c.push(crate::widgets::recipe::focus_ring_inside(
                    r,
                    p.remove_radius,
                    theme,
                ));
            }
        }
    }

    fn render_overlay(&self, c: &mut Compositor, layer: LayerId, rect: Rect, theme: &Theme) {
        let l = self.layout(rect, theme);
        self.controls.render_overlay(c, layer, &l.controls, theme);
    }

    fn project(&self, b: &mut SnapshotBuilder, id: &SemId, parent: &SemId) {
        b.node(id, Role::Group, self.title())
            .parent(parent)
            .description(self.link.as_str());
        if let Some(access) = &self.access {
            b.node(access, Role::Group, "General access").parent(id);
            for (i, (_, title, line)) in ACCESS.iter().enumerate() {
                b.node(access.key(i.to_string()), Role::Label, *title)
                    .parent(access)
                    .description(*line);
            }
        }
        if let Some(family) = child(Some(id), "person") {
            for (i, p) in self.people.iter().enumerate() {
                let row = family.key(i.to_string());
                let mut node = b.node(&row, Role::Group, p.name.as_str());
                node.parent(id).description(p.email.as_str());
                if p.can.is_none() {
                    node.state("owner", true);
                }
            }
        }
        // The controls of a person's row hang from the row.
        for control in &self.controls.0 {
            let Some(cid) = control.id() else {
                continue;
            };
            let row = cid
                .parent_path()
                .filter(|p| p.as_str().contains(".person["));
            let parent = row.unwrap_or_else(|| id.clone());
            control.project(b, &parent);
        }
    }

    fn run(
        &mut self,
        target: &SemId,
        action: &str,
        param: Option<&Value>,
    ) -> Option<Result<bool, String>> {
        self.controls.run(target, action, param).map(|(_, r)| r)
    }

    fn focus(&mut self, id: Option<&SemId>) -> bool {
        self.controls.focus(id)
    }

    fn has_open(&self) -> bool {
        self.controls.any_open()
    }

    fn text_mut(&mut self) -> Option<&mut crate::widgets::form::TextField> {
        self.controls.text_mut()
    }

    fn ime_area(&self, rect: Rect, theme: &Theme) -> Option<Rect> {
        self.controls
            .ime_area(&self.layout(rect, theme).controls, theme)
    }

    fn escape_text(&mut self) -> bool {
        self.controls.escape()
    }

    fn end_compose(&mut self) -> bool {
        self.controls.end_compose()
    }

    fn close(&mut self) -> bool {
        self.controls.close()
    }
}
