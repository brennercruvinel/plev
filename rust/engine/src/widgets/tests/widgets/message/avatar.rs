//! The avatar (front C of task-104): the kit's ladder at each theme's
//! control `Md`, the disc without a photo, and the stack of people over
//! every width it is given.

use super::*;
use crate::widgets::content::{AvatarStack, StackLayout};

fn themes() -> [Theme; 4] {
    [core(), Theme::hoff(), Theme::dark(), Theme::light()]
}

fn people(n: usize) -> Vec<Avatar> {
    (0..n)
        .map(|i| Avatar::new(format!("person {i}")).online(i == 0))
        .collect()
}

fn stack(n: usize) -> AvatarStack {
    AvatarStack::new(people(n)).sem(id("page.stack"))
}

/// The core draws the kit's edges (CommentPin `size-6` to ShareProfile
/// `size-20`); another theme the same shares of its control `Md`, so its
/// `Md` is its control and its `Xl` two of them, as before the ladder.
#[test]
fn the_core_draws_the_kit_ladder_and_a_theme_its_shares_of_its_control() {
    let t = core();
    let px = AvatarSize::ALL.map(|s| s.px(&t));
    assert_eq!(px, [24.0, 32.0, 40.0, 48.0, 80.0]);
    for t in themes() {
        let md = t.control.height(crate::styling::theme::ControlSize::Md);
        assert_eq!(AvatarSize::Md.px(&t), md);
        assert_eq!(AvatarSize::Xl.px(&t), md * 2.0);
    }
}

/// Without a photo: the kit's `#F8F7F7` disc in the core, the initial at
/// 7:1 on it and the online dot at 3:1 on the paper, under every theme.
#[test]
fn the_initial_reads_on_its_disc_and_the_dot_on_the_paper() {
    assert_eq!(core().message.avatar.fill, SHADES[1], "UploadAvatar:8");
    for t in themes() {
        let a = &t.message.avatar;
        assert!(ratio(a.initial, a.fill) >= 7.0, "{:?}", a.initial);
        assert!(ratio(a.online_fill, a.ring) >= 3.0, "{:?}", a.online_fill);
        let mut c = Compositor::new();
        c.begin_frame();
        Avatar::new("ann").render(&mut c, Rect::new(0.0, 0.0, 40.0, 40.0), &t);
        let drawn = c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec();
        assert!(drawn.iter().any(|n| matches!(
            n,
            SceneNode::Text { color, .. } if *color == a.initial.0
        )));
    }
}

/// The kit's stack: discs 32 apart by 24 (`-ml-2`), each in a 2 px paper
/// ring, the first drawn last so it sits on top.
#[test]
fn a_stack_slides_each_disc_under_the_one_before() {
    let t = core();
    let s = stack(3);
    assert_eq!(s.preferred_size(&t), (32.0 + 2.0 * 24.0, 32.0));
    let l = s.layout(Rect::new(0.0, 0.0, 400.0, 32.0), &t);
    assert_eq!(l.people.len(), 3);
    assert_eq!(l.people[1].x - l.people[0].x, 24.0);
    assert!(l.more.is_none());
    let mut c = Compositor::new();
    c.begin_frame();
    s.render(&mut c, Rect::new(0.0, 0.0, 400.0, 32.0), &t);
    let drawn = texts(c.layer(LayerId::DEFAULT).unwrap().nodes());
    assert_eq!(drawn, ["P", "P", "P"]);
    let first = c.binding(&id("page.stack.avatar[0]")).unwrap().rect;
    assert_eq!(first[2], 32.0 - 2.0 * t.message.avatar.stack_ring);
}

/// Narrower than everyone, the last disc that fits counts the rest; the
/// stack never leaves its bounds and loses nobody, from no room to all.
#[test]
fn a_narrow_stack_counts_the_rest_and_stays_in_its_bounds() {
    for t in themes() {
        for n in [0, 1, 2, 5, 12] {
            let s = stack(n);
            for w in (0..=480).step_by(4) {
                let bounds = Rect::new(8.0, 4.0, w as f32, 40.0);
                let StackLayout { rect, people, more } = s.layout(bounds, &t);
                assert!(rect.right() <= bounds.right() + 0.01, "{n} in {w}");
                let held = people.len() + more.map_or(0, |(_, k)| k);
                let px = AvatarSize::Sm.px(&t);
                if w as f32 >= px {
                    assert_eq!(held, n, "{n} in {w}: everyone is shown or counted");
                }
                if let Some((slot, k)) = more {
                    assert!(k >= 2 || people.is_empty(), "a count of one is a person");
                    assert!(slot.right() <= bounds.right() + 0.01);
                }
            }
        }
    }
    let t = core();
    let l = stack(12).layout(Rect::new(0.0, 0.0, 100.0, 32.0), &t);
    assert_eq!((l.people.len(), l.more.map(|m| m.1)), (2, Some(10)));
}

/// A reader hears everyone: a list of images named by the people, online
/// or not, with no action; those the count holds are placed on it.
#[test]
fn a_stack_projects_everyone_and_places_the_counted_on_the_count() {
    let t = core();
    let s = stack(5);
    let bounds = Rect::new(0.0, 0.0, 80.0, 32.0);
    let mut c = Compositor::new();
    c.begin_frame();
    s.render(&mut c, bounds, &t);
    let (slot, _) = s.layout(bounds, &t).more.expect("a count");
    let last = c.binding(&id("page.stack.avatar[4]")).unwrap().rect;
    assert!(slot.contains(last[0] + 1.0, last[1] + 1.0));
    let root = id("page");
    let seen = snap(|b| {
        b.node(&root, Role::Region, "Avatar");
        s.project(b, &id("page.stack"), Some(&root), "Reviewers");
    });
    let list = seen.get(&id("page.stack")).unwrap();
    assert_eq!((list.role, list.label.as_str()), (Role::List, "Reviewers"));
    assert_eq!(seen.children(&id("page.stack")).count(), 5);
    let first = seen.get(&id("page.stack.avatar[0]")).unwrap();
    assert_eq!(
        (first.role, first.label.as_str()),
        (Role::Image, "person 0")
    );
    assert!(format!("{:?}", first.state).contains("true"));
    assert!(first.actions.is_empty() && first.focusable);
}

/// A reader's focus rings the disc, and only while it is there.
#[test]
fn the_focus_ring_rings_the_disc_only_while_focused() {
    let t = core();
    let count = |a: &Avatar| {
        let mut c = Compositor::new();
        c.begin_frame();
        a.render(&mut c, Rect::new(0.0, 0.0, 40.0, 40.0), &t);
        c.layer(LayerId::DEFAULT).unwrap().nodes().len()
    };
    let mut a = Avatar::new("ann");
    let rest = count(&a);
    a.set_focused(true);
    assert_eq!(count(&a), rest + 1);
}
