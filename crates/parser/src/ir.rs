//! parser intermediate representation: the framework-neutral element tree that
//! stage 1 (parse) feeds and stage 3 (emit) prints. Props are already
//! normalized to plev builder method names; values carry either literals or
//! theme-token rust expressions resolved by stage 2.
//!
//! Every node and every prop also carries where it came from in the source
//! (`Origin`, `Prop::src`): the code emitter never reads it, the plev IR
//! emitter reports by it (`emit_ir`), so each construct lands in exactly one
//! category of its report.

/// One node of the transpiled element tree.
#[derive(Debug, Clone, PartialEq)]
pub struct ParserNode {
    pub tag: Tag,
    /// Normalized props in source order; each one prints as `.name(args)`.
    pub props: Vec<Prop>,
    pub children: Vec<ParserNode>,
    /// What the node was in the source. Not printed by the code emitter.
    pub origin: Origin,
}

impl ParserNode {
    /// A node with no source origin (a test tree, a rewrite's product).
    pub fn new(tag: Tag, props: Vec<Prop>, children: Vec<ParserNode>) -> Self {
        Self {
            tag,
            props,
            children,
            origin: Origin::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tag {
    Div,
    /// A text run: one TextStyle per run, props carry the full style.
    Text(TextValue),
    /// An opaque `Element` function parameter (e.g. a JSX children slot).
    Slot(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextValue {
    /// Literal string from the source ("Discover").
    Literal(String),
    /// A `&str` function parameter ({title}).
    Param(String),
}

/// One construct of the source: `file:line` and the construct as the
/// droplist names it (`.inner { padding: 16px }`, `.px_2()`,
/// `<button class=button>`).
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Src {
    pub at: String,
    pub what: String,
}

impl Src {
    pub fn new(at: impl Into<String>, what: impl Into<String>) -> Self {
        Self {
            at: at.into(),
            what: what.into(),
        }
    }
}

/// Where a node came from: the element or chain it was made from, its
/// source tag and first class, the attributes the code emitter does not
/// read, and the constructs a documented rewrite folded into it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Origin {
    /// The element (tsx) or the `.child(..)` call (gpui) the node is.
    pub src: Option<Src>,
    /// The source tag: `div`, `button`, `input`, `span`; `div()` for gpui.
    pub tag: String,
    /// The first class name (react) or a structural name (gpui), for ids.
    pub name: String,
    /// Attributes other than `className`, in source order.
    pub attrs: Vec<Attr>,
    /// Constructs a rewrite folded into this node, each with its rule.
    pub folded: Vec<(Src, &'static str)>,
    /// The `div().h(v)` of the margin-bottom rewrite.
    pub spacer: bool,
}

/// A JSX attribute other than `className` (`onClick={onOpen}`,
/// `placeholder="Reply"`).
#[derive(Debug, Clone, PartialEq)]
pub struct Attr {
    pub name: String,
    pub value: AttrValue,
    pub src: Src,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttrValue {
    /// A string literal.
    Text(String),
    /// `{identifier}`.
    Ident(String),
    /// Any other expression, as source text.
    Expr(String),
    /// A bare attribute (`disabled`).
    Flag,
}

/// A plev builder call: `.name(args...)`.
///
/// `src` lists the source constructs this prop came from (one, or more when
/// declarations agreed on it); it is not part of equality, which stays
/// name and arguments, as the resolvers compare props.
#[derive(Debug, Clone)]
pub struct Prop {
    pub name: &'static str,
    pub args: Vec<Arg>,
    pub src: Vec<Src>,
}

impl PartialEq for Prop {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.args == other.args
    }
}

impl Prop {
    pub fn new(name: &'static str, args: Vec<Arg>) -> Self {
        Self {
            name,
            args,
            src: Vec::new(),
        }
    }
    pub fn f32(name: &'static str, v: f32) -> Self {
        Self::new(name, vec![Arg::F32(v)])
    }
    pub fn token(name: &'static str, t: impl Into<String>) -> Self {
        Self::new(name, vec![Arg::Token(t.into())])
    }
    pub fn flag(name: &'static str) -> Self {
        Self::new(name, vec![])
    }
    /// This prop, from the construct `src`.
    pub fn from(mut self, src: Src) -> Self {
        self.src.push(src);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Arg {
    F32(f32),
    Int(i64),
    /// `[x, y]` pair (shadow offsets).
    Pair([f32; 2]),
    /// A rust expression emitted verbatim (theme tokens, enum variants).
    Token(String),
}

/// Function parameters of the emitted component, in order of first use.
#[derive(Debug, Clone, PartialEq)]
pub enum Param {
    /// `name: Element`
    Slot(String),
    /// `name: &str`
    Text(String),
}

/// One honest entry of the droplist: a piece of the source that the
/// transpiler could not (or chose not to) represent in plev.
#[derive(Debug, Clone, PartialEq)]
pub struct Dropped {
    /// What was dropped (selector, declaration, method call, match arm).
    pub what: String,
    /// Why it has no plev equivalent, or which rewrite replaced it.
    pub why: String,
    /// `file:line` of the source construct.
    pub at: String,
    /// Lost, or replaced by a documented rewrite.
    pub kind: DropKind,
    /// Other `file:line`s this entry answers for: the declarations of a
    /// rule dropped whole, the elements of a dropped subtree, the calls
    /// inside a dropped match arm.
    pub covers: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropKind {
    /// Nothing of it survives.
    Lost,
    /// A documented rewrite carries its effect (`why` names the rewrite).
    Rewritten,
}

impl Dropped {
    pub fn new(what: impl Into<String>, why: impl Into<String>, at: impl Into<String>) -> Self {
        Self {
            what: what.into(),
            why: why.into(),
            at: at.into(),
            kind: DropKind::Lost,
            covers: Vec::new(),
        }
    }
    /// The entry of a construct a documented rewrite replaced.
    pub fn rewritten(mut self) -> Self {
        self.kind = DropKind::Rewritten;
        self
    }
    /// The entry also answers for `sites`.
    pub fn covering(mut self, sites: Vec<String>) -> Self {
        self.covers = sites;
        self
    }
}

/// A construct the resolver counted as mapped that became no prop: it
/// holds in plev by construction, or a rewrite consumed it.
#[derive(Debug, Clone, PartialEq)]
pub struct Note {
    pub src: Src,
    pub kind: NoteKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteKind {
    /// Holds by construction (`box-sizing: border-box`, `margin: 0`).
    Holds(&'static str),
    /// A documented rewrite consumed it (the overlay merge, a combinator).
    Rewrite(&'static str),
}

/// Output of stage 2 for one component instance.
#[derive(Debug)]
pub struct Resolution {
    pub fn_name: String,
    /// The component's own name as the source writes it (`NoteCard`).
    pub name: String,
    /// The component's declaration.
    pub decl: Src,
    pub source_label: String,
    pub root: ParserNode,
    pub params: Vec<Param>,
    /// Count of source constructs faithfully represented (incl. no-ops that
    /// hold in plev by construction, e.g. `box-sizing: border-box`).
    pub mapped: usize,
    pub dropped: Vec<Dropped>,
    /// The mapped constructs that became no prop.
    pub notes: Vec<Note>,
}

/// Final transpiler output: deterministic rust source plus the truth about
/// what was kept and what was lost.
#[derive(Debug)]
pub struct Transpiled {
    pub code: String,
    pub mapped: usize,
    pub dropped: Vec<Dropped>,
}

/// `HoffResearchCard` -> `hoff_research_card`.
pub fn snake_case(name: &str) -> String {
    let mut out = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snake_case_component_names() {
        assert_eq!(snake_case("HoffResearchCard"), "hoff_research_card");
        assert_eq!(snake_case("Separator"), "separator");
        assert_eq!(snake_case("already_snake"), "already_snake");
    }

    #[test]
    fn a_prop_compares_by_name_and_arguments_not_by_origin() {
        let a = Prop::f32("w", 368.0).from(Src::new("a.sass:3", ".a { width: 368px }"));
        let b = Prop::f32("w", 368.0).from(Src::new("a.sass:9", ".b { width: 368px }"));
        assert_eq!(a, b);
        assert_ne!(a, Prop::f32("w", 367.0));
        assert_eq!(a.src.len(), 1);
    }
}
