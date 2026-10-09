//! Builder props -> IR tokens, one decision per group of props that makes
//! one IR field (the padding sides, the three parts of a text style), each
//! decision the verdict of every construct behind it.
//!
//! A value goes to the nearest token (`tokens::nearest`): at distance 0 it
//! is `transformed`, within the token's tolerance `approximated` with the
//! difference, past it `impossible` and the field is left to the
//! component's default.

use plevir::{
    Align, ColorRole, MAX_FLEX, Props, ShapeRole, Size, SizeToken, SpacingToken, TypographyRole,
};

use crate::report::Verdict;
use crate::tokens::{Near, Tokens, TypeStyle, color_distance, nearest, px_distance, role_named};
use crate::tree::{Arg, Prop, Src};

/// The verdict of the constructs `srcs`.
#[derive(Clone, Debug)]
pub(crate) struct Decision {
    pub srcs: Vec<Src>,
    pub verdict: Verdict,
}

fn decide(srcs: &[Src], verdict: Verdict) -> Decision {
    Decision {
        srcs: srcs.to_vec(),
        verdict,
    }
}

fn supported(ir: impl Into<String>) -> Verdict {
    Verdict::Supported { ir: ir.into() }
}

fn impossible(reason: impl Into<String>) -> Verdict {
    Verdict::Impossible {
        reason: reason.into(),
    }
}

/// `12.0` -> `12`, `15.96` -> `15.96`.
pub(crate) fn num(v: f32) -> String {
    let r = (v * 100.0).round() / 100.0;
    if r.fract() == 0.0 {
        format!("{r:.0}")
    } else {
        format!("{r}")
    }
}

/// The value of a numeric argument, a literal or a length token.
fn length(arg: Option<&Arg>, t: &Tokens) -> Option<f32> {
    match arg? {
        Arg::F32(v) => Some(*v),
        Arg::Int(i) => Some(*i as f32),
        Arg::Token(e) => t.length_of(e),
        Arg::Pair(_) => None,
    }
}

/// The verdict of a value mapped to `near`, the text of the IR field it
/// fills (`ir`), the value as the source has it and as the token has it.
fn by_measure<T>(near: &Near<T>, ir: String, source: &str, token: &str, what: &str) -> Verdict {
    let (d, tol) = (num(near.distance), num(near.tolerance));
    if near.distance == 0.0 {
        Verdict::Transformed {
            ir,
            rule: format!("{source} is {token} in the theme"),
        }
    } else if near.distance <= near.tolerance {
        Verdict::Approximated {
            ir,
            difference: format!(
                "{source} is {d} {what} from {token}, within its tolerance of {tol}"
            ),
        }
    } else {
        impossible(format!(
            "{source} has no token: the nearest, {token}, is {d} {what} away, past its tolerance of {tol}"
        ))
    }
}

/// A color expression -> the color role for `field`.
pub(crate) fn color(
    expr: &str,
    field: &str,
    srcs: &[Src],
    t: &Tokens,
) -> (Option<ColorRole>, Decision) {
    if let Some(role) = expr.strip_prefix("theme.colors.").and_then(role_named) {
        let ir = format!("props.{field} = {}", role.as_str());
        return (Some(role), decide(srcs, supported(ir)));
    }
    let Some(value) = t.color_of(expr) else {
        let reason = format!("the value of `{expr}` in the theme is not known to the importer");
        return (None, decide(srcs, impossible(reason)));
    };
    let Some(near) = nearest(&t.colors, &value, color_distance) else {
        return (
            None,
            decide(srcs, impossible("the theme has no color roles")),
        );
    };
    let role = near.token;
    let ir = format!("props.{field} = {}", role.as_str());
    let token = format!("theme.colors.{}", role.as_str());
    let verdict = by_measure(&near, ir, &format!("`{expr}`"), &token, "levels");
    let keep = !matches!(verdict, Verdict::Impossible { .. });
    (keep.then_some(role), decide(srcs, verdict))
}

/// The padding sides, set in call order as the builder sets them.
#[derive(Default)]
pub(crate) struct Sides {
    /// top, right, bottom, left.
    px: [f32; 4],
    srcs: Vec<Src>,
}

impl Sides {
    /// Applies `.p`, `.px`, `.py`, `.pt`, `.pr`, `.pb` or `.pl`; false for
    /// any other prop.
    pub(crate) fn set(&mut self, p: &Prop) -> bool {
        let sides: &[usize] = match p.name {
            "p" => &[0, 1, 2, 3],
            "px" => &[1, 3],
            "py" => &[0, 2],
            "pt" => &[0],
            "pr" => &[1],
            "pb" => &[2],
            "pl" => &[3],
            _ => return false,
        };
        if let Some(Arg::F32(v)) = p.args.first() {
            for &i in sides {
                if let Some(side) = self.px.get_mut(i) {
                    *side = *v;
                }
            }
        }
        self.srcs.extend(p.src.iter().cloned());
        true
    }

    /// The IR padding: the token of the smallest side, so the IR never
    /// adds room the source does not have; every larger side loses the
    /// difference.
    pub(crate) fn decide(&self, t: &Tokens) -> (Option<SpacingToken>, Option<Decision>) {
        if self.srcs.is_empty() {
            return (None, None);
        }
        let names = ["top", "right", "bottom", "left"];
        let m = self.px.iter().copied().fold(f32::INFINITY, f32::min);
        if m <= 0.0 {
            let lost: Vec<String> = names
                .iter()
                .zip(self.px)
                .filter(|(_, v)| *v > 0.0)
                .map(|(n, v)| format!("{} px on the {n}", num(v)))
                .collect();
            let reason = format!(
                "the IR's padding is one token for the four sides; with 0 on a side there is none, so {} {} lost",
                lost.join(" and "),
                if lost.len() == 1 { "is" } else { "are" }
            );
            return (None, Some(decide(&self.srcs, impossible(reason))));
        }
        let Some(near) = nearest(&t.spacing, &m, px_distance) else {
            return (
                None,
                Some(decide(&self.srcs, impossible("the theme has no spacing"))),
            );
        };
        let token = near.token;
        let tv = t.spacing_of(token).unwrap_or(m);
        let ir = format!("props.padding = {}", token.as_str());
        let name = format!("theme.spacing.{} ({} px)", token.as_str(), num(tv));
        let even = self.px.iter().all(|v| *v == tv);
        let verdict = if near.distance > near.tolerance || even {
            by_measure(&near, ir, &format!("{} px", num(m)), &name, "px")
        } else {
            let lost: Vec<String> = names
                .iter()
                .zip(self.px)
                .filter(|(_, v)| *v != tv)
                .map(|(n, v)| format!("{n} {} is {}", num(v), num(tv)))
                .collect();
            Verdict::Approximated {
                ir,
                difference: format!(
                    "one token for the four sides, the smallest one's, {name}: {}",
                    lost.join(", ")
                ),
            }
        };
        let keep = !matches!(verdict, Verdict::Impossible { .. });
        (keep.then_some(token), Some(decide(&self.srcs, verdict)))
    }
}

/// A fixed width or height -> the size token of that dimension, as both
/// bounds.
pub(crate) fn size(p: &Prop, height: bool, t: &Tokens) -> (Option<SizeToken>, Decision) {
    let what = if height { "height" } else { "width" };
    let Some(v) = length(p.args.first(), t) else {
        let reason = format!("a {what} that is not a number of pixels");
        return (None, decide(&p.src, impossible(reason)));
    };
    let table = t.sizes_of(height);
    let Some(near) = nearest(&table, &v, px_distance) else {
        let reason = format!("{} px: the theme has no {what} token", num(v));
        return (None, decide(&p.src, impossible(reason)));
    };
    let token = near.token;
    let tv = table
        .iter()
        .find(|(k, _)| *k == token)
        .map_or(v, |(_, x)| *x);
    let ir = format!("size.min = size.max = {}", token.as_str());
    let name = format!("theme.size.{} ({} px)", token.as_str(), num(tv));
    let verdict = by_measure(
        &near,
        ir,
        &format!("a fixed {what} of {} px", num(v)),
        &name,
        "px",
    );
    let keep = !matches!(verdict, Verdict::Impossible { .. });
    (keep.then_some(token), decide(&p.src, verdict))
}

/// A corner radius -> the shape role.
pub(crate) fn shape(p: &Prop, t: &Tokens) -> (Option<ShapeRole>, Decision) {
    let Some(v) = length(p.args.first(), t) else {
        return (
            None,
            decide(
                &p.src,
                impossible("a radius that is not a number of pixels"),
            ),
        );
    };
    let Some(near) = nearest(&t.shapes, &v, px_distance) else {
        return (
            None,
            decide(&p.src, impossible("the theme has no shape roles")),
        );
    };
    let token = near.token;
    let tv = t
        .shapes
        .iter()
        .find(|(k, _)| *k == token)
        .map_or(v, |(_, x)| *x);
    let ir = format!("props.shape = {}", token.as_str());
    let name = format!("theme.shape.{} ({} px)", token.as_str(), num(tv));
    let verdict = by_measure(
        &near,
        ir,
        &format!("a radius of {} px", num(v)),
        &name,
        "px",
    );
    let keep = !matches!(verdict, Verdict::Impossible { .. });
    (keep.then_some(token), decide(&p.src, verdict))
}

/// `grow(n)` -> the flex weight.
pub(crate) fn flex(p: &Prop) -> (Option<u8>, Decision) {
    let v = match p.args.first() {
        Some(Arg::F32(v)) => *v,
        _ => 1.0,
    };
    if v <= 0.0 {
        return (
            None,
            decide(&p.src, supported("no flex weight, the default")),
        );
    }
    let n = v.round().clamp(1.0, f32::from(MAX_FLEX));
    let ir = format!("size.flex = {}", num(n));
    let verdict = if n == v {
        supported(ir)
    } else {
        Verdict::Approximated {
            ir,
            difference: format!(
                "a weight of {} is {} (whole, 1 to {MAX_FLEX})",
                num(v),
                num(n)
            ),
        }
    };
    (Some(n as u8), decide(&p.src, verdict))
}

/// The text style of a text run (`font_size`, `line_height`,
/// `font_weight`, with the builder's defaults for what is left out) -> the
/// typography role.
pub(crate) fn typography(
    style: &[&Prop],
    t: &Tokens,
) -> (Option<TypographyRole>, Option<Decision>) {
    if style.is_empty() {
        return (None, None);
    }
    let arg = |name: &str| {
        style
            .iter()
            .find(|p| p.name == name)
            .and_then(|p| p.args.first())
    };
    let f = |a: Option<&Arg>| match a {
        Some(Arg::F32(v)) => Some(*v),
        Some(Arg::Int(i)) => Some(*i as f32),
        _ => None,
    };
    // the builder's text: 16 px, a line height of 1.3 its size, weight 400
    let size = f(arg("font_size")).unwrap_or(16.0);
    let line_height = f(arg("line_height")).unwrap_or(size * 1.3);
    let weight = f(arg("font_weight")).map_or(400, |w| w as u16);
    let wanted = TypeStyle {
        size,
        line_height,
        weight,
    };
    let srcs: Vec<Src> = style.iter().flat_map(|p| p.src.iter().cloned()).collect();
    let Some(near) = t.nearest_type(&wanted) else {
        return (
            None,
            Some(decide(&srcs, impossible("the theme has no type ramp"))),
        );
    };
    let role = near.token;
    let got = t.type_of(role).unwrap_or(wanted);
    let ir = format!("props.typography = {}", role.as_str());
    let source = format!(
        "{}/{} px weight {}",
        num(size),
        num(line_height),
        wanted.weight
    );
    let name = format!(
        "theme.typography.{} ({}/{} px weight {})",
        role.as_str(),
        num(got.size),
        num(got.line_height),
        got.weight
    );
    let mut verdict = by_measure(&near, ir.clone(), &source, &name, "px");
    if got.weight != wanted.weight && !matches!(verdict, Verdict::Impossible { .. }) {
        verdict = Verdict::Approximated {
            ir,
            difference: format!(
                "{source} goes to {name}: no role of weight {} is near",
                wanted.weight
            ),
        };
    }
    let keep = !matches!(verdict, Verdict::Impossible { .. });
    (keep.then_some(role), Some(decide(&srcs, verdict)))
}

/// A gap by token (`gap(v)`), and the spacing between two children the
/// margin-bottom rewrite made: the stack's one gap.
pub(crate) fn spacing(v: f32, t: &Tokens) -> Option<(SpacingToken, Near<SpacingToken>, f32)> {
    let near = nearest(&t.spacing, &v, px_distance)?;
    let tv = t.spacing_of(near.token)?;
    Some((near.token, near, tv))
}

/// The spacings between consecutive children of a stack (0 where the
/// rewrite put no spacer), each with the declarations behind it -> the
/// stack's one gap, the smallest spacing's token, so the IR never adds room
/// the source does not have. `what` names the spacing in the rule (the
/// margin below a child, a flex gap).
pub(crate) fn gap(
    pairs: &[(f32, Vec<Src>)],
    what: &str,
    t: &Tokens,
) -> (Option<SpacingToken>, Vec<Decision>) {
    if pairs.iter().all(|(_, s)| s.is_empty()) {
        return (None, Vec::new());
    }
    let m = pairs.iter().map(|(h, _)| *h).fold(f32::INFINITY, f32::min);
    let spacers = pairs.iter().filter(|(_, s)| !s.is_empty());
    if m <= 0.0 {
        let out = spacers
            .map(|(h, s)| {
                decide(
                    s,
                    impossible(format!(
                        "the IR's stack has one gap for all its children and another pair has none, so these {} px are lost",
                        num(*h)
                    )),
                )
            })
            .collect();
        return (None, out);
    }
    let Some((token, near, tv)) = spacing(m, t) else {
        return (None, Vec::new());
    };
    let ir = format!("layout.stack.gap = {}", token.as_str());
    let name = format!("theme.spacing.{} ({} px)", token.as_str(), num(tv));
    if near.distance > near.tolerance {
        let out = spacers
            .map(|(h, s)| {
                let v = by_measure(&near, ir.clone(), &format!("{} px", num(*h)), &name, "px");
                decide(s, v)
            })
            .collect();
        return (None, out);
    }
    let out = spacers
        .map(|(h, s)| {
            let verdict = if *h == tv {
                Verdict::Transformed {
                    ir: ir.clone(),
                    rule: format!("{what} is the stack's gap, {name}"),
                }
            } else {
                Verdict::Approximated {
                    ir: ir.clone(),
                    difference: format!(
                        "{} px here; the stack's one gap is the smallest spacing's token, {name}",
                        num(*h)
                    ),
                }
            };
            decide(s, verdict)
        })
        .collect();
    (Some(token), out)
}

/// The layout, look and size of a container, from its builder props.
#[derive(Default)]
pub(crate) struct Container {
    pub props: Props,
    pub size: Size,
    pub row: bool,
    pub align: Option<Align>,
    /// A `gap(v)`, with its constructs: decided with the children, by the
    /// container (`emit_ir::Cx::container`).
    pub gap: Option<(f32, Vec<Src>)>,
    pub decisions: Vec<Decision>,
}

/// Every prop of a container `div`.
pub(crate) fn container(props: &[Prop], t: &Tokens) -> Container {
    let mut out = Container::default();
    let mut sides = Sides::default();
    for p in props {
        if sides.set(p) {
            continue;
        }
        if p.name == "gap"
            && let Some(v) = length(p.args.first(), t)
        {
            out.gap = Some((v, p.src.clone()));
            continue;
        }
        let d = match p.name {
            "row" => {
                out.row = true;
                decide(&p.src, supported("layout.stack.axis = horizontal"))
            }
            "col" => decide(&p.src, supported("layout.stack.axis = vertical")),
            "align_center" => {
                out.align = Some(Align::Center);
                decide(&p.src, supported("layout.stack.align = center"))
            }
            "align_items" => match align(p.args.first()) {
                Some(a) => {
                    out.align = Some(a);
                    let ir = format!("layout.stack.align = {}", a.as_str());
                    decide(&p.src, supported(ir))
                }
                None => decide(&p.src, impossible(lost("align_items"))),
            },
            "grow" => {
                let (v, d) = flex(p);
                out.size.flex = v;
                d
            }
            "w" | "h" => {
                let height = p.name == "h";
                let (v, d) = size(p, height, t);
                if let Some(token) = v {
                    out.size.min = Some(token);
                    out.size.max = Some(token);
                }
                d
            }
            "bg" => {
                let expr = match p.args.first() {
                    Some(Arg::Token(e)) => e.as_str(),
                    _ => "",
                };
                let (v, d) = color(expr, "background", &p.src, t);
                out.props.background = v;
                d
            }
            "rounded" => {
                let (v, d) = shape(p, t);
                out.props.shape = v;
                d
            }
            name => decide(&p.src, impossible(lost(name))),
        };
        out.decisions.push(d);
    }
    let (padding, d) = sides.decide(t);
    out.props.padding = padding;
    out.decisions.extend(d);
    out
}

/// `Align::Start` (the builder's) -> the stack's align.
fn align(arg: Option<&Arg>) -> Option<Align> {
    let Some(Arg::Token(e)) = arg else {
        return None;
    };
    match e.strip_prefix("Align::")? {
        "Start" => Some(Align::Start),
        "Center" => Some(Align::Center),
        "End" => Some(Align::End),
        "Stretch" => Some(Align::Stretch),
        _ => None,
    }
}

/// Every prop of a text run: its style and its color.
pub(crate) fn text(props: &[Prop], t: &Tokens) -> (Props, Vec<Decision>) {
    let mut out = Props::default();
    let mut decisions = Vec::new();
    let style: Vec<&Prop> = props
        .iter()
        .filter(|p| matches!(p.name, "font_size" | "line_height" | "font_weight"))
        .collect();
    let (role, d) = typography(&style, t);
    out.typography = role;
    decisions.extend(d);
    for p in props {
        match p.name {
            "font_size" | "line_height" | "font_weight" => {}
            "text_color" => {
                let expr = match p.args.first() {
                    Some(Arg::Token(e)) => e.as_str(),
                    _ => "",
                };
                let (v, d) = color(expr, "color", &p.src, t);
                out.color = v;
                decisions.push(d);
            }
            name => decisions.push(decide(&p.src, impossible(lost(name)))),
        }
    }
    (out, decisions)
}

/// Why a builder prop has no IR field.
pub(crate) fn lost(name: &str) -> String {
    match name {
        "border" | "border_color" => {
            "the IR has no border: a container draws its fill and its elevation only".into()
        }
        "shadow_inset" => "the IR has no inner shadow".into(),
        "shadow_drop" => {
            "the IR's shadows are the theme's elevation stacks, not a single shadow".into()
        }
        "backdrop_blur" => "the IR has no backdrop blur".into(),
        "clip_children" => {
            "the IR does not clip: a container lays its children out inside it but does not cut what they draw".into()
        }
        "justify" => {
            "the IR's stack has no justification along its axis; free space stays at its end".into()
        }
        "shrink" => "the IR has no shrink weight".into(),
        other => format!("`.{other}()` has no IR field"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::Category;

    fn t() -> Tokens {
        Tokens {
            spacing: vec![
                (SpacingToken::Xs, 4.0),
                (SpacingToken::Sm, 8.0),
                (SpacingToken::Md, 12.0),
                (SpacingToken::Lg, 16.0),
                (SpacingToken::Xl, 24.0),
                (SpacingToken::Xxl, 32.0),
            ],
            colors: vec![
                (
                    ColorRole::Bg,
                    [48.0 / 255.0, 48.0 / 255.0, 48.0 / 255.0, 1.0],
                ),
                (
                    ColorRole::Surface,
                    [46.0 / 255.0, 46.0 / 255.0, 46.0 / 255.0, 1.0],
                ),
            ],
            sizes: vec![(SizeToken::CardW, 368.0), (SizeToken::ToastW, 320.0)],
            ..Tokens::default()
        }
    }

    fn src(n: u32) -> Src {
        Src::new(format!("a.sass:{n}"), format!("decl {n}"))
    }

    #[test]
    fn padding_takes_the_smallest_side() {
        let mut sides = Sides::default();
        sides.set(&Prop::f32("pt", 16.0).from(src(1)));
        sides.set(&Prop::f32("px", 16.0).from(src(1)));
        sides.set(&Prop::f32("pb", 32.0).from(src(1)));
        let (token, d) = sides.decide(&t());
        assert_eq!(token, Some(SpacingToken::Lg));
        let d = d.unwrap();
        assert_eq!(d.verdict.category(), Category::Approximated);
        assert!(format!("{:?}", d.verdict).contains("bottom 32 is 16"));
        let mut even = Sides::default();
        even.set(&Prop::f32("p", 16.0).from(src(2)));
        assert_eq!(
            even.decide(&t()).1.unwrap().verdict.category(),
            Category::Transformed
        );
        let mut side_only = Sides::default();
        side_only.set(&Prop::f32("px", 16.0).from(src(3)));
        let (token, d) = side_only.decide(&t());
        assert_eq!(token, None);
        assert_eq!(d.unwrap().verdict.category(), Category::Impossible);
        let mut off = Sides::default();
        off.set(&Prop::f32("p", 19.0).from(src(4)));
        assert_eq!(off.decide(&t()).0, None, "19 is 3 from lg, past 2");
    }

    #[test]
    fn a_color_by_role_by_value_or_not_at_all() {
        let (role, d) = color("theme.colors.bg", "background", &[src(1)], &t());
        assert_eq!(
            (role, d.verdict.category()),
            (Some(ColorRole::Bg), Category::Supported)
        );
        let surface = crate::formats::react::css::color_token((46, 46, 46, 1.0));
        let (role, d) = color(&surface, "background", &[src(1)], &t());
        assert_eq!(role, Some(ColorRole::Surface));
        assert_eq!(d.verdict.category(), Category::Transformed);
        let (role, d) = color(
            "engine::theme::hoff::n3(0.7)",
            "background",
            &[src(1)],
            &t(),
        );
        assert_eq!((role, d.verdict.category()), (None, Category::Impossible));
        let (role, d) = color("theme.glass.unknown", "background", &[src(1)], &t());
        assert_eq!((role, d.verdict.category()), (None, Category::Impossible));
    }

    #[test]
    fn gaps_take_the_smallest_spacing() {
        let what = "the margin below a child";
        let (g, d) = gap(&[(8.0, vec![src(1)]), (24.0, vec![src(2)])], what, &t());
        assert_eq!(g, Some(SpacingToken::Sm));
        assert_eq!(d[0].verdict.category(), Category::Transformed);
        assert_eq!(d[1].verdict.category(), Category::Approximated);
        let (g, d) = gap(&[(8.0, vec![src(1)]), (0.0, vec![])], what, &t());
        assert_eq!(g, None);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].verdict.category(), Category::Impossible);
        assert_eq!(gap(&[(0.0, vec![])], what, &t()).0, None);
    }

    #[test]
    fn a_fixed_width_is_its_token_as_both_bounds() {
        let (v, d) = size(&Prop::f32("w", 368.0).from(src(1)), false, &t());
        assert_eq!(
            (v, d.verdict.category()),
            (Some(SizeToken::CardW), Category::Transformed)
        );
        let (v, d) = size(&Prop::f32("w", 360.0).from(src(1)), false, &t());
        assert_eq!(
            (v, d.verdict.category()),
            (Some(SizeToken::CardW), Category::Approximated)
        );
        let (v, d) = size(&Prop::f32("h", 336.0).from(src(1)), true, &t());
        assert_eq!((v, d.verdict.category()), (None, Category::Impossible));
    }
}
