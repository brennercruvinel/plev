//! FocusGraph -- spatial + sequential focus navigation.

use crate::input::{HitRegion, ViewId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusDirection {
    Next,
    Previous,
    Up,
    Down,
    Left,
    Right,
}

pub struct FocusGraph {
    focusable_views: Vec<(ViewId, [f32; 4])>, // id + bounds [x, y, w, h]
}

impl FocusGraph {
    pub fn from_hit_regions(regions: &[HitRegion]) -> Self {
        let focusable_views: Vec<_> = regions
            .iter()
            .filter(|r| r.focusable)
            .map(|r| (r.view_id, [r.x, r.y, r.w, r.h]))
            .collect();
        Self { focusable_views }
    }

    pub fn next(&self, current: ViewId, direction: FocusDirection) -> Option<ViewId> {
        if self.focusable_views.is_empty() {
            return None;
        }

        let current_idx = self
            .focusable_views
            .iter()
            .position(|(id, _)| *id == current);

        match direction {
            FocusDirection::Next => {
                let idx = current_idx
                    .map(|i| (i + 1) % self.focusable_views.len())
                    .unwrap_or(0);
                Some(self.focusable_views[idx].0)
            }
            FocusDirection::Previous => {
                let idx = current_idx
                    .map(|i| {
                        if i == 0 {
                            self.focusable_views.len() - 1
                        } else {
                            i - 1
                        }
                    })
                    .unwrap_or(self.focusable_views.len() - 1);
                Some(self.focusable_views[idx].0)
            }
            FocusDirection::Up
            | FocusDirection::Down
            | FocusDirection::Left
            | FocusDirection::Right => {
                let current_bounds = current_idx.map(|i| self.focusable_views[i].1)?;
                let cx = current_bounds[0] + current_bounds[2] / 2.0;
                let cy = current_bounds[1] + current_bounds[3] / 2.0;

                let mut best: Option<(ViewId, f32)> = None;
                for &(id, bounds) in &self.focusable_views {
                    if id == current {
                        continue;
                    }
                    let ox = bounds[0] + bounds[2] / 2.0;
                    let oy = bounds[1] + bounds[3] / 2.0;
                    let dx = ox - cx;
                    let dy = oy - cy;

                    let in_direction = match direction {
                        FocusDirection::Up => dy < 0.0,
                        FocusDirection::Down => dy > 0.0,
                        FocusDirection::Left => dx < 0.0,
                        FocusDirection::Right => dx > 0.0,
                        _ => unreachable!(),
                    };
                    if !in_direction {
                        continue;
                    }

                    let dist = dx * dx + dy * dy;
                    if best.is_none() || dist < best.unwrap().1 {
                        best = Some((id, dist));
                    }
                }
                best.map(|(id, _)| id)
            }
        }
    }

    pub fn first(&self) -> Option<ViewId> {
        self.focusable_views.first().map(|(id, _)| *id)
    }
}
