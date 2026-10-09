//! Stage 2 (figma, task-71 phase 2): a Figma export -> ParserNode tree,
//! droplist and notes, the same `Resolution` the other resolvers give, so
//! the code emitter and the plev IR emitter read it unchanged.
//!
//! The screen is the first layer under the document's page and section. A
//! frame with auto layout is a stack (`layoutMode` its axis, `itemSpacing`
//! its gap, `padding*` its padding, the counter alignment its align); a
//! size is fixed only where the layer fixes it (not where it hugs, grows or
//! stretches); a text is a run with its style; a solid fill is a color.
//! Everything goes to the builder props the other sources use, so the IR
//! emitter measures it against the theme's tokens by the same rule.
//!
//! A frame with no auto layout places its children by position, which the
//! IR does not have. The rule: when its children do not overlap along y,
//! they are a column in y order (along x, a row in x order), and each
//! one's offset from where the stack puts it is reported as approximated,
//! with the difference in px; when they overlap, their positions are
//! impossible and they stay in source order. A vector, an ellipse, a
//! hidden layer, an effect, an override the export could not apply: each
//! is an entry at its line.

use crate::ParserError;
use crate::emit::plevir::sanitize;
use crate::formats::figma::parse::{FigDoc, FigNode};
use crate::tree::{
    Dropped, Note, NoteKind, Origin, ParserNode, Prop, Resolution, Src, Tag, TextValue, snake_case,
};

const HIDDEN: &str = "a hidden layer draws nothing: left out, as the design shows it";
const VECTOR: &str =
    "a vector drawing has no IR component (the IR has no icon or shape): the layer is lost";
const WRAPPER: &str = "the page and the section hold the screen; the screen is the surface";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Axis {
    Across,
    Down,
}

/// What a layer needs from the frame it is in.
struct Parent {
    auto: Option<Axis>,
    /// Every child stretches across: the frame's align carries it.
    stretch_all: bool,
    root: bool,
}

/// A layer that became a node, with its bounds and the construct of its
/// position (decided by the frame it is in).
struct Kid {
    node: ParserNode,
    bounds: [f32; 4],
    at: Src,
}

pub(crate) struct Ctx<'a> {
    file: &'a str,
    pub(super) mapped: usize,
    dropped: Vec<Dropped>,
    notes: Vec<Note>,
}

pub fn resolve_figma(doc: &FigDoc, file: &str) -> Result<Resolution, ParserError> {
    let mut cx = Ctx {
        file,
        mapped: 0,
        dropped: Vec::new(),
        notes: Vec::new(),
    };
    let screen = cx
        .screen(&doc.document)
        .ok_or_else(|| ParserError::Parse("figma json: no screen under the document".into()))?;
    let top = Parent {
        auto: None,
        stretch_all: false,
        root: true,
    };
    let root = match cx.node(screen, &top) {
        Some(kid) => kid.node,
        None => ParserNode::new(Tag::Div, vec![], vec![]),
    };
    let name = screen.name.clone();
    Ok(Resolution {
        fn_name: sanitize(&snake_case(&name), "screen"),
        name: name.clone(),
        decl: Src::new(
            cx.at(screen.name_line),
            format!("screen \"{}\"", ascii(&name)),
        ),
        source_label: file.to_string(),
        root,
        params: Vec::new(),
        mapped: cx.mapped,
        dropped: cx.dropped,
        notes: cx.notes,
    })
}

/// `12.0` -> `12`, `140.5` -> `140.5`.
fn num(v: f32) -> String {
    let r = (v * 100.0).round() / 100.0;
    if r.fract() == 0.0 {
        format!("{r:.0}")
    } else {
        format!("{r}")
    }
}

/// The text as a construct names it: ASCII, anything else escaped
/// (`\u{1f319}`), so a layer's name never puts a symbol in a report.
fn ascii(text: &str) -> String {
    text.chars()
        .map(|c| {
            if c.is_ascii() && !c.is_ascii_control() {
                c.to_string()
            } else {
                c.escape_unicode().to_string()
            }
        })
        .collect()
}

/// `FRAME "settings"`: a layer as a construct names it.
fn label(n: &FigNode) -> String {
    format!("{} \"{}\"", n.kind, ascii(&n.name))
}

fn kind_of(n: &FigNode) -> &str {
    n.kind.as_str()
}

/// True for a layer that becomes a node (visible, and not a vector).
fn becomes_node(n: &FigNode) -> bool {
    n.visible()
        && matches!(
            kind_of(n),
            "FRAME" | "INSTANCE" | "COMPONENT" | "GROUP" | "RECTANGLE" | "TEXT"
        )
        && (kind_of(n) != "TEXT" || n.str("characters").is_some_and(|c| !c.trim().is_empty()))
}

impl<'a> Ctx<'a> {
    fn at(&self, line: usize) -> String {
        format!("{}:{line}", self.file)
    }

    pub(super) fn src(&self, line: usize, what: impl Into<String>) -> Src {
        Src::new(self.at(line), what)
    }

    fn hold(&mut self, src: Src, why: &'static str) {
        self.mapped += 1;
        self.notes.push(Note {
            src,
            kind: NoteKind::Holds(why),
        });
    }

    pub(super) fn lose(&mut self, src: &Src, why: impl Into<String>) {
        self.dropped
            .push(Dropped::new(src.what.clone(), why, src.at.clone()));
    }

    /// The `file:line` of every key and layer under `n`.
    fn sites(&self, n: &FigNode, out: &mut Vec<String>) {
        out.push(self.at(n.name_line));
        out.extend(n.keys.iter().map(|(_, _, l)| self.at(*l)));
        for c in &n.children {
            out.push(self.at(c.line));
            self.sites(c, out);
        }
    }

    fn drop_layer(&mut self, n: &FigNode, why: &str, rewritten: bool) {
        let mut covers = Vec::new();
        self.sites(n, &mut covers);
        let what = label(n);
        let mut entry = Dropped::new(what, why, self.at(n.line)).covering(covers);
        if rewritten {
            entry = entry.rewritten();
        }
        self.dropped.push(entry);
    }

    /// The first layer under the document's pages and sections; the others
    /// are reported.
    fn screen<'d>(&mut self, n: &'d FigNode) -> Option<&'d FigNode> {
        if !matches!(kind_of(n), "DOCUMENT" | "CANVAS" | "SECTION") {
            return Some(n);
        }
        self.hold(self.src(n.line, label(n)), WRAPPER);
        for (k, _, line) in &n.keys {
            let why = "a page or a section's own place and look are not part of the screen";
            self.hold(self.src(*line, k.clone()), why);
        }
        let mut found = None;
        for c in &n.children {
            if found.is_none() {
                found = self.screen(c);
            } else {
                self.drop_layer(
                    c,
                    "another layer of the page: the importer takes the first screen",
                    false,
                );
            }
        }
        found
    }

    fn node(&mut self, n: &FigNode, parent: &Parent) -> Option<Kid> {
        if !n.visible() {
            self.drop_layer(n, HIDDEN, true);
            return None;
        }
        match kind_of(n) {
            "TEXT" => self.text(n, parent),
            "FRAME" | "INSTANCE" | "COMPONENT" | "GROUP" | "RECTANGLE" => {
                Some(self.frame(n, parent))
            }
            "VECTOR" | "ELLIPSE" | "STAR" | "LINE" | "REGULAR_POLYGON" | "BOOLEAN_OPERATION" => {
                self.drop_layer(n, VECTOR, false);
                None
            }
            k => {
                let why = format!("a layer of type {k} is outside the importer");
                self.drop_layer(n, &why, false);
                None
            }
        }
    }

    fn origin(&self, n: &FigNode) -> Origin {
        Origin {
            src: Some(self.src(n.line, label(n))),
            tag: n.kind.clone(),
            name: n.name.clone(),
            ..Origin::default()
        }
    }

    /// A sizing key as (fixed, its construct): `FIXED` fixes the axis,
    /// `AUTO` hugs.
    fn sizing(&self, n: &FigNode, key: &str) -> (bool, Option<Src>) {
        let v = n.str(key);
        let src = v.map(|v| self.src(n.line_of(key), format!("{key} {v}")));
        (v != Some("AUTO"), src)
    }

    /// The size and the place of `n`, given what the layer fixes per axis
    /// (`own`, across then down): a fixed axis is `w`/`h`; one that hugs,
    /// grows or stretches is left to the stack. Returns the construct of
    /// its position, for the frame it is in to decide.
    fn size(
        &mut self,
        n: &FigNode,
        parent: &Parent,
        own: [(bool, Option<Src>); 2],
        props: &mut Vec<Prop>,
    ) -> Src {
        let [x, y, w, h] = n.bounds().unwrap_or_default();
        let line = n.line_of("absoluteBoundingBox");
        let size = self.src(line, format!("absoluteBoundingBox {} x {}", num(w), num(h)));
        let at = self.src(
            line,
            format!("absoluteBoundingBox at {}, {}", num(x), num(y)),
        );
        let grow = n.num("layoutGrow").filter(|g| *g > 0.0);
        let stretch = n.str("layoutAlign") == Some("STRETCH");
        let inside = parent.auto.is_some();
        if let Some(g) = grow {
            let src = self.src(n.line_of("layoutGrow"), format!("layoutGrow {}", num(g)));
            if inside {
                self.mapped += 1;
                props.push(Prop::f32("grow", g).from(src));
            } else {
                self.hold(src, "outside an auto layout, layoutGrow does nothing");
            }
        }
        if stretch && !(inside && parent.stretch_all) {
            let src = self.src(n.line_of("layoutAlign"), "layoutAlign STRETCH");
            if inside {
                let why = "a child stretched among children aligned otherwise: the IR aligns every child of a stack alike";
                self.lose(&src, why);
            } else {
                self.hold(src, "outside an auto layout, layoutAlign does nothing");
            }
        }
        if n.str("layoutPositioning") == Some("ABSOLUTE") {
            let src = self.src(n.line_of("layoutPositioning"), "layoutPositioning ABSOLUTE");
            self.lose(
                &src,
                "the IR has no absolute position: the layer is laid out in the stack",
            );
        }
        let mut fixed = [own[0].0, own[1].0];
        let (along, across) = match parent.auto {
            Some(Axis::Across) => (0, 1),
            Some(Axis::Down) => (1, 0),
            None => (2, 2),
        };
        if grow.is_some() && along < 2 {
            fixed[along] = false;
        }
        if stretch && parent.stretch_all && across < 2 {
            fixed[across] = false;
        }
        for (i, (name, v)) in [("w", w), ("h", h)].into_iter().enumerate() {
            let key = own[i].1.clone();
            if fixed[i] {
                self.mapped += 1;
                let mut p = Prop::f32(name, v).from(size.clone());
                p.src.extend(key);
                props.push(p);
            } else if let Some(k) = key {
                let why = if own[i].0 {
                    "its stack sizes it on this axis: it grows or stretches"
                } else {
                    "it hugs its content on this axis, as a stack's child does"
                };
                self.hold(k, why);
            }
        }
        if !fixed[0] && !fixed[1] {
            self.hold(
                size,
                "its size follows the layout: it hugs, grows or stretches",
            );
        }
        if inside {
            self.hold(
                at.clone(),
                "a child of an auto layout is placed by its stack",
            );
        } else if parent.root {
            self.hold(
                at.clone(),
                "the screen's place on the canvas is not part of the screen",
            );
        }
        at
    }

    fn text(&mut self, n: &FigNode, parent: &Parent) -> Option<Kid> {
        let chars = n.str("characters").unwrap_or_default().to_string();
        if chars.trim().is_empty() {
            self.drop_layer(n, "an empty text draws nothing: left out", true);
            return None;
        }
        let mut props = Vec::new();
        let auto = self.style(n, &mut props);
        self.fills(n, "text_color", &mut props);
        let chars_src = self.src(
            n.line_of("characters"),
            format!("characters \"{}\"", ascii(&chars)),
        );
        self.hold(chars_src, "the layer's text is the run's text");
        let own = match &auto {
            Some((a, s)) if a == "WIDTH_AND_HEIGHT" => {
                [(false, Some(s.clone())), (false, Some(s.clone()))]
            }
            Some((a, s)) if a == "HEIGHT" => [(true, Some(s.clone())), (false, Some(s.clone()))],
            Some((_, s)) => [(true, Some(s.clone())), (true, Some(s.clone()))],
            None => [(true, None), (true, None)],
        };
        let at = self.size(n, parent, own, &mut props);
        self.strokes(n, &mut props);
        self.shape(n, &mut props);
        self.instance(n);
        let mut node = ParserNode::new(Tag::Text(TextValue::Literal(chars)), props, vec![]);
        node.origin = self.origin(n);
        Some(Kid {
            node,
            bounds: n.bounds().unwrap_or_default(),
            at,
        })
    }

    /// An instance's component and the overrides the export could not
    /// apply.
    fn instance(&mut self, n: &FigNode) {
        if let Some(c) = n.str("componentId") {
            let src = self.src(n.line_of("componentId"), format!("componentId {c}"));
            self.hold(
                src,
                "an instance's layers are its component's, expanded by the export",
            );
        }
        let list = n.get("unappliedOverrides").and_then(|v| v.as_array());
        for o in list.into_iter().flatten() {
            let path = o.get("path").and_then(|p| p.as_str()).unwrap_or("?");
            let fields = o.get("fields").map(|f| f.to_string()).unwrap_or_default();
            let what = format!("unappliedOverrides {path} {fields}");
            let src = self.src(n.line_of("unappliedOverrides"), what);
            let why = if o.get("matched").and_then(|m| m.as_bool()) == Some(false) {
                "an override whose layer the export did not find: not applied"
            } else {
                "an override the export could not apply (a style or a variable reference, not a value): the instance shows its component's value"
            };
            self.lose(&src, why);
        }
    }

    fn frame(&mut self, n: &FigNode, parent: &Parent) -> Kid {
        let auto = match n.str("layoutMode") {
            Some("HORIZONTAL") => Some(Axis::Across),
            Some("VERTICAL") => Some(Axis::Down),
            _ => None,
        };
        let becoming: Vec<&FigNode> = n.children.iter().filter(|c| becomes_node(c)).collect();
        let stretch_all = auto.is_some()
            && !becoming.is_empty()
            && becoming
                .iter()
                .all(|c| c.str("layoutAlign") == Some("STRETCH"));
        let mut layout = Vec::new();
        let own = match auto {
            Some(a) => {
                self.stack(n, a, stretch_all, &becoming, &mut layout);
                let primary = self.sizing(n, "primaryAxisSizingMode");
                let counter = self.sizing(n, "counterAxisSizingMode");
                match a {
                    Axis::Across => [primary, counter],
                    Axis::Down => [counter, primary],
                }
            }
            None => [(true, None), (true, None)],
        };
        let mut props = Vec::new();
        let at = self.size(n, parent, own, &mut props);
        self.padding(n, &mut props);
        self.fills(n, "bg", &mut props);
        self.strokes(n, &mut props);
        self.shape(n, &mut props);
        self.instance(n);
        let inner = Parent {
            auto,
            stretch_all,
            root: false,
        };
        let mut kids: Vec<Kid> = n
            .children
            .iter()
            .filter_map(|c| self.node(c, &inner))
            .collect();
        if auto.is_none() && !kids.is_empty() {
            self.place(n, &mut kids, &mut layout);
        }
        layout.extend(props);
        let node = ParserNode {
            tag: Tag::Div,
            props: layout,
            children: kids.into_iter().map(|k| k.node).collect(),
            origin: self.origin(n),
        };
        Kid {
            node,
            bounds: n.bounds().unwrap_or_default(),
            at,
        }
    }

    /// An auto layout: its axis, gap, alignment across and justification.
    fn stack(
        &mut self,
        n: &FigNode,
        axis: Axis,
        stretch_all: bool,
        becoming: &[&FigNode],
        out: &mut Vec<Prop>,
    ) {
        let mode = n.str("layoutMode").unwrap_or_default();
        let src = self.src(n.line_of("layoutMode"), format!("layoutMode {mode}"));
        self.mapped += 1;
        let name = if axis == Axis::Across { "row" } else { "col" };
        out.push(Prop::flag(name).from(src));
        let primary = n.str("primaryAxisAlignItems");
        if let Some(g) = n.num("itemSpacing") {
            let src = self.src(n.line_of("itemSpacing"), format!("itemSpacing {}", num(g)));
            if primary == Some("SPACE_BETWEEN") {
                self.hold(
                    src,
                    "with space between, the spacing is automatic: itemSpacing is not used",
                );
            } else {
                self.mapped += 1;
                out.push(Prop::f32("gap", g).from(src));
            }
        }
        let counter = n.str("counterAxisAlignItems");
        let counter_src = counter.map(|c| {
            self.src(
                n.line_of("counterAxisAlignItems"),
                format!("counterAxisAlignItems {c}"),
            )
        });
        if stretch_all {
            let mut p = Prop::token("align_items", "Align::Stretch");
            for c in becoming {
                p.src
                    .push(self.src(c.line_of("layoutAlign"), "layoutAlign STRETCH"));
            }
            self.mapped += p.src.len();
            out.push(p);
            if let Some(s) = counter_src {
                self.hold(
                    s,
                    "every child stretches across: the counter alignment places none",
                );
            }
        } else {
            let align = match counter {
                Some("CENTER") => Some("Align::Center"),
                Some("MAX") => Some("Align::End"),
                Some(_) => None,
                // Figma's default: children at the start across
                None => Some("Align::Start"),
            };
            match (align, counter_src) {
                (Some(a), Some(s)) => {
                    self.mapped += 1;
                    out.push(Prop::token("align_items", a).from(s));
                }
                (Some(a), None) => out.push(Prop::token("align_items", a)),
                (None, Some(s)) => self.lose(&s, "a baseline alignment has no IR form"),
                (None, None) => {}
            }
        }
        if let Some(p) = primary {
            let src = self.src(
                n.line_of("primaryAxisAlignItems"),
                format!("primaryAxisAlignItems {p}"),
            );
            let justify = match p {
                "CENTER" => Some("Justify::Center"),
                "MAX" => Some("Justify::End"),
                "SPACE_BETWEEN" => Some("Justify::SpaceBetween"),
                _ => None,
            };
            match justify {
                Some(j) => {
                    self.mapped += 1;
                    out.push(Prop::token("justify", j).from(src));
                }
                None => self.lose(&src, "a justification the builder does not have"),
            }
        }
    }

    /// The position rule for a frame with no auto layout (see the module).
    fn place(&mut self, n: &FigNode, kids: &mut [Kid], layout: &mut Vec<Prop>) {
        let [fx, fy, _, _] = n.bounds().unwrap_or_default();
        let rel = |k: &Kid| [k.bounds[0] - fx, k.bounds[1] - fy, k.bounds[2], k.bounds[3]];
        let disjoint = |axis: usize, kids: &[Kid]| {
            let mut spans: Vec<(f32, f32)> = kids
                .iter()
                .map(|k| (rel(k)[axis], rel(k)[axis] + rel(k)[axis + 2]))
                .collect();
            spans.sort_by(|a, b| a.0.total_cmp(&b.0));
            spans.windows(2).all(|w| w[1].0 >= w[0].1 - 0.5)
        };
        let axis = if disjoint(1, kids) {
            1
        } else if disjoint(0, kids) {
            0
        } else {
            let why = "the frame's children overlap and the IR has no absolute position: they are stacked in source order";
            for k in kids.iter() {
                self.lose(&k.at, why);
            }
            return;
        };
        kids.sort_by(|a, b| rel(a)[axis].total_cmp(&rel(b)[axis]));
        let (stack, order) = if axis == 0 {
            layout.push(Prop::flag("row"));
            ("row", "x")
        } else {
            ("column", "y")
        };
        let mut along = 0.0;
        for k in kids.iter() {
            let r = rel(k);
            let (sx, sy) = if axis == 1 {
                (0.0, along)
            } else {
                (along, 0.0)
            };
            along += r[axis + 2];
            if (r[0] - sx).abs() < 0.5 && (r[1] - sy).abs() < 0.5 {
                let why = "where the stack puts it: the frame's children do not overlap and keep their order";
                self.hold(k.at.clone(), why);
                continue;
            }
            let why = format!(
                "at x {}, y {} in a frame with no auto layout; its children do not overlap, so they are a {stack} in {order} order, which puts it at x {}, y {}",
                num(r[0]),
                num(r[1]),
                num(sx),
                num(sy)
            );
            let entry = Dropped::new(k.at.what.clone(), why, k.at.at.clone());
            self.dropped.push(entry.approximated());
        }
    }
}
