use super::*;

fn leaf(style: LayoutStyle) -> LayoutItem {
    LayoutItem {
        style,
        children: vec![],
    }
}

fn container(style: LayoutStyle, children: Vec<usize>) -> LayoutItem {
    LayoutItem { style, children }
}

#[test]
fn test_single_node_fills_viewport() {
    let mut engine = LayoutEngine::new();
    // Root with explicit size fills viewport
    let items = vec![leaf(LayoutStyle {
        width: Some(800.0),
        height: Some(600.0),
        ..Default::default()
    })];
    let bounds = engine.compute(&items, 800.0, 600.0);

    assert_eq!(bounds.len(), 1);
    assert_eq!(bounds[0].x, 0.0);
    assert_eq!(bounds[0].y, 0.0);
    assert_eq!(bounds[0].width, 800.0);
    assert_eq!(bounds[0].height, 600.0);
}

#[test]
fn test_vertical_stack() {
    let mut engine = LayoutEngine::new();
    let items = vec![
        // 0: root container (column)
        container(
            LayoutStyle {
                direction: Direction::Column,
                width: Some(400.0),
                height: Some(300.0),
                ..Default::default()
            },
            vec![1, 2, 3],
        ),
        // 1, 2, 3: children with fixed height
        leaf(LayoutStyle {
            height: Some(50.0),
            ..Default::default()
        }),
        leaf(LayoutStyle {
            height: Some(50.0),
            ..Default::default()
        }),
        leaf(LayoutStyle {
            height: Some(50.0),
            ..Default::default()
        }),
    ];
    let bounds = engine.compute(&items, 800.0, 600.0);

    // Children stack vertically
    assert_eq!(bounds[1].y, 0.0);
    assert_eq!(bounds[2].y, 50.0);
    assert_eq!(bounds[3].y, 100.0);
    // All children fill parent width (stretch default)
    assert_eq!(bounds[1].width, 400.0);
    assert_eq!(bounds[2].width, 400.0);
    assert_eq!(bounds[3].width, 400.0);
}

#[test]
fn test_horizontal_stack() {
    let mut engine = LayoutEngine::new();
    let items = vec![
        container(
            LayoutStyle {
                direction: Direction::Row,
                width: Some(300.0),
                height: Some(100.0),
                ..Default::default()
            },
            vec![1, 2, 3],
        ),
        leaf(LayoutStyle {
            width: Some(80.0),
            ..Default::default()
        }),
        leaf(LayoutStyle {
            width: Some(80.0),
            ..Default::default()
        }),
        leaf(LayoutStyle {
            width: Some(80.0),
            ..Default::default()
        }),
    ];
    let bounds = engine.compute(&items, 800.0, 600.0);

    assert_eq!(bounds[1].x, 0.0);
    assert_eq!(bounds[2].x, 80.0);
    assert_eq!(bounds[3].x, 160.0);
    // Children fill parent height in Row direction (stretch cross axis)
    assert_eq!(bounds[1].height, 100.0);
}

#[test]
fn test_padding() {
    let mut engine = LayoutEngine::new();
    let items = vec![
        container(
            LayoutStyle {
                direction: Direction::Column,
                width: Some(200.0),
                height: Some(200.0),
                padding: [10.0, 20.0, 10.0, 20.0], // top, right, bottom, left
                ..Default::default()
            },
            vec![1],
        ),
        leaf(LayoutStyle {
            height: Some(50.0),
            ..Default::default()
        }),
    ];
    let bounds = engine.compute(&items, 800.0, 600.0);

    // Child offset by padding
    assert_eq!(bounds[1].x, 20.0); // left padding
    assert_eq!(bounds[1].y, 10.0); // top padding
    // Child width = parent - left - right padding
    assert_eq!(bounds[1].width, 160.0); // 200 - 20 - 20
}

#[test]
fn test_gap() {
    let mut engine = LayoutEngine::new();
    let items = vec![
        container(
            LayoutStyle {
                direction: Direction::Column,
                width: Some(200.0),
                height: Some(300.0),
                gap: 10.0,
                ..Default::default()
            },
            vec![1, 2],
        ),
        leaf(LayoutStyle {
            height: Some(50.0),
            ..Default::default()
        }),
        leaf(LayoutStyle {
            height: Some(50.0),
            ..Default::default()
        }),
    ];
    let bounds = engine.compute(&items, 800.0, 600.0);

    assert_eq!(bounds[1].y, 0.0);
    assert_eq!(bounds[2].y, 60.0); // 50 + 10 gap
}

#[test]
fn test_fixed_size() {
    let mut engine = LayoutEngine::new();
    let items = vec![leaf(LayoutStyle {
        width: Some(200.0),
        height: Some(100.0),
        ..Default::default()
    })];
    let bounds = engine.compute(&items, 800.0, 600.0);

    assert_eq!(bounds[0].width, 200.0);
    assert_eq!(bounds[0].height, 100.0);
}

#[test]
fn test_flex_grow() {
    let mut engine = LayoutEngine::new();
    let items = vec![
        container(
            LayoutStyle {
                direction: Direction::Row,
                width: Some(300.0),
                height: Some(100.0),
                ..Default::default()
            },
            vec![1, 2],
        ),
        leaf(LayoutStyle {
            flex_grow: 1.0,
            ..Default::default()
        }),
        leaf(LayoutStyle {
            flex_grow: 2.0,
            ..Default::default()
        }),
    ];
    let bounds = engine.compute(&items, 800.0, 600.0);

    // flex_grow 1:2 ratio splits 300px into 100 + 200
    assert_eq!(bounds[1].width, 100.0);
    assert_eq!(bounds[2].width, 200.0);
}

#[test]
fn test_alignment_center() {
    let mut engine = LayoutEngine::new();
    let items = vec![
        container(
            LayoutStyle {
                direction: Direction::Column,
                align: Align::Center,
                width: Some(200.0),
                height: Some(200.0),
                ..Default::default()
            },
            vec![1],
        ),
        leaf(LayoutStyle {
            width: Some(80.0),
            height: Some(40.0),
            ..Default::default()
        }),
    ];
    let bounds = engine.compute(&items, 800.0, 600.0);

    // Centered: (200 - 80) / 2 = 60
    assert_eq!(bounds[1].x, 60.0);
}

#[test]
fn test_justify_space_between() {
    let mut engine = LayoutEngine::new();
    let items = vec![
        container(
            LayoutStyle {
                direction: Direction::Column,
                justify: Justify::SpaceBetween,
                width: Some(200.0),
                height: Some(200.0),
                ..Default::default()
            },
            vec![1, 2],
        ),
        leaf(LayoutStyle {
            height: Some(40.0),
            ..Default::default()
        }),
        leaf(LayoutStyle {
            height: Some(40.0),
            ..Default::default()
        }),
    ];
    let bounds = engine.compute(&items, 800.0, 600.0);

    // space-between: first at top, last at bottom
    assert_eq!(bounds[1].y, 0.0);
    assert_eq!(bounds[2].y, 160.0); // 200 - 40
}

#[test]
fn test_empty_items() {
    let mut engine = LayoutEngine::new();
    let bounds = engine.compute(&[], 800.0, 600.0);
    assert!(bounds.is_empty());
}
