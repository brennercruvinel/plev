//! parser intermediate representation: the framework-neutral element tree that
//! stage 1 (parse) feeds and stage 3 (emit) prints. Props are already
//! normalized to plev builder method names; values carry either literals or
//! theme-token rust expressions resolved by stage 2.

/// One node of the transpiled element tree.
#[derive(Debug, Clone, PartialEq)]
pub struct ParserNode {
    pub tag: Tag,
    /// Normalized props in source order; each one prints as `.name(args)`.
    pub props: Vec<Prop>,
    pub children: Vec<ParserNode>,
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

/// A plev builder call: `.name(args...)`.
#[derive(Debug, Clone, PartialEq)]
pub struct Prop {
    pub name: &'static str,
    pub args: Vec<Arg>,
}

impl Prop {
    pub fn new(name: &'static str, args: Vec<Arg>) -> Self {
        Self { name, args }
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
}

impl Dropped {
    pub fn new(what: impl Into<String>, why: impl Into<String>, at: impl Into<String>) -> Self {
        Self {
            what: what.into(),
            why: why.into(),
            at: at.into(),
        }
    }
}

/// Output of stage 2 for one component instance.
#[derive(Debug)]
pub struct Resolution {
    pub fn_name: String,
    pub source_label: String,
    pub root: ParserNode,
    pub params: Vec<Param>,
    /// Count of source constructs faithfully represented (incl. no-ops that
    /// hold in plev by construction, e.g. `box-sizing: border-box`).
    pub mapped: usize,
    pub dropped: Vec<Dropped>,
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
}
