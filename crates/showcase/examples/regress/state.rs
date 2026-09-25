//! A gallery page in a state, staged for a baseline (task-69 checkpoint
//! b): the page opened as `showcase::app::stage_page` opens it, then a
//! script run through the gallery's public entries only (the port's
//! `act`, as an agent acts, and the Silences page's two staging entries),
//! then settled on the same fixed clock. A state is named, and the name
//! stands for the page in the baseline's key (`--page "<name>"`), so a
//! state is blessed and checked as a page is.
//!
//! The states are the Silences page with the review of the checkpoint
//! open beside the fixed screen: proposed by the rules, and shown from
//! the fixture of the Claude's proposal of the first round
//! (`fixtures/ir/silences-review-claude.ir.json`), no model asked. The
//! frames the page shows are synthetic, made here from their instant, so
//! no ffmpeg runs and the frame is the same on every machine (a layer
//! hashes an image by its size, `hash.rs`).

use engine::compositor::Compositor;
use engine::gpu::image::{ImageHandle, load_image_rgba};
use engine::headless::{Frame, Renderer, Viewport};
use sem::{Actor, Request, Snapshot, Value};
use showcase::app::{RenderError, StagedPage, stage_page};
use showcase::view::ShowcaseView;

/// Animation steps a state may take to settle, and one step: the clock of
/// `stage_page` (120 steps at 60 Hz), so a state settles as a page does.
const SETTLE_STEPS: usize = 120;
const SETTLE_DT: f32 = 1.0 / 60.0;

/// The Claude's review of the checkpoint's first round, as IR.
const CLAUDE_REVIEW: &str = include_str!("../../fixtures/ir/silences-review-claude.ir.json");

/// How a state's surface is proposed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Proposal {
    /// The rules, through the proposing row by `act`.
    Rules,
    /// An IR fixture, verified and shown by the page's staging entry.
    Fixture(&'static str),
}

/// A page in a state: its name, the page it is of, the intent typed in
/// the row, and how its surface is proposed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct State {
    pub name: &'static str,
    pub page: &'static str,
    pub intent: &'static str,
    pub proposal: Proposal,
}

/// Every state a baseline may be of.
pub const STATES: &[State] = &[
    State {
        name: "Silences, review by the rules",
        page: "Silences",
        intent: "the list with the video and the counts",
        proposal: Proposal::Rules,
    },
    State {
        name: "Silences, review by the Claude (fixture)",
        page: "Silences",
        intent: "cut the silences of this video, then I want to review the cuts",
        proposal: Proposal::Fixture(CLAUDE_REVIEW),
    },
];

/// The state named `name`, whatever the case.
pub fn find(name: &str) -> Option<&'static State> {
    STATES.iter().find(|s| s.name.eq_ignore_ascii_case(name))
}

/// A page staged for a baseline: a gallery page at launch, or a state
/// (each holds a whole view, so each is boxed).
pub enum Page {
    Gallery(Box<StagedPage>),
    State(Box<StagedState>),
}

/// A state staged: the view, the scene of its last frame, and how it
/// came to rest.
pub struct StagedState {
    name: &'static str,
    view: ShowcaseView,
    compositor: Compositor,
    steps: usize,
    settled: bool,
}

/// Stage `page` (a gallery page's title or a state's name) under `theme`
/// at `viewport`.
pub fn stage(viewport: Viewport, page: &str, theme: &str) -> Result<Page, String> {
    match find(page) {
        Some(state) => StagedState::stage(state, viewport, theme)
            .map(Box::new)
            .map(Page::State),
        None => stage_page(viewport, page, theme, None)
            .map(Box::new)
            .map(Page::Gallery)
            .map_err(|e| e.to_string()),
    }
}

impl Page {
    /// The page's title as the sidebar shows it, or the state's name.
    pub fn title(&self) -> String {
        match self {
            Page::Gallery(p) => p.title().to_string(),
            Page::State(s) => s.name.to_string(),
        }
    }

    pub fn compositor(&self) -> &Compositor {
        match self {
            Page::Gallery(p) => p.compositor(),
            Page::State(s) => &s.compositor,
        }
    }

    pub fn steps(&self) -> usize {
        match self {
            Page::Gallery(p) => p.steps(),
            Page::State(s) => s.steps,
        }
    }

    pub fn settled(&self) -> bool {
        match self {
            Page::Gallery(p) => p.settled(),
            Page::State(s) => s.settled,
        }
    }

    /// What the page means in this frame; `None` for a page that projects
    /// nothing.
    pub fn semantics(&self) -> Result<Option<Snapshot>, String> {
        match self {
            Page::Gallery(p) => match p.semantics() {
                Ok(s) => Ok(Some(s)),
                Err(RenderError::NoSemantics(_)) => Ok(None),
                Err(e) => Err(e.to_string()),
            },
            Page::State(s) => Ok(s.view.semantics()),
        }
    }

    /// The frame by the window's frame path, as `StagedPage::render`.
    pub fn render(&mut self, renderer: &mut Renderer) -> Result<Frame, String> {
        match self {
            Page::Gallery(p) => p.render(renderer).map_err(|e| e.to_string()),
            Page::State(s) => {
                let clear = s.view.theme.colors.bg.to_linear_array();
                renderer
                    .render(&mut s.compositor, clear)
                    .map_err(|e| e.to_string())
            }
        }
    }
}

impl StagedState {
    fn stage(state: &'static State, viewport: Viewport, theme: &str) -> Result<Self, String> {
        let mut view = ShowcaseView::new(viewport.width, viewport.height);
        view.jump_to_page(state.page);
        if !view.page_title().eq_ignore_ascii_case(state.page) {
            return Err(format!("{}: no page titled '{}'", state.name, state.page));
        }
        view.apply_theme(theme);
        if view.theme_name != theme {
            return Err(format!("no theme named '{theme}'"));
        }
        view.resize(viewport.width, viewport.height, viewport.scale);
        // the frames first: the page never asks the worker for one
        view.stage_silences_frames(&mut synthetic);
        act(
            &mut view,
            "silences_surface.intent",
            "set_value",
            Some(state.intent),
        )?;
        match state.proposal {
            Proposal::Rules => {
                act(
                    &mut view,
                    "silences_surface.proposer",
                    "select",
                    Some("rules"),
                )?;
                act(&mut view, "silences_surface.propose", "propose", None)?;
            }
            Proposal::Fixture(json) => {
                view.stage_silences_ir(json)
                    .map_err(|e| format!("{}: {e}", state.name))?;
            }
        }
        view.stage_silences_frames(&mut synthetic);
        let mut compositor = Compositor::new();
        let (mut steps, mut settled) = (0, false);
        while steps < SETTLE_STEPS && !settled {
            let moving = view.tick(if steps == 0 { 0.0 } else { SETTLE_DT });
            view.render(&mut compositor);
            steps += 1;
            settled = !moving;
        }
        Ok(Self {
            name: state.name,
            view,
            compositor,
            steps,
            settled,
        })
    }
}

/// `action` on `target` with `text`, by the regression harness as a test
/// actor, observed on the whole page as it is now; an error unless it
/// completed.
fn act(
    view: &mut ShowcaseView,
    target: &str,
    action: &str,
    text: Option<&str>,
) -> Result<(), String> {
    let seen = view
        .semantics()
        .ok_or_else(|| "the page projects nothing".to_string())?;
    let actor = Actor::Test {
        name: "regress".to_string(),
    };
    let param = text.map(|t| Value::Text(t.to_string()));
    let req = Request::observe(&seen, actor, target, action, param)
        .map_err(|e| format!("{target} {action}: {e}"))?;
    let receipt = view
        .act(req)
        .ok_or_else(|| format!("{target} {action}: not taken"))?;
    match receipt.stage() {
        Some(sem::Stage::Completed(_)) => Ok(()),
        other => Err(format!("{target} {action}: {other:?}")),
    }
}

/// A frame of the test video, synthetic: seven bars across 320 by 180,
/// the colours of the test pattern, and a band under them whose grey
/// follows the instant, so two instants are two pictures.
fn synthetic(ms: u64) -> Result<ImageHandle, String> {
    const BARS: [[u8; 3]; 7] = [
        [235, 235, 235],
        [235, 235, 16],
        [16, 235, 235],
        [16, 235, 16],
        [235, 16, 235],
        [235, 16, 16],
        [16, 16, 235],
    ];
    let (w, h) = (320u32, 180u32);
    let grey = u8::try_from(40 + (ms / 100) % 200).unwrap_or(120);
    let mut pixels = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let [r, g, b] = if y >= h * 3 / 4 {
                [grey; 3]
            } else {
                BARS[(x * 7 / w) as usize]
            };
            pixels.extend_from_slice(&[r, g, b, 255]);
        }
    }
    load_image_rgba(w, h, pixels).map_err(|e| e.to_string())
}
