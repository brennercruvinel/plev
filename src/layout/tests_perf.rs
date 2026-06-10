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
fn test_nested_containers() {
    let mut engine = LayoutEngine::new();
    let items = vec![
        container(
            LayoutStyle {
                direction: Direction::Column,
                width: Some(400.0),
                height: Some(400.0),
                ..Default::default()
            },
            vec![1, 4],
        ),
        container(
            LayoutStyle {
                direction: Direction::Row,
                height: Some(100.0),
                ..Default::default()
            },
            vec![2, 3],
        ),
        leaf(LayoutStyle {
            width: Some(50.0),
            ..Default::default()
        }),
        leaf(LayoutStyle {
            flex_grow: 1.0,
            ..Default::default()
        }),
        leaf(LayoutStyle {
            flex_grow: 1.0,
            ..Default::default()
        }),
    ];
    let bounds = engine.compute(&items, 800.0, 600.0);

    assert_eq!(bounds[1].y, 0.0);
    assert_eq!(bounds[1].height, 100.0);
    assert_eq!(bounds[1].width, 400.0);
    assert_eq!(bounds[2].x, 0.0);
    assert_eq!(bounds[2].width, 50.0);
    assert_eq!(bounds[3].x, 50.0);
    assert_eq!(bounds[3].width, 350.0);
    assert_eq!(bounds[4].y, 100.0);
    assert_eq!(bounds[4].height, 300.0);
}

#[test]
fn test_1000_nodes_under_1ms() {
    let mut engine = LayoutEngine::new();
    let mut items = Vec::with_capacity(1001);

    let root_children: Vec<usize> = (1..=100).collect();
    items.push(container(
        LayoutStyle {
            direction: Direction::Column,
            width: Some(1920.0),
            height: Some(1080.0),
            ..Default::default()
        },
        root_children,
    ));

    for i in 0..100 {
        let base = 101 + i * 9;
        let children: Vec<usize> = (base..base + 9).collect();
        items.push(container(
            LayoutStyle {
                direction: Direction::Row,
                height: Some(10.0),
                ..Default::default()
            },
            children,
        ));
    }

    for _ in 0..900 {
        items.push(leaf(LayoutStyle {
            flex_grow: 1.0,
            ..Default::default()
        }));
    }

    assert_eq!(items.len(), 1001);

    let start = web_time::Instant::now();
    let bounds = engine.compute(&items, 1920.0, 1080.0);
    let elapsed = start.elapsed();

    assert_eq!(bounds.len(), 1001);
    let threshold = if cfg!(debug_assertions) {
        std::time::Duration::from_millis(50)
    } else {
        std::time::Duration::from_millis(1)
    };
    assert!(
        elapsed < threshold,
        "Layout of 1001 nodes took {:?} (> {:?})",
        elapsed,
        threshold
    );
}
