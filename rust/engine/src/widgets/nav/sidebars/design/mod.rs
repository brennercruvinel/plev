//! The kit's DesignAndAnimationSidebar
//! (`components/DesignAndAnimationSidebar`): a rounded canvas panel. The
//! head (who is here, the Share button), the Design and Animation tabs,
//! then the tab's groups (`Group`): a title, at its end an add button, a
//! switch or the loop's repeat, and its body.
//!
//! Design: Show Frame (a switch), Artboard (the size select, the width
//! and height fields), Materials and Styles (tiles), Background (the
//! ColorPicker of front A: a named stand-in until it is in the engine,
//! TODO(task-104a)), Camera (Isometric or Perspective, the distortion's
//! range and value). Animation: Motion Blur (a switch), Loop (the strip
//! with the looped span, Short or Long, the span's length), Effects
//! (tiles), Lens (three selects, each with its eye). Pictures are their
//! stand-in gradient; selects, switches and tabs are the form's and the
//! navigation's own.
//!
//! With a [`SemId`]: `<id>.share`, `<id>.tabs`, a group `<id>.block[<b>]`
//! with its `.switch`, `.select[<i>]`, `.tabs` and `.tile[<i>]`.

mod paint;

use accord::{Role, SemId, SnapshotBuilder};

use crate::styling::theme::Theme;
use crate::widgets::buttons::Button;
use crate::widgets::form::Select;
use crate::widgets::form::Switch;
use crate::widgets::form::tokens::SelectLook;
use crate::widgets::nav::Tabs;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// The design tab or the animation's.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DesignTab {
    #[default]
    Design,
    Animation,
}

/// What a group's title has at its end.
#[derive(Clone, Debug)]
pub enum End {
    Add,
    Switch(Switch),
    Repeat,
}

/// What a group shows under its title.
#[derive(Clone, Debug)]
pub enum Body {
    None,
    Artboard(Select, [String; 2]),
    Tiles {
        n: usize,
        cols: usize,
        well: bool,
        small: bool,
        picked: Option<usize>,
    },
    Picker,
    Camera(Tabs, f32),
    Loop(Tabs, [f32; 3]),
    Lens(Vec<Select>),
}

#[derive(Clone, Debug)]
pub struct Block {
    pub title: String,
    pub tab: DesignTab,
    pub end: End,
    pub body: Body,
}

/// A group's rects: its title row, its end control, its body.
#[derive(Clone, Copy, Debug)]
pub struct BlockLayout {
    pub index: usize,
    pub header: Rect,
    pub end: Rect,
    pub body: Rect,
}

#[derive(Clone, Debug)]
pub struct DesignSidebar {
    pub tabs: Tabs,
    pub share: Button,
    pub blocks: Vec<Block>,
    focused: Option<SemId>,
    sem: Option<SemId>,
}

fn block(title: &str, tab: DesignTab, end: End, body: Body) -> Block {
    Block {
        title: title.into(),
        tab,
        end,
        body,
    }
}

fn tiles(n: usize, cols: usize, well: bool, small: bool) -> Body {
    Body::Tiles {
        n,
        cols,
        well,
        small,
        picked: None,
    }
}

impl DesignSidebar {
    /// The kit's groups, its values as it starts.
    pub fn kit() -> Self {
        use DesignTab::{Animation, Design};
        let select = |options: &[&str], icon| {
            Select::new(options.iter().copied(), 0)
                .look(SelectLook::Ghost)
                .icon(icon)
        };
        let lens = vec![
            select(
                &[
                    "Wide-Angle Lens",
                    "Standard Lens",
                    "Telephoto Lens",
                    "Macro Lens",
                    "Fish-Eye Lens",
                ],
                "camera",
            ),
            select(&["Zoom out", "Zoom in"], "zoom-in").look(SelectLook::Filled),
            select(&["Rotate around", "Rotate 180°", "Rotate 90°"], "rotate-cw")
                .look(SelectLook::Filled),
        ];
        let sizes = ["800x600", "1024x768", "1280x1024", "1600x1200", "1920x1080"];
        let artboard = select(&sizes, "frame").label("X Post");
        let blocks = vec![
            block(
                "Show Frame",
                Design,
                End::Switch(Switch::new(true)),
                Body::None,
            ),
            block(
                "Artboard",
                Design,
                End::Add,
                Body::Artboard(artboard, ["800".into(), "600".into()]),
            ),
            block("Materials", Design, End::Add, tiles(4, 2, true, false)),
            block("Styles", Design, End::Add, tiles(2, 2, false, false)),
            block("Background", Design, End::Add, Body::Picker),
            block(
                "Camera",
                Design,
                End::Add,
                Body::Camera(Tabs::new(["Isometric", "Perspective"]), 0.283),
            ),
            block(
                "Motion Blur",
                Animation,
                End::Switch(Switch::new(true)),
                Body::None,
            ),
            block(
                "Loop",
                Animation,
                End::Repeat,
                Body::Loop(Tabs::new(["Short", "Long"]), [6.0, 14.0, 20.0]),
            ),
            block("Effects", Animation, End::Add, tiles(6, 3, false, true)),
            block("Lens", Animation, End::Add, Body::Lens(lens)),
        ];
        Self {
            tabs: Tabs::new(["Design", "Animation"]),
            share: Button::new("Share"),
            blocks,
            focused: None,
            sem: None,
        }
    }

    pub fn tab(mut self, tab: DesignTab) -> Self {
        self.tabs.active = tab as usize;
        self
    }

    pub fn current_tab(&self) -> DesignTab {
        if self.tabs.active == 1 {
            DesignTab::Animation
        } else {
            DesignTab::Design
        }
    }

    pub fn sem(mut self, id: SemId) -> Self {
        let child = |at: &SemId, name: &str| at.child(name).ok();
        if let Some(s) = child(&id, "share") {
            self.share = self.share.clone().sem(s);
        }
        if let Some(t) = child(&id, "tabs") {
            self.tabs = self.tabs.clone().sem(t);
        }
        for (b, block) in self.blocks.iter_mut().enumerate() {
            let Some(at) = child(&id, "block").map(|f| f.key(b.to_string())) else {
                continue;
            };
            if let (End::Switch(s), Some(sid)) = (&mut block.end, child(&at, "switch")) {
                *s = s.clone().sem(sid);
            }
            let selects = child(&at, "select");
            let select_id = |i: usize| selects.as_ref().map(|f| f.key(i.to_string()));
            match &mut block.body {
                Body::Artboard(s, _) => {
                    if let Some(sid) = select_id(0) {
                        *s = s.clone().sem(sid);
                    }
                }
                Body::Lens(list) => {
                    for (i, s) in list.iter_mut().enumerate() {
                        if let Some(sid) = select_id(i) {
                            *s = s.clone().sem(sid);
                        }
                    }
                }
                Body::Camera(t, _) | Body::Loop(t, _) => {
                    if let Some(tid) = child(&at, "tabs") {
                        *t = t.clone().sem(tid);
                    }
                }
                _ => {}
            }
        }
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    /// `<id>.block[<b>].tile[<i>]`.
    pub fn tile_id(&self, b: usize, i: usize) -> Option<SemId> {
        let at = self.sem.as_ref()?.child("block").ok()?.key(b.to_string());
        Some(at.child("tile").ok()?.key(i.to_string()))
    }

    fn body_h(&self, body: &Body, w: f32, theme: &Theme) -> f32 {
        let d = &theme.navigation.sidebars.canvas.design;
        match body {
            Body::None => 0.0,
            Body::Artboard(s, _) => s.height_in(theme) + d.gap + d.field_h,
            Body::Tiles { n, cols, .. } => {
                let cols = *cols as f32;
                let cell = (w - (cols - 1.0) * d.gap) / cols;
                let rows = (*n as f32 / cols).ceil();
                rows * cell + (rows - 1.0).max(0.0) * d.gap
            }
            Body::Picker => d.field_h * 2.0,
            Body::Camera(..) => {
                let label = d.field_text.line_height;
                Tabs::height(theme)
                    + theme.navigation.sidebars.canvas.band_pad_y
                    + label
                    + d.gap
                    + d.field_h
            }
            Body::Loop(..) => d.strip_h + d.gap + d.field_h.max(Tabs::height(theme)),
            Body::Lens(list) => {
                let n = list.len() as f32;
                list.iter().map(|s| s.height_in(theme)).sum::<f32>()
                    + (n - 1.0).max(0.0) * d.stack_gap
            }
        }
    }

    /// The head's height: the avatars in its padding.
    fn head_h(&self, theme: &Theme) -> f32 {
        let d = &theme.navigation.sidebars.canvas.design;
        d.head_pad * 2.0 + d.avatar.max(self.share.preferred_size(theme).1)
    }

    /// The tab band's rect and the top of the groups.
    fn band(&self, bounds: Rect, theme: &Theme) -> (Rect, f32) {
        let t = &theme.navigation.sidebars;
        let c = &t.canvas;
        let y = bounds.y + self.head_h(theme) + t.edge_width + c.band_pad_y;
        let tabs = Rect::new(
            bounds.x + c.pad,
            y,
            bounds.w - c.pad * 2.0,
            Tabs::height(theme),
        );
        (tabs, tabs.bottom() + c.band_pad_y + t.edge_width)
    }

    pub fn share_rect(&self, bounds: Rect, theme: &Theme) -> Rect {
        let d = &theme.navigation.sidebars.canvas.design;
        let (w, h) = self.share.preferred_size(theme);
        let y = bounds.y + (self.head_h(theme) - h) / 2.0;
        Rect::new(bounds.right() - d.head_pad - w, y, w, h)
    }

    /// The groups of the current tab, down from the band.
    pub fn layout(&self, bounds: Rect, theme: &Theme) -> Vec<BlockLayout> {
        let c = &theme.navigation.sidebars.canvas;
        let d = &c.design;
        let (_, mut y) = self.band(bounds, theme);
        let (x, w) = (bounds.x + c.pad, bounds.w - c.pad * 2.0);
        let tab = self.current_tab();
        let mut out = Vec::new();
        for (index, block) in self.blocks.iter().enumerate().filter(|(_, b)| b.tab == tab) {
            let header = Rect::new(bounds.x, y, bounds.w, d.header_h);
            let end = match &block.end {
                End::Switch(_) => {
                    let [sw, sh] = theme.control.switch_size;
                    Rect::new(x + w - sw, y + (d.header_h - sh) / 2.0, sw, sh)
                }
                End::Add | End::Repeat => {
                    Rect::new(x + w - d.add, y + (d.header_h - d.add) / 2.0, d.add, d.add)
                }
            };
            let h = self.body_h(&block.body, w, theme);
            let body = Rect::new(x, header.bottom(), w, h);
            y = body.bottom() + if h > 0.0 { c.pad } else { 0.0 };
            out.push(BlockLayout {
                index,
                header,
                end,
                body,
            });
        }
        out
    }

    pub fn size(&self, theme: &Theme) -> (f32, f32) {
        let t = &theme.navigation.sidebars;
        let bounds = Rect::new(0.0, 0.0, t.width, 0.0);
        let end = self.layout(bounds, theme).last().map_or(0.0, |l| {
            l.body.bottom() + if l.body.h > 0.0 { t.canvas.pad } else { 0.0 }
        });
        (t.width, end.max(t.min_h))
    }

    /// The selects of the current tab with their rects, for the events
    /// and for the open list drawn over everything.
    fn select_rects(&self, bounds: Rect, theme: &Theme) -> Vec<(usize, usize, Rect)> {
        let d = &theme.navigation.sidebars.canvas.design;
        let mut out = Vec::new();
        for l in self.layout(bounds, theme) {
            let w = l.body.w - d.gap - d.add;
            match &self.blocks[l.index].body {
                Body::Artboard(s, _) => out.push((
                    l.index,
                    0,
                    Rect::new(l.body.x, l.body.y, w, s.height_in(theme)),
                )),
                Body::Lens(list) => {
                    let mut y = l.body.y;
                    for (i, s) in list.iter().enumerate() {
                        out.push((l.index, i, Rect::new(l.body.x, y, w, s.height_in(theme))));
                        y += s.height_in(theme) + d.stack_gap;
                    }
                }
                _ => {}
            }
        }
        out
    }

    fn select_mut(&mut self, b: usize, i: usize) -> Option<&mut Select> {
        match &mut self.blocks[b].body {
            Body::Artboard(s, _) if i == 0 => Some(s),
            Body::Lens(list) => list.get_mut(i),
            _ => None,
        }
    }

    /// The inner tabs' rect of a camera or loop group.
    fn inner_tabs(body: &Body, r: Rect, theme: &Theme) -> Option<Rect> {
        let d = &theme.navigation.sidebars.canvas.design;
        match body {
            Body::Camera(..) => Some(Rect::new(r.x, r.y, r.w, Tabs::height(theme))),
            Body::Loop(..) => {
                let y = r.y + d.strip_h + d.gap;
                Some(Rect::new(
                    r.x,
                    y,
                    r.w - d.chip_w_narrow - d.stack_gap,
                    Tabs::height(theme),
                ))
            }
            _ => None,
        }
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        let mut moving = false;
        for block in &mut self.blocks {
            if let End::Switch(s) = &mut block.end {
                moving |= s.tick(dt);
            }
        }
        moving
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        // An open list has every event.
        for (b, i, r) in self.select_rects(bounds, theme) {
            if let Some(s) = self.select_mut(b, i)
                && s.is_open()
            {
                return s.handle_event(event, r, theme);
            }
        }
        let (tabs, _) = self.band(bounds, theme);
        let mut out = self.tabs.handle_event(event, tabs, theme);
        let share = self.share_rect(bounds, theme);
        out = out.merge(self.share.handle_event(event, share));
        for (b, i, r) in self.select_rects(bounds, theme) {
            if let Some(s) = self.select_mut(b, i) {
                out = out.merge(s.handle_event(event, r, theme));
            }
        }
        let (x, y) = event.pos();
        for l in self.layout(bounds, theme) {
            let block = &mut self.blocks[l.index];
            if let End::Switch(s) = &mut block.end {
                out = out.merge(s.handle_event(event, l.end));
            }
            if let Some(r) = Self::inner_tabs(&block.body, l.body, theme)
                && let Body::Camera(t, _) | Body::Loop(t, _) = &mut block.body
            {
                out = out.merge(t.handle_event(event, r, theme));
            }
            if let Body::Tiles {
                n, cols, picked, ..
            } = &mut block.body
                && matches!(event, WidgetEvent::MouseDown { .. })
                && let Some(i) = Self::cells(l.body, *n, *cols, theme)
                    .iter()
                    .position(|c| c.contains(x, y))
            {
                *picked = Some(i);
                out = out.merge(EventResult::clicked());
            }
        }
        out
    }

    /// Keyboard focus on the part `id` names, or none; `true` when it
    /// names one of them.
    pub fn focus_id(&mut self, id: Option<&SemId>) -> bool {
        let is = |own: Option<&SemId>| id.is_some() && own == id;
        let mut found = false;
        let on = is(self.share.sem_id());
        self.share.set_focused(on);
        found |= on;
        let on = is(self.tabs.sem_id());
        self.tabs.set_focused(on);
        found |= on;
        for block in &mut self.blocks {
            if let End::Switch(s) = &mut block.end {
                let on = is(s.sem_id());
                s.set_focused(on);
                found |= on;
            }
            match &mut block.body {
                Body::Artboard(s, _) => {
                    let on = is(s.sem_id());
                    s.set_focused(on);
                    found |= on;
                }
                Body::Lens(list) => {
                    for s in list {
                        let on = is(s.sem_id());
                        s.set_focused(on);
                        found |= on;
                    }
                }
                Body::Camera(t, _) | Body::Loop(t, _) => {
                    let on = is(t.sem_id());
                    t.set_focused(on);
                    found |= on;
                }
                _ => {}
            }
        }
        let tile = id.is_some_and(|i| self.tile_at_id(i).is_some());
        self.focused = if tile { id.cloned() } else { None };
        found || tile
    }

    /// The tile `id` names: its group and index.
    pub fn tile_at_id(&self, id: &SemId) -> Option<(usize, usize)> {
        self.blocks
            .iter()
            .enumerate()
            .find_map(|(b, block)| match block.body {
                Body::Tiles { n, .. } => (0..n)
                    .find(|i| self.tile_id(b, *i).as_ref() == Some(id))
                    .map(|i| (b, i)),
                _ => None,
            })
    }

    /// What a reader's action does to the part `id` names: a switch
    /// flips, a tile is picked, a tab is chosen (`<tabs>[<i>]`).
    pub fn activate(&mut self, id: &SemId) -> bool {
        if let Some((b, i)) = self.tile_at_id(id)
            && let Body::Tiles { picked, .. } = &mut self.blocks[b].body
        {
            *picked = Some(i);
            return true;
        }
        let pick = |t: &mut Tabs| {
            let Some(own) = t.sem_id() else { return false };
            let index = id
                .as_str()
                .strip_prefix(own.as_str())
                .and_then(|r| r.strip_prefix('['))
                .and_then(|r| r.strip_suffix(']'))
                .and_then(|n| n.parse::<usize>().ok());
            match index {
                Some(i) if i < t.labels.len() => {
                    t.active = i;
                    true
                }
                _ => false,
            }
        };
        if pick(&mut self.tabs) {
            return true;
        }
        for block in &mut self.blocks {
            if let End::Switch(s) = &mut block.end
                && s.sem_id() == Some(id)
            {
                s.toggle();
                return true;
            }
            if let Body::Camera(t, _) | Body::Loop(t, _) = &mut block.body
                && pick(t)
            {
                return true;
            }
        }
        false
    }

    /// A `Role::Region` named `label`: the Share button, the tabs, and
    /// per group of the current tab a `Role::Group` with its switch,
    /// selects, tabs and tiles.
    #[track_caller]
    pub fn project(
        &self,
        b: &mut SnapshotBuilder,
        id: &SemId,
        label: &str,
        parent: Option<&SemId>,
    ) {
        let mut node = b.node(id, Role::Region, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        if let Some(at) = self.share.sem_id() {
            let mut node = b.node(at, Role::Button, "Share");
            node.parent(id).action("activate");
        }
        if let Some(at) = self.tabs.sem_id() {
            self.tabs.project(b, at, "Design or animation", Some(id));
        }
        let tab = self.current_tab();
        for (bi, block) in self.blocks.iter().enumerate().filter(|(_, b)| b.tab == tab) {
            let Some(at) = id.child("block").ok().map(|f| f.key(bi.to_string())) else {
                continue;
            };
            b.node(&at, Role::Group, block.title.as_str()).parent(id);
            if let End::Switch(s) = &block.end
                && let Some(sid) = s.sem_id()
            {
                s.project(b, sid, block.title.as_str(), Some(&at));
            }
            match &block.body {
                Body::Artboard(s, _) => {
                    if let Some(sid) = s.sem_id() {
                        s.project(b, sid, "Artboard size", Some(&at));
                    }
                }
                Body::Lens(list) => {
                    for s in list {
                        if let Some(sid) = s.sem_id() {
                            s.project(b, sid, s.selected_label().unwrap_or("Lens"), Some(&at));
                        }
                    }
                }
                Body::Camera(t, _) | Body::Loop(t, _) => {
                    if let Some(tid) = t.sem_id() {
                        t.project(b, tid, block.title.as_str(), Some(&at));
                    }
                }
                Body::Tiles { n, picked, .. } => {
                    for i in 0..*n {
                        let Some(tid) = self.tile_id(bi, i) else {
                            continue;
                        };
                        let name = format!("{} {}", block.title, i + 1);
                        let mut node = b.node(&tid, Role::Button, name);
                        node.parent(&at).state("selected", *picked == Some(i));
                        node.action("activate");
                    }
                }
                _ => {}
            }
        }
    }
}
