//! The comment popups of the kit (task-104 front C, wave 3): the thread
//! beside its pin, the new comment, the quick comment, over every width
//! they are given and in each state their page shows.

use super::*;
use crate::widgets::message::attachment::AttachmentKind;
use crate::widgets::message::comment::{CommentThread, ThreadPart};
use crate::widgets::message::marker::CursorState;
use crate::widgets::message::new_comment::{NewComment, NewCommentPart};
use crate::widgets::message::quick_comment::{QuickComment, QuickPart};
use crate::widgets::message::tokens::CommentTokens;
use accord::Value;

fn themes() -> [Theme; 4] {
    [core(), Theme::hoff(), Theme::dark(), Theme::light()]
}

fn inside(inner: Rect, outer: Rect) -> bool {
    let e = 0.5;
    inner.x >= outer.x - e
        && inner.y >= outer.y - e
        && inner.right() <= outer.right() + e
        && inner.bottom() <= outer.bottom() + e
}

fn conversation() -> CommentThread {
    CommentThread::new(
        vec![
            thread(LUNA),
            Message::new("Sam", "1s", "@Luna Thank, I’m working on it"),
        ],
        "Sam",
    )
}

fn quick_card() -> QuickComment {
    QuickComment::new(
        "Shop House Icon",
        "3D Objects",
        vec![quick(LONG)],
        "Tran Mau Tri Tam",
        "Reply to @randomfash",
    )
    .verified(true)
}

/// The core is the kit verbatim; the others keep its geometry over their
/// own text and paper.
#[test]
fn the_core_is_the_kit_and_the_others_derive_it() {
    let kit = CommentTokens::hoff_core();
    assert_eq!(core().message.comment, kit);
    assert_eq!((kit.popup_w, kit.new.w, kit.quick.w), (280.0, 280.0, 368.0));
    for theme in themes() {
        let t = &theme.message.comment;
        assert_eq!(t.reply.avatar, kit.reply.avatar);
        if *t != kit {
            assert_eq!(t.reply.text.color, theme.colors.text);
            assert_eq!(t.quick.title.color, theme.colors.text);
        }
    }
}

/// The thread over 120 to 700 px: the popup after the pin, as wide as the
/// kit's or what is left, every message and the reply inside it, the line
/// and the submit button inside the well; closed, the pin alone.
#[test]
fn the_thread_holds_over_every_width() {
    for theme in themes() {
        for t in [
            conversation(),
            CommentThread::new(vec![thread(SCRIPTS), thread(LONG)], "Sam"),
        ] {
            for w in (160..=700).step_by(4) {
                let b = Rect::new(10.0, 20.0, w as f32, 0.0);
                let l = t.layout(b, &theme);
                let p = l.popup.expect("open");
                let clear = p.x >= l.pin.right() || p.y >= l.pin.bottom();
                assert!(clear, "{w}: beside the pin or under it");
                assert!(p.right() <= b.right() + 0.5, "{w}: in the column");
                assert!(p.w <= theme.message.comment.popup_w);
                for r in &l.rows {
                    assert!(inside(*r, p), "{w}: a message out of the popup");
                }
                assert!(inside(l.well, p) && inside(l.line, l.well) && inside(l.send, l.well));
                let (sw, sh) = t.size(w as f32, &theme);
                let bottom = p.bottom().max(l.pin.bottom());
                let want = (p.right().max(l.pin.right()) - b.x, bottom - b.y);
                assert!(
                    (sw - want.0).abs() < 0.01 && (sh - want.1).abs() < 0.01,
                    "{w}"
                );
            }
        }
    }
    let mut t = conversation();
    t.set_open(false);
    let theme = core();
    let pin = t.layout(at(400.0), &theme).pin;
    assert_eq!(t.size(400.0, &theme), (pin.w, pin.h));
}

/// A reply sent ends the thread as the replier's message and empties the
/// line; the pin opens and closes it and reads it.
#[test]
fn the_thread_sends_a_reply_and_its_pin_reads_it() {
    let id = SemId::parse("t").unwrap();
    let mut t = CommentThread::new(vec![thread(LUNA)], "Sam")
        .unread(true)
        .sem(id);
    assert_eq!(
        t.act(ThreadPart::Send, "activate", None),
        Ok(()),
        "nothing sent"
    );
    assert_eq!(t.messages().len(), 1);
    let text = Value::Text("On it".into());
    assert_eq!(t.act(ThreadPart::Reply, "set_value", Some(&text)), Ok(()));
    assert_eq!(t.act(ThreadPart::Send, "activate", None), Ok(()));
    assert_eq!(t.messages().len(), 2);
    assert_eq!((t.messages()[1].author(), t.reply().text()), ("Sam", ""));
    assert_eq!(t.act(ThreadPart::Close, "activate", None), Ok(()));
    assert!(!t.is_open());
    assert_eq!(t.act(ThreadPart::Pin, "activate", None), Ok(()));
    assert!(t.is_open() && !t.is_unread(), "the pin opened and read it");
    for part in [
        ThreadPart::Pin,
        ThreadPart::Close,
        ThreadPart::Message(1),
        ThreadPart::Reply,
        ThreadPart::Send,
    ] {
        let pid = t.part_id(part).unwrap();
        assert_eq!(t.part_of(&pid), Some(part));
    }
    let mut b = SnapshotBuilder::new();
    t.project(&mut b, t.sem_id().unwrap(), "Comment thread", None);
    b.finish().expect("the projection validates");
}

/// The new comment: closed at rest with the pinned cursor, open once
/// written in with the typing one, the uploading one while a picture
/// uploads; its tools attach, mention and add an emoji; it posts and
/// empties; it holds over every width.
#[test]
fn the_new_comment_opens_attaches_and_posts() {
    let theme = core();
    let mut n = NewComment::new().sem(SemId::parse("n").unwrap());
    assert!(!n.is_open());
    assert_eq!(n.cursor_state(), CursorState::Pinned);
    let closed = n.size(400.0, &theme).1;
    assert!(
        n.act(NewCommentPart::Tool(0), "activate", None).is_err(),
        "closed"
    );
    let text = Value::Text("Nice".into());
    assert_eq!(
        n.act(NewCommentPart::Text, "set_value", Some(&text)),
        Ok(())
    );
    assert_eq!(n.cursor_state(), CursorState::Typing);
    assert!(n.size(400.0, &theme).1 > closed, "it opens");
    assert_eq!(n.act(NewCommentPart::Tool(0), "activate", None), Ok(()));
    assert_eq!(n.act(NewCommentPart::Tool(1), "activate", None), Ok(()));
    assert_eq!(n.line().text(), "Nice@");
    assert_eq!(n.tray().items().len(), 1);
    assert_eq!(n.act(NewCommentPart::Send, "activate", None), Ok(()));
    assert_eq!((n.posted(), n.is_open()), (1, false));
    assert_eq!(
        NewComment::new().uploading(true).cursor_state(),
        CursorState::Uploading
    );
    for theme in themes() {
        let n = NewComment::new()
            .with_text("Two angles")
            .with_file("Front", "PNG", AttachmentKind::Image(None))
            .with_file("Brief.pdf", "PDF · 1.2 MB", AttachmentKind::File);
        for w in (168..=700).step_by(4) {
            let b = Rect::new(10.0, 20.0, w as f32, 0.0);
            let l = n.layout(b, &theme);
            assert!(
                !inside(l.cursor, l.frame),
                "{w}: the cursor clear of the box"
            );
            assert!(l.frame.right() <= b.right() + 0.5, "{w}");
            for r in l
                .tray
                .iter()
                .chain(&l.tools)
                .chain(l.send.iter())
                .chain([&l.line])
            {
                assert!(inside(*r, l.frame), "{w}: {r:?}");
            }
        }
    }
}

/// The quick comment over every width: the card the kit's or the column,
/// every part inside it; `Send` posts the reply as the replier's comment;
/// every part's id names it back.
#[test]
fn the_quick_comment_holds_and_posts() {
    for theme in themes() {
        let q = quick_card();
        for w in (200..=700).step_by(4) {
            let b = Rect::new(10.0, 20.0, w as f32, 0.0);
            let l = q.layout(b, &theme);
            assert_eq!(l.card.w, (w as f32).min(theme.message.comment.quick.w));
            let parts = [l.close, l.title, l.preview, l.avatar, l.line, l.send];
            for r in parts.iter().chain(&l.comments).chain(&l.tools) {
                assert!(inside(*r, l.card), "{w}: {r:?}");
            }
            assert!(inside(l.picture, l.preview));
        }
    }
    let mut q = quick_card().sem(SemId::parse("q").unwrap());
    let text = Value::Text("Agreed".into());
    assert_eq!(q.act(QuickPart::Reply, "set_value", Some(&text)), Ok(()));
    assert_eq!(q.act(QuickPart::Send, "activate", None), Ok(()));
    assert_eq!(q.comments().len(), 2);
    assert!(q.comments()[1].is_verified());
    for part in [
        QuickPart::Close,
        QuickPart::Comment(0),
        QuickPart::Like(0),
        QuickPart::Reply,
        QuickPart::Tool(3),
        QuickPart::Send,
    ] {
        let pid = q.part_id(part).unwrap();
        assert_eq!(q.part_of(&pid), Some(part), "{pid}");
    }
    let mut b = SnapshotBuilder::new();
    q.project(&mut b, q.sem_id().unwrap(), None);
    b.finish().expect("the projection validates");
}
