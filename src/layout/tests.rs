use super::*;

fn leaf(style: LayoutStyle) -> LayoutItem {
    LayoutItem {
        style,
        children: vec![],
        text: None,
    }
}

fn container(style: LayoutStyle, children: Vec<usize>) -> LayoutItem {
    LayoutItem {
        style,
        children,
        text: None,
    }
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

// -- Percentage dimensions (parallel width_percent/height_percent fields) --

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

// -- Text measurement integration (taffy measure function) --

fn text_leaf(content: &str, font_size: f32) -> LayoutItem {
    LayoutItem {
        style: LayoutStyle::default(),
        children: vec![],
        text: Some(TextMeasureSpec {
            content: content.to_string(),
            style: crate::text::TextStyle::new(font_size).with_family("Inter"),
            max_width: None,
        }),
    }
}

#[test]
fn test_text_leaf_gets_measured_size() {
    let mut engine = LayoutEngine::new();
    let items = vec![
        container(
            LayoutStyle {
                direction: Direction::Row,
                align: Align::Start,
                ..Default::default()
            },
            vec![1],
        ),
        text_leaf("Hello, World!", 16.0),
    ];
    let bounds = engine.compute(&items, 800.0, 600.0);

    let (w, h) = crate::text::TextMeasurer::measure_styled(
        "Hello, World!",
        &crate::text::TextStyle::new(16.0).with_family("Inter"),
        None,
    );
    assert!(
        (bounds[1].width - w).abs() < 1.0,
        "{} vs {}",
        bounds[1].width,
        w
    );
    assert!(
        (bounds[1].height - h).abs() < 1.0,
        "{} vs {}",
        bounds[1].height,
        h
    );
}

#[test]
fn test_text_leaf_proportional_widths_differ() {
    let mut engine = LayoutEngine::new();
    let items = vec![
        container(
            LayoutStyle {
                direction: Direction::Column,
                align: Align::Start,
                ..Default::default()
            },
            vec![1, 2],
        ),
        text_leaf("iiii", 16.0),
        text_leaf("WWWW", 16.0),
    ];
    let bounds = engine.compute(&items, 800.0, 600.0);
    assert!(
        bounds[1].width < bounds[2].width,
        "'iiii' ({}) must lay out narrower than 'WWWW' ({})",
        bounds[1].width,
        bounds[2].width
    );
}

#[test]
fn test_text_leaf_wraps_in_narrow_container() {
    let mut engine = LayoutEngine::new();
    let long = "the quick brown fox jumps over the lazy dog";
    let items = vec![
        container(
            LayoutStyle {
                direction: Direction::Column,
                align: Align::Stretch,
                width: Some(80.0),
                ..Default::default()
            },
            vec![1],
        ),
        text_leaf(long, 16.0),
    ];
    let bounds = engine.compute(&items, 800.0, 600.0);

    let style = crate::text::TextStyle::new(16.0).with_family("Inter");
    let (_, single_line_h) = crate::text::TextMeasurer::measure_styled("a", &style, None);
    assert!(bounds[1].width <= 80.0 + 0.5);
    assert!(
        bounds[1].height >= single_line_h * 2.0,
        "text in an 80px container must wrap to multiple lines (h={})",
        bounds[1].height
    );
}
