//! selection widgets: the toggle, the toggle group, the button group,
//! the radio group, the calendar, the date picker, and the chip's and
//! the checkbox's selection tokens, one file per widget; the helpers
//! they share are here.

use super::*;
use crate::compose::animation::set_reduced_motion;
use crate::drawing::text::TextMeasurer;
use crate::widgets::selection::toggle::Toggle;
use crate::widgets::selection::{
    ButtonGroup, Calendar, Date, DatePicker, GroupItem, RadioGroup, RadioLayout, SegmentSize,
    ToggleGroup,
};
use accord::{Role, SemId, Snapshot, SnapshotBuilder, Value};

fn id(text: &str) -> SemId {
    SemId::parse(text).unwrap()
}

fn snap(project: impl FnOnce(&mut SnapshotBuilder)) -> Snapshot {
    let mut b = SnapshotBuilder::new();
    project(&mut b);
    b.finish().unwrap()
}

fn nodes(t: &Toggle, bounds: Rect, theme: &Theme) -> Vec<SceneNode> {
    let mut c = Compositor::new();
    c.begin_frame();
    t.render(&mut c, bounds, theme);
    c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec()
}

fn texts(nodes: &[SceneNode]) -> Vec<(String, f32)> {
    nodes
        .iter()
        .filter_map(|n| match n {
            SceneNode::Text { key, x, .. } => Some((key.text.clone(), *x)),
            _ => None,
        })
        .collect()
}

mod button_group;
mod calendar;
mod checkbox;
mod chip;
mod date_picker;
mod radio_group;
mod toggle;
mod toggle_group;
