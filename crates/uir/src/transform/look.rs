//! the operations on how a node looks, by token only: `Emphasize` (one
//! step up the token the node is drawn by), `Density` (every gap and
//! padding one step of the spacing scale) and `Restyle` (a color by
//! role). none of them changes what the verifier checks, so none fails
//! it; each is refused when it means nothing for the node. their inverse
//! is the props and layouts before.

use sem::SemId;

use super::tree;
use super::{Draft, Inverse, Look, Step, TransformError, refused};
use crate::ir::IrNode;
use crate::token::{ColorRole, ControlSize, Elevation, Layout, SpacingToken, TypographyRole};
use crate::vocab::Component;

fn look(node: &IrNode) -> Look {
    Look {
        id: node.sem.clone(),
        props: node.props,
        layout: node.layout,
    }
}

/// the rung of a text style on the type ramp, by its size
/// (`engine::theme::TypographyScale`: small 10, caption 12, base-2 and
/// body-2 14, base and body 16, title 20, headline 32, h4 the display).
fn rung(t: TypographyRole) -> usize {
    use TypographyRole as T;
    match t {
        T::SmallR | T::SmallSm | T::Readout => 0,
        T::CaptionR | T::CaptionSm | T::Hairline => 1,
        T::Base2r
        | T::Base2m
        | T::Base2sm
        | T::Base2b
        | T::Body2r
        | T::Body2m
        | T::Body2b
        | T::Mono => 2,
        T::BaseR | T::BaseM | T::Body => 3,
        T::Title => 4,
        T::Headline => 5,
        T::H4 => 6,
    }
}

/// the style of each rung, the weighted one where the rung has one:
/// emphasis takes the next rung's.
const RAMP: [TypographyRole; 7] = [
    TypographyRole::SmallSm,
    TypographyRole::CaptionSm,
    TypographyRole::Base2sm,
    TypographyRole::BaseM,
    TypographyRole::Title,
    TypographyRole::Headline,
    TypographyRole::H4,
];

const ELEVATIONS: [Elevation; 4] = [
    Elevation::Base,
    Elevation::Raised,
    Elevation::Floating,
    Elevation::Overlay,
];

/// the token after `now` in `ladder`, or `None` at the top.
fn up<T: Copy + PartialEq>(ladder: &[T], now: T) -> Option<T> {
    let at = ladder.iter().position(|&t| t == now)?;
    ladder.get(at + 1).copied()
}

fn name(c: Option<Component>) -> &'static str {
    c.map_or("container", Component::name)
}

/// `Emphasize`: a text one size up the type ramp (no style is `body`,
/// which goes to `title`); a button or an image one control size up (a
/// button with none goes to `md`, a step above the button's own height;
/// an image with none fills its place and is as large as it gets); a
/// container, a card or a list one elevation up (none is `base`). refused
/// at the top, and for a component no token of emphasis draws.
pub(super) fn emphasize(d: &mut Draft, id: &SemId) -> Result<Inverse, TransformError> {
    let node = d.node_mut(id)?;
    let before = look(node);
    let what = name(node.component);
    let top = || refused(format!("`{id}` is as emphasized as a {what} gets"));
    match node.component {
        Some(Component::Text) => {
            let now = node.props.typography.map_or(3, rung);
            let next = *RAMP.get(now + 1).ok_or_else(top)?;
            node.props.typography = Some(next);
        }
        Some(Component::Button | Component::Image) => {
            let now = match (node.component, node.props.control_size) {
                (_, Some(size)) => up(ControlSize::ALL, size),
                (Some(Component::Button), None) => Some(ControlSize::Md),
                _ => None,
            };
            node.props.control_size = Some(now.ok_or_else(top)?);
        }
        None | Some(Component::Card | Component::List) => {
            let now = match node.props.elevation {
                None | Some(Elevation::Hint) => Elevation::Base,
                Some(e) => e,
            };
            node.props.elevation = Some(up(&ELEVATIONS, now).ok_or_else(top)?);
        }
        Some(_) => {
            return Err(refused(format!(
                "a {what} draws no token of emphasis: `{id}` stays as it is"
            )));
        }
    }
    Ok(Inverse::Looks(vec![before]))
}

/// one step of the spacing scale from `now`, held at its ends.
fn space(now: SpacingToken, step: Step) -> SpacingToken {
    let all = SpacingToken::ALL;
    let at = all.iter().position(|&t| t == now).unwrap_or(0);
    let to = match step {
        Step::Denser => at.saturating_sub(1),
        Step::Sparser => (at + 1).min(all.len() - 1),
    };
    all.get(to).copied().unwrap_or(now)
}

/// `Density`: every gap of a stack or a grid and every padding one step
/// of the spacing scale, where a token is set (with none the component's
/// own spacing stays). refused when nothing moves.
pub(super) fn density(d: &mut Draft, step: Step) -> Result<Inverse, TransformError> {
    let ids = tree::ids(d.root());
    let mut before = Vec::new();
    for id in &ids {
        let node = d.node_mut(id)?;
        let was = look(node);
        if let Some(p) = node.props.padding {
            node.props.padding = Some(space(p, step));
        }
        node.layout = match node.layout {
            Some(Layout::Stack {
                axis,
                gap: Some(g),
                align,
            }) => Some(Layout::Stack {
                axis,
                gap: Some(space(g, step)),
                align,
            }),
            Some(Layout::Grid {
                min_col,
                gap: Some(g),
            }) => Some(Layout::Grid {
                min_col,
                gap: Some(space(g, step)),
            }),
            other => other,
        };
        if node.props != was.props || node.layout != was.layout {
            before.push(was);
        }
    }
    if before.is_empty() {
        let end = match step {
            Step::Denser => "xs",
            Step::Sparser => "xxl",
        };
        return Err(refused(format!(
            "nothing to move: every gap and padding set is at `{end}` already, or none is set"
        )));
    }
    Ok(Inverse::Looks(before))
}

/// the roles a text is drawn in: an ink, not a fill or a line.
const FOREGROUND: [ColorRole; 8] = [
    ColorRole::Text,
    ColorRole::TextDim,
    ColorRole::TextMid,
    ColorRole::Accent,
    ColorRole::Success,
    ColorRole::Danger,
    ColorRole::Warning,
    ColorRole::Info,
];

/// the roles a container is filled with.
const BACKGROUND: [ColorRole; 5] = [
    ColorRole::Bg,
    ColorRole::Surface,
    ColorRole::BgPanel,
    ColorRole::BgHover,
    ColorRole::AccentDim,
];

fn roles(list: &[ColorRole]) -> String {
    let names: Vec<&str> = list.iter().map(|r| r.as_str()).collect();
    names.join(", ")
}

/// `Restyle`: a text's color (`props.color`) or a container's or a
/// list's fill (`props.background`), the two colors the IR sets that a
/// surface draws; the role one of the ones that part takes.
pub(super) fn restyle(
    d: &mut Draft,
    id: &SemId,
    role: ColorRole,
) -> Result<Inverse, TransformError> {
    let node = d.node_mut(id)?;
    let before = look(node);
    let what = name(node.component);
    let (slot, fits, part) = match node.component {
        Some(Component::Text) => (&mut node.props.color, &FOREGROUND[..], "color"),
        None | Some(Component::List) => (&mut node.props.background, &BACKGROUND[..], "fill"),
        Some(_) => {
            return Err(refused(format!(
                "a {what} is drawn in the theme's own colors: no role of the IR restyles it"
            )));
        }
    };
    if !fits.contains(&role) {
        return Err(refused(format!(
            "a {what}'s {part} is one of {}, not `{}`",
            roles(fits),
            role.as_str()
        )));
    }
    if *slot == Some(role) {
        return Err(refused(format!("`{id}` is `{}` already", role.as_str())));
    }
    *slot = Some(role);
    Ok(Inverse::Looks(vec![before]))
}
