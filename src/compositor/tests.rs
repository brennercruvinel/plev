use super::*;

#[test]
fn default_layer_exists() {
    let comp = Compositor::new();
    assert_eq!(comp.layers().len(), 1);
    assert_eq!(comp.layers()[0].id, LayerId::DEFAULT);
}

#[test]
fn create_and_remove_layers() {
    let mut comp = Compositor::new();
    let l1 = comp.create_layer(10);
    let l2 = comp.create_layer(5);
    assert_eq!(comp.layers().len(), 3);

    comp.remove_layer(l1);
    assert_eq!(comp.layers().len(), 2);
    assert!(comp.layer(l1).is_none());
    assert!(comp.layer(l2).is_some());
}

#[test]
fn cannot_remove_default_layer() {
    let mut comp = Compositor::new();
    comp.remove_layer(LayerId::DEFAULT);
    assert_eq!(comp.layers().len(), 1);
    assert!(comp.layer(LayerId::DEFAULT).is_some());
}

#[test]
fn z_order_sorting() {
    let mut comp = Compositor::new();
    let _l1 = comp.create_layer(10);
    let _l2 = comp.create_layer(-5);
    let _l3 = comp.create_layer(5);

    comp.sorted = false;
    comp.layers.sort_by_key(|l| l.z_order);

    let z_orders: Vec<i32> = comp.layers().iter().map(|l| l.z_order).collect();
    assert_eq!(z_orders, vec![-5, 0, 5, 10]);
}

#[test]
fn per_layer_dirty_tracking() {
    let mut comp = Compositor::new();
    let l1 = comp.create_layer(1);

    assert!(comp.layer(LayerId::DEFAULT).unwrap().is_dirty());
    assert!(comp.layer(l1).unwrap().is_dirty());

    comp.begin_frame();
    comp.push(SceneNode::Rect {
        x: 0.0,
        y: 0.0,
        w: 100.0,
        h: 100.0,
        color: [1.0, 0.0, 0.0, 1.0],
    });
    comp.push_to_layer(
        l1,
        SceneNode::Rect {
            x: 50.0,
            y: 50.0,
            w: 50.0,
            h: 50.0,
            color: [0.0, 1.0, 0.0, 1.0],
        },
    );

    for layer in &mut comp.layers {
        layer.resolve_dirty();
    }

    assert!(comp.layer(LayerId::DEFAULT).unwrap().is_dirty());
    assert!(comp.layer(l1).unwrap().is_dirty());

    comp.mark_layer_clean(LayerId::DEFAULT);
    comp.mark_layer_clean(l1);

    comp.begin_frame();
    comp.push(SceneNode::Rect {
        x: 0.0,
        y: 0.0,
        w: 100.0,
        h: 100.0,
        color: [1.0, 0.0, 0.0, 1.0],
    });
    comp.push_to_layer(
        l1,
        SceneNode::Rect {
            x: 50.0,
            y: 50.0,
            w: 50.0,
            h: 50.0,
            color: [0.0, 1.0, 0.0, 1.0],
        },
    );

    for layer in &mut comp.layers {
        layer.resolve_dirty();
    }

    assert!(!comp.layer(LayerId::DEFAULT).unwrap().is_dirty());
    assert!(!comp.layer(l1).unwrap().is_dirty());
}

#[test]
fn layer_opacity_and_visibility() {
    let mut comp = Compositor::new();
    let l1 = comp.create_layer(1);

    comp.set_layer_opacity(l1, 0.5);
    assert_eq!(comp.layer(l1).unwrap().opacity, 0.5);

    comp.set_layer_opacity(l1, 2.0);
    assert_eq!(comp.layer(l1).unwrap().opacity, 1.0);

    comp.set_layer_visible(l1, false);
    assert!(!comp.layer(l1).unwrap().visible);
}

#[test]
fn push_to_nonexistent_layer_warns() {
    let mut comp = Compositor::new();
    comp.push_to_layer(
        LayerId(999),
        SceneNode::Rect {
            x: 0.0,
            y: 0.0,
            w: 10.0,
            h: 10.0,
            color: [1.0, 1.0, 1.0, 1.0],
        },
    );
}

#[test]
fn set_layer_effects() {
    let mut comp = Compositor::new();
    let l1 = comp.create_layer(1);

    assert!(!comp.layer_has_effects(l1));
    assert!(comp.layer(l1).unwrap().effects().is_empty());

    comp.set_layer_effects(l1, vec![LayerEffect::Blur { sigma: 8.0 }]);
    assert!(comp.layer_has_effects(l1));
    assert_eq!(comp.layer(l1).unwrap().effects().len(), 1);
    assert!(matches!(
        comp.layer(l1).unwrap().effects()[0],
        LayerEffect::Blur { sigma } if sigma == 8.0
    ));

    comp.set_layer_effects(
        l1,
        vec![
            LayerEffect::Shadow {
                sigma: 4.0,
                color: [0.0, 0.0, 0.0, 0.5],
            },
            LayerEffect::Blur { sigma: 2.0 },
        ],
    );
    assert_eq!(comp.layer(l1).unwrap().effects().len(), 2);

    comp.set_layer_effects(l1, vec![]);
    assert!(!comp.layer_has_effects(l1));
}

#[test]
fn layer_without_effects_unchanged() {
    let comp = Compositor::new();
    assert!(!comp.layer_has_effects(LayerId::DEFAULT));
    assert!(!comp.layer(LayerId::DEFAULT).unwrap().has_effects());
}

#[test]
fn effects_on_nonexistent_layer() {
    let mut comp = Compositor::new();
    comp.set_layer_effects(LayerId(999), vec![LayerEffect::Blur { sigma: 5.0 }]);
    assert!(!comp.layer_has_effects(LayerId(999)));
}

#[test]
fn path_node_participates_in_dirty_tracking() {
    let mut comp = Compositor::new();

    let path_data = crate::path::PathBuilder::circle(50.0, 50.0, 25.0).fill([1.0, 0.0, 0.0, 1.0]);

    comp.begin_frame();
    comp.push(SceneNode::Path {
        data: path_data.clone(),
    });
    for layer in &mut comp.layers {
        layer.resolve_dirty();
    }
    assert!(comp.layer(LayerId::DEFAULT).unwrap().is_dirty());
    comp.mark_layer_clean(LayerId::DEFAULT);

    comp.begin_frame();
    comp.push(SceneNode::Path {
        data: path_data.clone(),
    });
    for layer in &mut comp.layers {
        layer.resolve_dirty();
    }
    assert!(!comp.layer(LayerId::DEFAULT).unwrap().is_dirty());

    comp.mark_layer_clean(LayerId::DEFAULT);
    comp.begin_frame();
    let different = crate::path::PathBuilder::circle(100.0, 100.0, 10.0).fill([0.0, 1.0, 0.0, 1.0]);
    comp.push(SceneNode::Path { data: different });
    for layer in &mut comp.layers {
        layer.resolve_dirty();
    }
    assert!(comp.layer(LayerId::DEFAULT).unwrap().is_dirty());
}
