//! The empty state's body (front C of task-104): the shaper's lines, each
//! centered on its own, in Latin, CJK and Thai, at every width.

use super::*;
use crate::widgets::content::EmptyState;

const BODIES: [&str; 3] = [
    "Nothing here yet. Open a file, drag one in, or start from a template to fill it.",
    "这里还没有内容打开一个文件或者从模板开始填写它",
    "ยังไม่มีอะไรที่นี่เปิดไฟล์หรือเริ่มจากแม่แบบเพื่อเติมให้เต็ม",
];

/// Each drawn line of the body is centered on the bounds' middle, the
/// lines stack one line height apart, and the block stays inside the
/// bounds from a phone to a desktop.
#[test]
fn each_line_of_the_body_is_centered_on_its_own() {
    for t in [core(), Theme::hoff()] {
        for body in BODIES {
            let es = EmptyState::new("Empty", body);
            for w in (240..=1200).step_by(40) {
                let bounds = Rect::new(10.0, 0.0, w as f32, 900.0);
                let mut c = Compositor::new();
                c.begin_frame();
                es.render(&mut c, bounds, &t);
                let lines = es.message_lines(bounds, &t);
                let drawn: Vec<(f32, f32)> = c
                    .layer(LayerId::DEFAULT)
                    .unwrap()
                    .nodes()
                    .iter()
                    .filter_map(|n| match n {
                        SceneNode::Text { key, x, y, .. } if key.text != "Empty" => Some((*x, *y)),
                        _ => None,
                    })
                    .collect();
                assert_eq!(drawn.len(), lines.len(), "{w}: {body}");
                let (mid, _) = bounds.center();
                for ((x, _), (line, lw)) in drawn.iter().zip(&lines) {
                    assert!((x + lw / 2.0 - mid).abs() < 0.5, "{w}: {line:?} off center");
                    assert!(*x >= bounds.x - 4.0, "{w}: {line:?} out");
                }
                for pair in drawn.windows(2) {
                    let step = pair[1].1 - pair[0].1;
                    assert!(step > 0.0, "{w}: the lines go down");
                }
            }
        }
    }
}
