//! Percentage dimensions (parallel width_percent/height_percent fields).

use super::{container, leaf};
use crate::layout::*;

#[test]
fn test_width_percent_of_parent() {
    let mut engine = LayoutEngine::new();
    let items = vec![
        container(
            LayoutStyle {
                direction: Direction::Row,
                width: Some(1000.0),
                height: Some(100.0),
                ..Default::default()
            },
            vec![1],
        ),
        leaf(LayoutStyle {
            width_percent: Some(0.5),
            ..Default::default()
        }),
    ];
    let bounds = engine.compute(&items, 1000.0, 600.0);

    assert_eq!(bounds[1].width, 500.0); // 50% of the 1000px container
}

#[test]
fn test_height_percent_of_parent() {
    let mut engine = LayoutEngine::new();
    let items = vec![
        container(
            LayoutStyle {
                direction: Direction::Column,
                width: Some(200.0),
                height: Some(400.0),
                ..Default::default()
            },
            vec![1],
        ),
        leaf(LayoutStyle {
            height_percent: Some(0.25),
            ..Default::default()
        }),
    ];
    let bounds = engine.compute(&items, 800.0, 600.0);

    assert_eq!(bounds[1].height, 100.0); // 25% of the 400px container
}

#[test]
fn test_percent_wins_over_fixed_px() {
    let mut engine = LayoutEngine::new();
    let items = vec![
        container(
            LayoutStyle {
                direction: Direction::Row,
                width: Some(1000.0),
                height: Some(100.0),
                ..Default::default()
            },
            vec![1],
        ),
        leaf(LayoutStyle {
            width: Some(120.0),
            width_percent: Some(0.5),
            ..Default::default()
        }),
    ];
    let bounds = engine.compute(&items, 1000.0, 600.0);

    assert_eq!(bounds[1].width, 500.0);
}

#[test]
fn test_percent_tracks_parent_resize() {
    let mut engine = LayoutEngine::new();
    let items = |parent_w: f32| {
        vec![
            container(
                LayoutStyle {
                    direction: Direction::Row,
                    width: Some(parent_w),
                    height: Some(100.0),
                    ..Default::default()
                },
                vec![1],
            ),
            leaf(LayoutStyle {
                width_percent: Some(0.5),
                ..Default::default()
            }),
        ]
    };

    assert_eq!(engine.compute(&items(1000.0), 1000.0, 600.0)[1].width, 500.0);
    assert_eq!(engine.compute(&items(600.0), 1000.0, 600.0)[1].width, 300.0);
}
