//! Focus into view (task-104 front I, WCAG 2.4.11): a control the focus
//! reaches outside the page's window scrolls into it, whoever moved the
//! focus there, at any height of the window; the pointer's focus and a
//! focus already in view leave the page where it is.

use accord::SemId;
use engine::compose::animation::set_reduced_motion;
use engine::control::accessibility::accesskit::{Action, ActionRequest, TreeId};
use engine::control::accessibility::{self, Assist, focus};
use engine::drawing::compositor::Compositor;
use engine::widgets::prelude::WidgetEvent;
use uigallery::views::{EditKey, ShowcaseView};
use winit::keyboard::ModifiersState;

use super::harness::Gallery;

/// Long pages, each with controls under the fold of a short window.
const LONG: &[&str] = &["Input", "Button"];

/// One turn of a 60 Hz display.
const TURN: f32 = 1.0 / 60.0;

/// Reduced motion for the test that holds it, off again at its end.
struct Reduced;

impl Reduced {
    fn on() -> Self {
        set_reduced_motion(true);
        Reduced
    }
}

impl Drop for Reduced {
    fn drop(&mut self) {
        set_reduced_motion(false);
    }
}

/// The gallery on `page` in a window `height` tall, a frame drawn.
fn at(page: &str, height: f32) -> Gallery {
    let mut view = ShowcaseView::new(1280.0, height);
    view.resize(1280.0, height, 1.0);
    view.jump_to_page(page);
    assert_eq!(view.page_title(), page);
    let mut g = Gallery {
        view,
        c: Compositor::new(),
    };
    g.frame();
    g
}

impl Gallery {
    /// The frames of the glide, until the page rests on the focus.
    fn settle(&mut self) {
        for _ in 0..600 {
            if !self.view.revealing() {
                return;
            }
            self.view.wake(TURN);
            self.frame();
        }
        panic!("the page never came to rest");
    }

    /// The page's stops of Tab (the sidebar's aside), in order.
    fn page_order(&self) -> Vec<SemId> {
        let seen = self.view.semantics().unwrap_or_default();
        focus::order(&seen)
            .into_iter()
            .filter(|id| self.view.is_component(id))
            .cloned()
            .collect()
    }

    fn focus_now(&self) -> Option<SemId> {
        self.view.semantics().and_then(|s| s.focused().cloned())
    }

    /// Tab (Shift-Tab with `back`), through the sidebar back to the page.
    fn tab_to_page(&mut self, back: bool) {
        let mods = if back {
            ModifiersState::SHIFT
        } else {
            ModifiersState::empty()
        };
        self.view.set_modifiers(mods);
        for _ in 0..200 {
            self.view.handle_edit_key(EditKey::Tab);
            self.frame();
            self.settle();
            if self.focus_now().is_some_and(|f| self.view.is_component(&f)) {
                break;
            }
        }
        self.view.set_modifiers(ModifiersState::empty());
    }

    /// A screen reader moves its focus to `id` (AccessKit's `Focus`, read
    /// back as the bridge does).
    fn reader_focus(&mut self, id: &SemId) {
        let (_, shown) = self.shown();
        let node = shown.ids.node(id).expect("the reader's tree holds it");
        let req = ActionRequest {
            action: Action::Focus,
            target_tree: TreeId::ROOT,
            target_node: node,
            data: None,
        };
        match accessibility::translate(&req, &shown) {
            Ok(Assist::Focus(to)) => assert!(self.view.assist_focus(&to), "{to}"),
            other => panic!("{id}: {other:?}"),
        }
        self.frame();
        self.settle();
    }

    /// A screen reader asks to see `id` (AccessKit's `ScrollIntoView`,
    /// `AXScrollToVisible` on macOS), read back as the bridge does.
    fn reader_scroll_to(&mut self, id: &SemId) {
        let (_, shown) = self.shown();
        let node = shown.ids.node(id).expect("the reader's tree holds it");
        let req = ActionRequest {
            action: Action::ScrollIntoView,
            target_tree: TreeId::ROOT,
            target_node: node,
            data: None,
        };
        match accessibility::translate(&req, &shown) {
            Ok(Assist::Reveal(to)) => assert!(self.view.assist_reveal(&to), "{to}"),
            other => panic!("{id}: {other:?}"),
        }
        self.frame();
        self.settle();
    }

    /// Whether `id` shows in the page's window: whole, or from its start
    /// when it is taller than the window.
    fn in_view(&self, id: &SemId) -> bool {
        let Some(b) = self.c.binding(id) else {
            return false;
        };
        let [_, y, _, h] = b.rect;
        let Some([_, cy, _, ch]) = b.clip else {
            return true;
        };
        const EPS: f32 = 0.5;
        if h <= ch + EPS {
            y >= cy - EPS && y + h <= cy + ch + EPS
        } else {
            y >= cy - EPS && y < cy + ch
        }
    }
}

/// The last of the page's stops, below the fold of a short window.
fn last_below_fold(g: &Gallery) -> SemId {
    let last = g.page_order().pop().expect("the page has stops");
    assert!(!g.in_view(&last), "{last} is under the fold to begin with");
    last
}

#[test]
fn tab_a_reader_and_an_agent_bring_a_control_under_the_fold_into_view() {
    for page in LONG {
        // the person: Tab until the page's last stop
        let mut person = at(page, 480.0);
        let last = last_below_fold(&person);
        let stops = person.page_order().len();
        for _ in 0..stops {
            person.tab_to_page(false);
            let at_now = person.focus_now().expect("a focus");
            assert!(person.in_view(&at_now), "{page}: Tab to {at_now}");
            if at_now == last {
                break;
            }
        }
        assert_eq!(person.focus_now().as_ref(), Some(&last), "{page}");
        assert!(person.view.page_offset() > 0.0, "{page}: the page scrolled");

        // a screen reader's Focus, straight to it
        let mut reader = at(page, 480.0);
        reader.reader_focus(&last);
        assert!(reader.in_view(&last), "{page}: the reader's focus");

        // an agent: the port has no verb of its own for the focus, so an
        // agent in the process moves it through the hook the bridge uses
        let mut agent = at(page, 480.0);
        assert!(agent.view.assist_focus(&last));
        agent.frame();
        agent.settle();
        assert!(agent.in_view(&last), "{page}: the agent's focus");
        assert_eq!(
            agent.view.page_offset(),
            reader.view.page_offset(),
            "{page}: the same place for the reader and the agent"
        );

        // and back up with Shift-Tab to the first stop
        let first = person.page_order()[0].clone();
        for _ in 0..stops {
            person.tab_to_page(true);
            if person.focus_now().as_ref() == Some(&first) {
                break;
            }
        }
        assert!(person.in_view(&first), "{page}: Shift-Tab back to {first}");
    }
}

#[test]
fn the_page_glides_with_the_theme_s_motion_and_jumps_with_reduced_motion() {
    let mut g = at("Input", 480.0);
    let last = last_below_fold(&g);
    assert!(g.view.assist_focus(&last));
    g.frame();
    assert!(g.view.revealing(), "the glide started");
    assert_eq!(g.view.page_offset(), 0.0, "the frame drew where it was");
    g.view.wake(TURN);
    g.frame();
    let first = g.view.page_offset();
    g.settle();
    assert!(first > 0.0 && first < g.view.page_offset(), "on its way");

    let _reduced = Reduced::on();
    let mut g = at("Input", 480.0);
    assert!(g.view.assist_focus(&last));
    g.frame();
    assert!(!g.view.revealing(), "nothing to glide");
    assert!(g.in_view(&last), "there in the same frame");
}

#[test]
fn a_focus_in_view_or_from_the_pointer_leaves_the_page_where_it_is() {
    let _reduced = Reduced::on();
    let mut g = at("Input", 480.0);
    let first = g.page_order()[0].clone();
    assert!(g.in_view(&first));
    assert!(g.view.assist_focus(&first));
    g.frame();
    assert_eq!(g.view.page_offset(), 0.0, "already in view");

    // the pointer on a control the window cuts at its bottom edge
    let cut_at = |g: &Gallery| {
        g.page_order().into_iter().find(|id| {
            g.c.binding(id).is_some_and(|b| {
                let [_, y, _, h] = b.rect;
                b.clip
                    .is_some_and(|[_, cy, _, ch]| y + 4.0 < cy + ch && y + h > cy + ch)
            })
        })
    };
    let mut found = None;
    for offset in (0..3000).step_by(8) {
        g.view.scroll_page_to(offset as f32);
        g.frame();
        if let Some(id) = cut_at(&g) {
            found = Some(id);
            break;
        }
    }
    let cut = found.expect("a control the fold cuts");
    let before = g.view.page_offset();
    let [x, y, w, _] = g.c.binding(&cut).map(|b| b.rect).expect("bound");
    let (x, y) = (x + w / 2.0, y + 2.0);
    g.view.handle_event(&WidgetEvent::MouseDown { x, y });
    g.view.handle_event(&WidgetEvent::MouseUp { x, y });
    g.frame();
    assert_eq!(g.view.page_offset(), before, "{cut}: the click stays put");
}

#[test]
fn every_stop_shows_at_every_window_height_from_400_to_1200() {
    let _reduced = Reduced::on();
    for page in LONG {
        for height in (400..=1200).step_by(100) {
            let mut g = at(page, height as f32);
            let stops = g.page_order().len();
            let mut seen = 0;
            for _ in 0..stops {
                g.tab_to_page(false);
                let Some(now) = g.focus_now() else { break };
                assert!(g.in_view(&now), "{page} at {height}: {now}");
                seen += 1;
            }
            assert_eq!(seen, stops, "{page} at {height}: every stop");
        }
    }
}

#[test]
fn a_reader_s_scroll_into_view_shows_the_node_and_leaves_the_focus() {
    for page in LONG {
        let mut g = at(page, 480.0);
        let last = last_below_fold(&g);
        let before = g.focus_now();
        g.reader_scroll_to(&last);
        assert!(g.in_view(&last), "{page}: {last} scrolled into view");
        assert_eq!(g.focus_now(), before, "{page}: the focus stays");
        // a node of another page is not this page's to show
        assert!(
            !g.view
                .assist_reveal(&SemId::parse("shell.sidebar").unwrap())
        );
    }
}

/// Across: no page scrolls sideways, so a stop of Tab must never land off
/// the window's sides. A carousel projects only the slides of its page
/// (its arrows page it, the focus never reaches a slide off it), a table
/// drops columns by priority when narrow; at every width from a phone's
/// to a desktop's, each stop shows within the window and its clip.
#[test]
fn across_every_stop_shows_within_the_window_s_width() {
    let _reduced = Reduced::on();
    for page in ["Carousel", "Data Table", "Table"] {
        for width in [360.0_f32, 480.0, 768.0, 1024.0, 1280.0] {
            let mut view = ShowcaseView::new(width, 800.0);
            view.resize(width, 800.0, 1.0);
            view.jump_to_page(page);
            assert_eq!(view.page_title(), page);
            let mut g = Gallery {
                view,
                c: Compositor::new(),
            };
            g.frame();
            let stops = g.page_order().len();
            assert!(stops > 0, "{page}: stops of Tab");
            for _ in 0..stops {
                g.tab_to_page(false);
                let Some(now) = g.focus_now() else { break };
                let Some(b) = g.c.binding(&now) else {
                    continue;
                };
                let [x, _, w, _] = b.rect;
                const EPS: f32 = 0.5;
                assert!(
                    x >= -EPS && x + w <= width + EPS,
                    "{page} at {width}: {now} at x {x}..{} off the window",
                    x + w
                );
                if let Some([cx, _, cw, _]) = b.clip {
                    assert!(
                        x >= cx - EPS && x + w <= cx + cw + EPS,
                        "{page} at {width}: {now} cut by its clip"
                    );
                }
            }
        }
    }
}
