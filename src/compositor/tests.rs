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

fn rect(x: f32, y: f32, w: f32, h: f32) -> SceneNode {
    SceneNode::Rect {
        x,
        y,
        w,
        h,
        color: [1.0, 0.0, 0.0, 1.0],
    }
}

fn rounded_rect(x: f32, y: f32, w: f32, h: f32) -> SceneNode {
    SceneNode::RoundedRect {
        x,
        y,
        w,
        h,
        color: [0.0, 1.0, 0.0, 1.0],
        corner_radius: 4.0,
        border_width: 0.0,
        border_color: [0.0, 0.0, 0.0, 0.0],
    }
}

// ---------------------------------------------------------------------------
// Viewport culling
// ---------------------------------------------------------------------------

#[test]
fn culling_skips_nodes_outside_viewport() {
    let mut comp = Compositor::new();
    comp.begin_frame();
    comp.push(rect(-200.0, 10.0, 100.0, 50.0)); // fully left
    comp.push(rect(900.0, 10.0, 100.0, 50.0)); // fully right
    comp.push(rect(10.0, 700.0, 100.0, 50.0)); // fully below
    comp.push(rect(10.0, 10.0, 100.0, 50.0)); // visible
    comp.resolve_scene((800.0, 600.0));

    let layer = comp.layer(LayerId::DEFAULT).unwrap();
    assert_eq!(layer.quad_vertices.len(), 4);
    assert_eq!(layer.quad_index_count, 6);
    assert_eq!(comp.stats().nodes_culled, 3);
}

#[test]
fn culling_keeps_partially_visible_nodes() {
    let mut comp = Compositor::new();
    comp.begin_frame();
    comp.push(rect(-50.0, -50.0, 100.0, 100.0)); // straddles top-left corner
    comp.push(rect(750.0, 550.0, 100.0, 100.0)); // straddles bottom-right
    comp.resolve_scene((800.0, 600.0));

    let layer = comp.layer(LayerId::DEFAULT).unwrap();
    assert_eq!(layer.quad_vertices.len(), 8);
    assert_eq!(comp.stats().nodes_culled, 0);
}

#[test]
fn culling_applies_to_rounded_rects() {
    let mut comp = Compositor::new();
    comp.begin_frame();
    comp.push(rounded_rect(-300.0, 0.0, 100.0, 100.0)); // fully out
    comp.push(rounded_rect(10.0, 10.0, 100.0, 100.0)); // visible
    comp.resolve_scene((800.0, 600.0));

    let layer = comp.layer(LayerId::DEFAULT).unwrap();
    assert_eq!(layer.sdf_vertices.len(), 4);
    assert_eq!(comp.stats().nodes_culled, 1);
}

#[test]
fn culling_applies_to_paths_via_bounds() {
    let mut comp = Compositor::new();
    let offscreen =
        crate::path::PathBuilder::circle(-100.0, -100.0, 10.0).fill([1.0, 0.0, 0.0, 1.0]);
    let onscreen = crate::path::PathBuilder::circle(50.0, 50.0, 10.0).fill([1.0, 0.0, 0.0, 1.0]);
    let visible_vertex_count = onscreen.vertices.len();

    comp.begin_frame();
    comp.push(SceneNode::Path { data: offscreen });
    comp.push(SceneNode::Path { data: onscreen });
    comp.resolve_scene((800.0, 600.0));

    let layer = comp.layer(LayerId::DEFAULT).unwrap();
    assert_eq!(layer.quad_vertices.len(), visible_vertex_count);
    assert_eq!(comp.stats().nodes_culled, 1);
}

// ---------------------------------------------------------------------------
// Render on demand (needs_render / invalidate)
// ---------------------------------------------------------------------------

#[test]
fn needs_render_false_for_unchanged_scene() {
    let mut comp = Compositor::new();
    assert!(comp.needs_render()); // fresh compositor has never rendered

    comp.begin_frame();
    comp.push(rect(0.0, 0.0, 10.0, 10.0));
    comp.resolve_scene((800.0, 600.0));
    comp.mark_layer_clean(LayerId::DEFAULT);

    // Same scene rebuilt -> no render needed
    comp.begin_frame();
    comp.push(rect(0.0, 0.0, 10.0, 10.0));
    assert!(!comp.needs_render());

    // A different node -> render needed
    comp.begin_frame();
    comp.push(rect(5.0, 0.0, 10.0, 10.0));
    assert!(comp.needs_render());
}

#[test]
fn needs_render_after_invalidate() {
    let mut comp = Compositor::new();
    comp.begin_frame();
    comp.push(rect(0.0, 0.0, 10.0, 10.0));
    comp.resolve_scene((800.0, 600.0));
    comp.mark_layer_clean(LayerId::DEFAULT);

    comp.begin_frame();
    comp.push(rect(0.0, 0.0, 10.0, 10.0));
    assert!(!comp.needs_render());

    comp.invalidate();
    assert!(comp.needs_render());

    // Resolving consumes the external invalidation
    comp.resolve_scene((800.0, 600.0));
    comp.mark_layer_clean(LayerId::DEFAULT);
    assert!(!comp.needs_render());
}

// ---------------------------------------------------------------------------
// Layer sort (only when z_order changes)
// ---------------------------------------------------------------------------

#[test]
fn layers_sort_only_when_z_order_changes() {
    let mut comp = Compositor::new();
    let l_top = comp.create_layer(10);
    let l_bottom = comp.create_layer(-5);
    assert!(!comp.sorted);

    comp.resolve_scene((100.0, 100.0));
    assert!(comp.sorted);
    let order: Vec<LayerId> = comp.layers().iter().map(|l| l.id).collect();
    assert_eq!(order, vec![l_bottom, LayerId::DEFAULT, l_top]);

    // No z-order change -> stays sorted and the order is stable
    comp.resolve_scene((100.0, 100.0));
    assert!(comp.sorted);
    let order_after: Vec<LayerId> = comp.layers().iter().map(|l| l.id).collect();
    assert_eq!(order_after, order);

    // Same z value -> no re-sort scheduled
    comp.set_layer_z_order(l_top, 10);
    assert!(comp.sorted);

    // New z value -> re-sort on next resolve
    comp.set_layer_z_order(l_top, -10);
    assert!(!comp.sorted);
    comp.resolve_scene((100.0, 100.0));
    assert!(comp.sorted);
    assert_eq!(comp.layers()[0].id, l_top);
}

// ---------------------------------------------------------------------------
// RenderStats
// ---------------------------------------------------------------------------

#[test]
fn render_stats_for_known_scene() {
    let mut comp = Compositor::new();
    comp.begin_frame();
    comp.push(rect(0.0, 0.0, 100.0, 50.0));
    comp.push(rect(10.0, 10.0, 100.0, 50.0));
    comp.push(rounded_rect(20.0, 20.0, 40.0, 40.0));
    comp.push(rect(-500.0, 0.0, 100.0, 50.0)); // culled
    comp.resolve_scene((800.0, 600.0));

    let s = comp.stats();
    assert_eq!(s.layers_total, 1);
    assert_eq!(s.layers_redrawn, 1);
    assert_eq!(s.quad_vertices, 8); // 2 visible rects * 4 vertices
    assert_eq!(s.sdf_vertices, 4); // 1 rounded rect * 4 vertices
    assert_eq!(s.nodes_culled, 1);

    comp.record_encode_stats(5, 12, 1234);
    let s = comp.stats();
    assert_eq!(s.draw_calls, 5);
    assert_eq!(s.glyphs, 12);
    assert_eq!(s.encode_micros, 1234);

    // Unchanged scene next frame -> nothing redrawn, totals preserved
    comp.mark_layer_clean(LayerId::DEFAULT);
    comp.begin_frame();
    comp.push(rect(0.0, 0.0, 100.0, 50.0));
    comp.push(rect(10.0, 10.0, 100.0, 50.0));
    comp.push(rounded_rect(20.0, 20.0, 40.0, 40.0));
    comp.push(rect(-500.0, 0.0, 100.0, 50.0));
    comp.resolve_scene((800.0, 600.0));

    let s = comp.stats();
    assert_eq!(s.layers_redrawn, 0);
    assert_eq!(s.nodes_culled, 0); // nothing rebuilt, nothing culled this frame
    assert_eq!(s.quad_vertices, 8);
    assert_eq!(s.sdf_vertices, 4);
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
