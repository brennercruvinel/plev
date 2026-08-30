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
// Image sprites
// ---------------------------------------------------------------------------

#[test]
fn image_node_emits_pixel_uvs_and_ranges() {
    let handle = crate::gpu::image::ImageHandle {
        id: 7,
        atlas_x: 100,
        atlas_y: 200,
        width: 32,
        height: 16,
    };
    let mut comp = Compositor::new();
    comp.begin_frame();
    comp.draw_image(10.0, 20.0, 64.0, 32.0, handle, 4.0);
    comp.resolve_scene((800.0, 600.0));

    let layer = comp.layer(LayerId::DEFAULT).unwrap();
    assert_eq!(layer.image_vertices.len(), 4);
    assert_eq!(layer.image_index_count, 6);
    assert_eq!(comp.stats().image_vertices, 4);

    let v = &layer.image_vertices;
    // Quad covers the target bounds, atlas coords are in pixels
    assert_eq!(v[0].position, [10.0, 20.0]);
    assert_eq!(v[0].atlas_px, [100.0, 200.0]);
    assert_eq!(v[2].position, [74.0, 52.0]);
    assert_eq!(v[2].atlas_px, [132.0, 216.0]);
    // Sampling clamp is inset half a texel from the image rect
    assert_eq!(v[0].uv_bounds, [100.5, 200.5, 131.5, 215.5]);
    // Rounded-corner SDF params: half size + radius
    assert_eq!(v[0].params, [32.0, 16.0, 4.0, 0.0]);

    assert_eq!(layer.image_draw_ranges().len(), 1);
    assert_eq!(
        layer.image_draw_ranges()[0],
        DrawRange {
            first_index: 0,
            index_count: 6,
            clip: None
        }
    );
}

#[test]
fn image_nodes_respect_culling_and_clip() {
    let handle = crate::gpu::image::ImageHandle {
        id: 1,
        atlas_x: 0,
        atlas_y: 0,
        width: 8,
        height: 8,
    };
    let mut comp = Compositor::new();
    comp.begin_frame();
    comp.draw_image(-500.0, 0.0, 50.0, 50.0, handle, 0.0); // culled
    comp.push_clip(0.0, 0.0, 100.0, 100.0);
    comp.draw_image(10.0, 10.0, 50.0, 50.0, handle, 0.0); // clipped
    comp.pop_clip();
    comp.resolve_scene((800.0, 600.0));

    let layer = comp.layer(LayerId::DEFAULT).unwrap();
    assert_eq!(layer.image_vertices.len(), 4);
    assert_eq!(comp.stats().nodes_culled, 1);
    assert_eq!(layer.image_draw_ranges().len(), 1);
    assert_eq!(
        layer.image_draw_ranges()[0].clip,
        Some([0.0, 0.0, 100.0, 100.0])
    );
}

// ---------------------------------------------------------------------------
// Clip stack
// ---------------------------------------------------------------------------

#[test]
fn nested_clips_intersect() {
    assert_eq!(
        intersect_rects([0.0, 0.0, 100.0, 100.0], [50.0, 25.0, 100.0, 100.0]),
        [50.0, 25.0, 50.0, 75.0]
    );
    // Disjoint rects produce a degenerate (empty) intersection
    let empty = intersect_rects([0.0, 0.0, 10.0, 10.0], [20.0, 0.0, 10.0, 10.0]);
    assert!(empty[2] <= 0.0);
}

#[test]
fn clip_to_scissor_clamps_to_viewport() {
    assert_eq!(
        clip_to_scissor([-10.0, -10.0, 100.0, 100.0], 800, 600),
        Some((0, 0, 90, 90))
    );
    assert_eq!(
        clip_to_scissor([750.0, 550.0, 100.0, 100.0], 800, 600),
        Some((750, 550, 50, 50))
    );
    // Entirely outside -> None
    assert_eq!(clip_to_scissor([900.0, 0.0, 50.0, 50.0], 800, 600), None);
    assert_eq!(clip_to_scissor([0.0, 0.0, 0.0, 50.0], 800, 600), None);
}

#[test]
fn nodes_are_grouped_into_draw_ranges_by_clip() {
    let mut comp = Compositor::new();
    comp.begin_frame();
    comp.push(rect(0.0, 0.0, 10.0, 10.0)); // unclipped
    comp.push(rect(10.0, 0.0, 10.0, 10.0)); // unclipped (merges with above)
    comp.push_clip(0.0, 0.0, 50.0, 50.0);
    comp.push(rect(20.0, 0.0, 10.0, 10.0)); // clipped
    comp.push_clip(10.0, 10.0, 50.0, 50.0); // nested: intersection
    comp.push(rect(30.0, 0.0, 10.0, 10.0)); // clipped (nested)
    comp.pop_clip();
    comp.pop_clip();
    comp.push(rect(40.0, 0.0, 10.0, 10.0)); // unclipped again
    comp.resolve_scene((800.0, 600.0));

    let layer = comp.layer(LayerId::DEFAULT).unwrap();
    let ranges = layer.quad_draw_ranges();
    assert_eq!(ranges.len(), 4);

    assert_eq!(
        ranges[0],
        DrawRange {
            first_index: 0,
            index_count: 12,
            clip: None
        }
    );
    assert_eq!(ranges[1].clip, Some([0.0, 0.0, 50.0, 50.0]));
    assert_eq!(ranges[1].first_index, 12);
    assert_eq!(ranges[1].index_count, 6);
    // Nested clip is the intersection of both rects
    assert_eq!(ranges[2].clip, Some([10.0, 10.0, 40.0, 40.0]));
    assert_eq!(ranges[3].clip, None);
    assert_eq!(ranges[3].first_index, 24);

    // Ranges tile the index buffer exactly
    let total: u32 = ranges.iter().map(|r| r.index_count).sum();
    assert_eq!(total, layer.quad_index_count);
}

#[test]
fn unbalanced_pop_clip_is_ignored() {
    let mut comp = Compositor::new();
    comp.begin_frame();
    comp.pop_clip(); // stray pop: must not panic nor clip anything
    comp.push(rect(0.0, 0.0, 10.0, 10.0));
    comp.resolve_scene((800.0, 600.0));

    let layer = comp.layer(LayerId::DEFAULT).unwrap();
    assert_eq!(layer.quad_draw_ranges().len(), 1);
    assert_eq!(layer.quad_draw_ranges()[0].clip, None);
}

#[test]
fn empty_clip_intersection_culls_nodes() {
    let mut comp = Compositor::new();
    comp.begin_frame();
    comp.push_clip(0.0, 0.0, 50.0, 50.0);
    comp.push_clip(100.0, 100.0, 50.0, 50.0); // disjoint from the first
    comp.push(rect(0.0, 0.0, 200.0, 200.0));
    comp.push(rounded_rect(0.0, 0.0, 200.0, 200.0));
    comp.pop_clip();
    comp.pop_clip();
    comp.resolve_scene((800.0, 600.0));

    let layer = comp.layer(LayerId::DEFAULT).unwrap();
    assert_eq!(layer.quad_index_count, 0);
    assert_eq!(layer.sdf_index_count, 0);
    assert_eq!(comp.stats().nodes_culled, 2);
}

#[test]
fn clip_applies_to_sdf_and_shadow_ranges() {
    let mut comp = Compositor::new();
    comp.begin_frame();
    comp.push_clip(0.0, 0.0, 100.0, 100.0);
    comp.push(rounded_rect(10.0, 10.0, 50.0, 50.0));
    comp.push(SceneNode::Shadow {
        x: 10.0,
        y: 10.0,
        w: 50.0,
        h: 50.0,
        corner_radius: 4.0,
        blur_radius: 8.0,
        offset: [0.0, 2.0],
        color: [0.0, 0.0, 0.0, 0.5],
        inset: false,
    });
    comp.pop_clip();
    comp.resolve_scene((800.0, 600.0));

    let layer = comp.layer(LayerId::DEFAULT).unwrap();
    assert_eq!(layer.sdf_draw_ranges().len(), 1);
    assert_eq!(
        layer.sdf_draw_ranges()[0].clip,
        Some([0.0, 0.0, 100.0, 100.0])
    );
    assert_eq!(layer.shadow_draw_ranges().len(), 1);
    assert_eq!(
        layer.shadow_draw_ranges()[0].clip,
        Some([0.0, 0.0, 100.0, 100.0])
    );
}

fn text_node(label: &str) -> SceneNode {
    SceneNode::Text {
        key: TextNodeKey::new(label, 14.0, 18.0, None),
        x: 0.0,
        y: 0.0,
        color: [1.0; 4],
    }
}

#[test]
fn text_node_groups_split_by_clip() {
    let mut comp = Compositor::new();
    comp.begin_frame();
    comp.push(text_node("a"));
    comp.push(text_node("b"));
    comp.push_clip(0.0, 0.0, 50.0, 50.0);
    comp.push(text_node("c"));
    comp.pop_clip();
    comp.push(text_node("d"));
    comp.resolve_scene((800.0, 600.0));

    let layer = comp.layer(LayerId::DEFAULT).unwrap();
    let groups = layer.text_node_groups();
    assert_eq!(groups.len(), 3);
    assert_eq!(groups[0].0.len(), 2);
    assert_eq!(groups[0].1, None);
    assert_eq!(groups[1].0.len(), 1);
    assert_eq!(groups[1].1, Some([0.0, 0.0, 50.0, 50.0]));
    assert_eq!(groups[2].1, None);
}

#[test]
fn merge_text_groups_rebases_indices_and_builds_ranges() {
    fn vertex(x: f32) -> crate::text::TextVertex {
        crate::text::TextVertex {
            position: [x, 0.0],
            uv: [0.0, 0.0],
            color: [1.0; 4],
        }
    }

    let groups = vec![
        (
            vec![vertex(0.0), vertex(1.0), vertex(2.0), vertex(3.0)],
            vec![0, 1, 2, 2, 3, 0],
            None,
        ),
        (
            vec![vertex(4.0), vertex(5.0), vertex(6.0), vertex(7.0)],
            vec![0, 1, 2, 2, 3, 0],
            Some([0.0, 0.0, 50.0, 50.0]),
        ),
    ];
    let (vertices, indices, ranges) = merge_text_groups(groups);

    assert_eq!(vertices.len(), 8);
    assert_eq!(indices.len(), 12);
    // Second group's indices rebased past the first group's vertices
    assert_eq!(&indices[6..], &[4, 5, 6, 6, 7, 4]);
    assert_eq!(ranges.len(), 2);
    assert_eq!(
        ranges[0],
        DrawRange {
            first_index: 0,
            index_count: 6,
            clip: None
        }
    );
    assert_eq!(
        ranges[1],
        DrawRange {
            first_index: 6,
            index_count: 6,
            clip: Some([0.0, 0.0, 50.0, 50.0])
        }
    );
}

#[test]
fn intersect_scissors_clamps_and_rejects_empty() {
    assert_eq!(
        intersect_scissors((0, 0, 100, 100), (50, 50, 100, 100)),
        Some((50, 50, 50, 50))
    );
    assert_eq!(intersect_scissors((0, 0, 10, 10), (20, 20, 10, 10)), None);
}

// ---------------------------------------------------------------------------
// Analytic shadow
// ---------------------------------------------------------------------------

#[test]
fn shadow_quad_is_expanded_by_blur_and_shifted_by_offset() {
    let mut comp = Compositor::new();
    comp.begin_frame();
    comp.push(SceneNode::Shadow {
        x: 100.0,
        y: 200.0,
        w: 80.0,
        h: 40.0,
        corner_radius: 8.0,
        blur_radius: 16.0,
        offset: [0.0, 4.0],
        color: [0.0, 0.0, 0.0, 0.5],
        inset: false,
    });
    comp.resolve_scene((800.0, 600.0));

    let layer = comp.layer(LayerId::DEFAULT).unwrap();
    assert_eq!(layer.shadow_vertices.len(), 4);
    assert_eq!(layer.shadow_index_count, 6);

    let pad = shadow_padding(16.0); // 3 * sigma = 1.5 * blur
    assert_eq!(pad, 24.0);

    let v = &layer.shadow_vertices;
    // Top-left corner: rect origin - padding + offset
    assert_eq!(v[0].position, [100.0 - pad, 200.0 - pad + 4.0]);
    // Bottom-right corner: rect end + padding + offset
    assert_eq!(v[2].position, [180.0 + pad, 240.0 + pad + 4.0]);
    // Local coords span the padded half extents, centered on the rect
    assert_eq!(v[0].local, [-(40.0 + pad), -(20.0 + pad)]);
    assert_eq!(v[2].local, [40.0 + pad, 20.0 + pad]);
    for vert in v {
        // half_w, half_h, corner_radius, sigma
        assert_eq!(vert.params, [40.0, 20.0, 8.0, shadow_sigma(16.0)]);
        assert_eq!(vert.color, [0.0, 0.0, 0.0, 0.5]);
    }
}

#[test]
fn shadow_is_culled_using_expanded_bounds() {
    let mut comp = Compositor::new();
    comp.begin_frame();
    // Rect outside the viewport, but the expanded shadow quad reaches in:
    // x+w = -10, padding = 30 -> quad right edge at +20.
    comp.push(SceneNode::Shadow {
        x: -110.0,
        y: 10.0,
        w: 100.0,
        h: 50.0,
        corner_radius: 0.0,
        blur_radius: 20.0,
        offset: [0.0, 0.0],
        color: [0.0, 0.0, 0.0, 1.0],
        inset: false,
    });
    // Far enough away that not even the expanded quad is visible.
    comp.push(SceneNode::Shadow {
        x: -500.0,
        y: 10.0,
        w: 100.0,
        h: 50.0,
        corner_radius: 0.0,
        blur_radius: 20.0,
        offset: [0.0, 0.0],
        color: [0.0, 0.0, 0.0, 1.0],
        inset: false,
    });
    comp.resolve_scene((800.0, 600.0));

    let layer = comp.layer(LayerId::DEFAULT).unwrap();
    assert_eq!(layer.shadow_vertices.len(), 4);
    assert_eq!(comp.stats().nodes_culled, 1);
    assert_eq!(comp.stats().shadow_vertices, 4);
}

#[test]
fn shadow_node_participates_in_dirty_tracking() {
    fn shadow(blur_radius: f32) -> SceneNode {
        SceneNode::Shadow {
            x: 0.0,
            y: 0.0,
            w: 10.0,
            h: 10.0,
            corner_radius: 2.0,
            blur_radius,
            offset: [0.0, 2.0],
            color: [0.0, 0.0, 0.0, 0.4],
            inset: false,
        }
    }

    let mut comp = Compositor::new();
    comp.begin_frame();
    comp.push(shadow(8.0));
    comp.resolve_scene((800.0, 600.0));
    comp.mark_layer_clean(LayerId::DEFAULT);

    // Same shadow rebuilt -> no render needed
    comp.begin_frame();
    comp.push(shadow(8.0));
    assert!(!comp.needs_render());

    // Different blur -> different hash -> render needed
    comp.begin_frame();
    comp.push(shadow(9.0));
    assert!(comp.needs_render());
}

#[test]
fn inset_shadow_quad_is_the_rect_with_inset_flag_and_offset() {
    let mut comp = Compositor::new();
    comp.begin_frame();
    // HOFF glass relief: inset 2px 4px 16px rgba(248,248,248,.06).
    comp.push(SceneNode::Shadow {
        x: 100.0,
        y: 200.0,
        w: 80.0,
        h: 40.0,
        corner_radius: 8.0,
        blur_radius: 16.0,
        offset: [2.0, 4.0],
        color: [248.0 / 255.0, 248.0 / 255.0, 248.0 / 255.0, 0.06],
        inset: true,
    });
    comp.resolve_scene((800.0, 600.0));

    let layer = comp.layer(LayerId::DEFAULT).unwrap();
    assert_eq!(layer.shadow_vertices.len(), 4);
    assert_eq!(layer.shadow_index_count, 6);

    let v = &layer.shadow_vertices;
    // The quad is EXACTLY the casting rect: no blur padding, no offset
    // shift (the shadow is clipped inside the rect).
    assert_eq!(v[0].position, [100.0, 200.0]);
    assert_eq!(v[2].position, [180.0, 240.0]);
    // Local coords span the plain half extents.
    assert_eq!(v[0].local, [-40.0, -20.0]);
    assert_eq!(v[2].local, [40.0, 20.0]);
    for vert in v {
        assert_eq!(vert.params, [40.0, 20.0, 8.0, shadow_sigma(16.0)]);
        // inset flag + offset move only the in-shader mask.
        assert_eq!(vert.params2, [1.0, 2.0, 4.0, 0.0]);
    }
}

#[test]
fn drop_shadow_keeps_params2_zeroed() {
    let mut comp = Compositor::new();
    comp.begin_frame();
    comp.push(SceneNode::Shadow {
        x: 0.0,
        y: 0.0,
        w: 10.0,
        h: 10.0,
        corner_radius: 2.0,
        blur_radius: 8.0,
        offset: [0.0, 2.0],
        color: [0.0, 0.0, 0.0, 0.4],
        inset: false,
    });
    comp.resolve_scene((800.0, 600.0));

    let layer = comp.layer(LayerId::DEFAULT).unwrap();
    for vert in &layer.shadow_vertices {
        assert_eq!(vert.params2, [0.0; 4]);
    }
}

#[test]
fn inset_flag_participates_in_dirty_tracking() {
    fn shadow(inset: bool) -> SceneNode {
        SceneNode::Shadow {
            x: 0.0,
            y: 0.0,
            w: 10.0,
            h: 10.0,
            corner_radius: 2.0,
            blur_radius: 8.0,
            offset: [0.0, 2.0],
            color: [0.0, 0.0, 0.0, 0.4],
            inset,
        }
    }

    let mut comp = Compositor::new();
    comp.begin_frame();
    comp.push(shadow(false));
    comp.resolve_scene((800.0, 600.0));
    comp.mark_layer_clean(LayerId::DEFAULT);

    comp.begin_frame();
    comp.push(shadow(true));
    assert!(comp.needs_render());
}

#[test]
fn inset_shadow_after_fill_stays_on_top_in_sequence() {
    let mut comp = Compositor::new();
    comp.begin_frame();
    // Glass card: fill first, then the inset relief on top of it.
    comp.push(rounded_rect(10.0, 10.0, 100.0, 50.0));
    comp.push(SceneNode::Shadow {
        x: 10.0,
        y: 10.0,
        w: 100.0,
        h: 50.0,
        corner_radius: 8.0,
        blur_radius: 16.0,
        offset: [2.0, 4.0],
        color: [1.0, 1.0, 1.0, 0.06],
        inset: true,
    });
    comp.resolve_scene((800.0, 600.0));

    assert_eq!(
        sequence_kinds(&comp),
        vec![DrawKind::SdfRect, DrawKind::Shadow]
    );
}

// ---------------------------------------------------------------------------
// Gradient brush
// ---------------------------------------------------------------------------

#[test]
fn gradient_rect_vertices_carry_both_colors_and_direction() {
    let mut comp = Compositor::new();
    comp.begin_frame();
    comp.push(SceneNode::GradientRect {
        x: 10.0,
        y: 20.0,
        w: 100.0,
        h: 50.0,
        color: [1.0, 0.0, 0.0, 1.0],
        color2: [0.0, 0.0, 1.0, 1.0],
        angle_deg: 90.0,
        corner_radius: 6.0,
        border_width: 0.0,
        border_color: [0.0; 4],
    });
    comp.resolve_scene((800.0, 600.0));

    let layer = comp.layer(LayerId::DEFAULT).unwrap();
    assert_eq!(layer.sdf_vertices.len(), 4);
    for v in &layer.sdf_vertices {
        assert_eq!(v.color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(v.color2, [0.0, 0.0, 1.0, 1.0]);
        assert!(v.gradient[2] > 0.0, "gradient must be enabled");
        // 90 degrees points right in screen space
        assert!((v.gradient[0] - 1.0).abs() < 1e-5);
        assert!(v.gradient[1].abs() < 1e-5);
    }
}

#[test]
fn solid_rounded_rect_has_gradient_disabled() {
    let mut comp = Compositor::new();
    comp.begin_frame();
    comp.push(rounded_rect(0.0, 0.0, 50.0, 50.0));
    comp.resolve_scene((800.0, 600.0));

    let layer = comp.layer(LayerId::DEFAULT).unwrap();
    for v in &layer.sdf_vertices {
        assert_eq!(v.gradient, [0.0; 4]);
        assert_eq!(v.color2, v.color);
    }
}

#[test]
fn gradient_direction_follows_css_convention() {
    let up = gradient_direction(0.0);
    assert!(up[0].abs() < 1e-6 && (up[1] + 1.0).abs() < 1e-6);
    let right = gradient_direction(90.0);
    assert!((right[0] - 1.0).abs() < 1e-6 && right[1].abs() < 1e-6);
    let down = gradient_direction(180.0);
    assert!(down[0].abs() < 1e-6 && (down[1] - 1.0).abs() < 1e-6);
}

#[test]
fn gradient_rect_participates_in_culling_and_dirty_tracking() {
    let mut comp = Compositor::new();
    comp.begin_frame();
    comp.push(SceneNode::GradientRect {
        x: -500.0,
        y: 0.0,
        w: 100.0,
        h: 50.0,
        color: [1.0; 4],
        color2: [0.0, 0.0, 0.0, 1.0],
        angle_deg: 0.0,
        corner_radius: 0.0,
        border_width: 0.0,
        border_color: [0.0; 4],
    });
    comp.resolve_scene((800.0, 600.0));
    assert_eq!(comp.stats().nodes_culled, 1);
    assert_eq!(comp.stats().sdf_vertices, 0);
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

// ---------------------------------------------------------------------------
// Draw sequence (push order across primitive types)
// ---------------------------------------------------------------------------

/// Kinds of the geometry commands in a layer's sequence, in order
/// (backdrop commands excluded; see `sequence_of` for the full walk).
fn sequence_kinds(comp: &Compositor) -> Vec<DrawKind> {
    comp.layer(LayerId::DEFAULT)
        .unwrap()
        .sequence()
        .iter()
        .filter_map(|cmd| match cmd {
            DrawCommand::Geometry { kind, .. } => Some(*kind),
            DrawCommand::BackdropBlur { .. } => None,
        })
        .collect()
}

#[test]
fn sequence_preserves_push_order_across_kinds() {
    let mut comp = Compositor::new();
    comp.begin_frame();
    // Card composition: quad bg, shadow, SDF pill, path icon on top of the
    // pill, then a rect over everything.
    comp.push(rect(0.0, 0.0, 100.0, 100.0));
    comp.push(SceneNode::Shadow {
        x: 10.0,
        y: 10.0,
        w: 50.0,
        h: 30.0,
        corner_radius: 8.0,
        blur_radius: 8.0,
        offset: [0.0, 2.0],
        color: [0.0, 0.0, 0.0, 0.4],
        inset: false,
    });
    comp.push(rounded_rect(10.0, 10.0, 50.0, 30.0));
    comp.push(SceneNode::Path {
        data: crate::path::PathBuilder::circle(30.0, 25.0, 6.0).fill([1.0; 4]),
    });
    comp.push(rect(0.0, 0.0, 20.0, 20.0));
    comp.resolve_scene((800.0, 600.0));

    // The path icon and the rect after it are both quad-pipeline geometry
    // pushed back to back, so they merge into ONE trailing quad command --
    // what matters is that it draws AFTER the SDF pill.
    assert_eq!(
        sequence_kinds(&comp),
        vec![
            DrawKind::Quad,
            DrawKind::Shadow,
            DrawKind::SdfRect,
            DrawKind::Quad,
        ]
    );

    // The trailing quad command picks up right after the first one in the
    // shared buffer: order within the buffer still follows push order.
    let layer = comp.layer(LayerId::DEFAULT).unwrap();
    let quad_cmds: Vec<_> = layer
        .sequence()
        .iter()
        .filter_map(|cmd| match cmd {
            DrawCommand::Geometry {
                kind: DrawKind::Quad,
                range,
            } => Some(*range),
            _ => None,
        })
        .collect();
    assert_eq!(quad_cmds[0].first_index, 0);
    assert_eq!(quad_cmds[1].first_index, 6); // path picks up after rect 1
    let total: u32 = quad_cmds.iter().map(|r| r.index_count).sum();
    assert_eq!(total, layer.quad_index_count);
}

#[test]
fn sequence_merges_adjacent_same_kind_same_clip() {
    let mut comp = Compositor::new();
    comp.begin_frame();
    comp.push(rect(0.0, 0.0, 10.0, 10.0));
    comp.push(rect(10.0, 0.0, 10.0, 10.0)); // merges with the previous
    comp.push_clip(0.0, 0.0, 50.0, 50.0);
    comp.push(rect(20.0, 0.0, 10.0, 10.0)); // new command: clip changed
    comp.pop_clip();
    comp.resolve_scene((800.0, 600.0));

    let layer = comp.layer(LayerId::DEFAULT).unwrap();
    assert_eq!(layer.sequence().len(), 2);
    assert!(matches!(
        layer.sequence()[0],
        DrawCommand::Geometry {
            kind: DrawKind::Quad,
            range: DrawRange {
                index_count: 12,
                clip: None,
                ..
            },
        }
    ));
    assert!(matches!(
        layer.sequence()[1],
        DrawCommand::Geometry {
            range: DrawRange {
                clip: Some([0.0, 0.0, 50.0, 50.0]),
                ..
            },
            ..
        }
    ));
}

#[test]
fn sequence_does_not_merge_across_interleaved_kind() {
    let mut comp = Compositor::new();
    comp.begin_frame();
    comp.push(rect(0.0, 0.0, 10.0, 10.0));
    comp.push(rounded_rect(0.0, 0.0, 10.0, 10.0));
    comp.push(rect(20.0, 0.0, 10.0, 10.0));
    comp.resolve_scene((800.0, 600.0));

    assert_eq!(
        sequence_kinds(&comp),
        vec![DrawKind::Quad, DrawKind::SdfRect, DrawKind::Quad]
    );
}

#[test]
fn culled_nodes_leave_no_sequence_gap() {
    let mut comp = Compositor::new();
    comp.begin_frame();
    comp.push(rect(0.0, 0.0, 10.0, 10.0));
    comp.push(rounded_rect(-500.0, 0.0, 10.0, 10.0)); // culled
    comp.push(rect(20.0, 0.0, 10.0, 10.0)); // merges with the first rect
    comp.resolve_scene((800.0, 600.0));

    assert_eq!(sequence_kinds(&comp), vec![DrawKind::Quad]);
}

#[test]
fn text_enters_sequence_in_push_order() {
    let mut comp = Compositor::new();
    comp.begin_frame();
    comp.push(text_node("behind"));
    comp.push(rect(0.0, 0.0, 10.0, 10.0)); // rect pushed after: covers glyph
    comp.push(text_node("front"));
    comp.resolve_scene((800.0, 600.0));

    assert_eq!(
        sequence_kinds(&comp),
        vec![DrawKind::Text, DrawKind::Quad, DrawKind::Text]
    );
    // One text group per Text command, in order.
    let layer = comp.layer(LayerId::DEFAULT).unwrap();
    let groups = layer.text_node_groups();
    assert_eq!(groups.len(), 2);
    assert!(matches!(&groups[0].0[0], SceneNode::Text { key, .. } if key.text == "behind"));
    assert!(matches!(&groups[1].0[0], SceneNode::Text { key, .. } if key.text == "front"));
}

#[test]
fn assign_text_ranges_patches_text_commands_in_order() {
    let mut comp = Compositor::new();
    comp.begin_frame();
    comp.push(text_node("a"));
    comp.push(rect(0.0, 0.0, 10.0, 10.0));
    comp.push(text_node("b"));
    comp.resolve_scene((800.0, 600.0));

    // Placeholder ranges before patch.
    {
        let layer = comp.layer(LayerId::DEFAULT).unwrap();
        for cmd in layer.sequence() {
            if let DrawCommand::Geometry {
                kind: DrawKind::Text,
                range,
            } = cmd
            {
                assert_eq!(range.index_count, 0);
            }
        }
    }

    let layer = comp.layer_mut(LayerId::DEFAULT).unwrap();
    layer.assign_text_ranges(&[
        DrawRange {
            first_index: 0,
            index_count: 6,
            clip: None,
        },
        DrawRange {
            first_index: 6,
            index_count: 12,
            clip: None,
        },
    ]);

    let layer = comp.layer(LayerId::DEFAULT).unwrap();
    let text_ranges: Vec<_> = layer
        .sequence()
        .iter()
        .filter_map(|cmd| match cmd {
            DrawCommand::Geometry {
                kind: DrawKind::Text,
                range,
            } => Some(*range),
            _ => None,
        })
        .collect();
    assert_eq!(text_ranges.len(), 2);
    assert_eq!(text_ranges[0].first_index, 0);
    assert_eq!(text_ranges[0].index_count, 6);
    assert_eq!(text_ranges[1].first_index, 6);
    assert_eq!(text_ranges[1].index_count, 12);
}

// ---------------------------------------------------------------------------
// Backdrop blur
// ---------------------------------------------------------------------------

#[test]
fn backdrop_records_resolve_at_its_point_in_the_sequence() {
    let mut comp = Compositor::new();
    comp.begin_frame();
    // Glass card: page bg, frosted region, translucent fill on top.
    comp.push(rect(0.0, 0.0, 800.0, 600.0));
    comp.draw_backdrop_blur(100.0, 100.0, 300.0, 200.0, 24.0, 16.0);
    comp.push(rounded_rect(100.0, 100.0, 300.0, 200.0));
    comp.resolve_scene((800.0, 600.0));

    let layer = comp.layer(LayerId::DEFAULT).unwrap();
    let seq = layer.sequence();
    assert_eq!(seq.len(), 3);
    assert!(matches!(
        seq[0],
        DrawCommand::Geometry {
            kind: DrawKind::Quad,
            ..
        }
    ));
    // The resolve happens BETWEEN the bg and the fill -- everything before
    // is part of the frosted backdrop, everything after draws on top.
    assert!(matches!(
        seq[1],
        DrawCommand::BackdropBlur {
            first_index: 0,
            sigma,
            clip: None,
        } if sigma == 16.0
    ));
    assert!(matches!(
        seq[2],
        DrawCommand::Geometry {
            kind: DrawKind::SdfRect,
            ..
        }
    ));

    // Quad geometry: the rect itself, with the rounded-corner mask params.
    assert_eq!(layer.backdrop_vertices.len(), 4);
    assert_eq!(layer.backdrop_index_count, 6);
    let v = &layer.backdrop_vertices;
    assert_eq!(v[0].position, [100.0, 100.0]);
    assert_eq!(v[2].position, [400.0, 300.0]);
    assert_eq!(v[0].local, [-150.0, -100.0]);
    for vert in v {
        assert_eq!(vert.params, [150.0, 100.0, 24.0, 0.0]);
    }
}

#[test]
fn backdrop_respects_clip_and_culling() {
    let mut comp = Compositor::new();
    comp.begin_frame();
    comp.draw_backdrop_blur(-500.0, 0.0, 100.0, 100.0, 8.0, 4.0); // culled
    comp.push_clip(0.0, 0.0, 200.0, 200.0);
    comp.draw_backdrop_blur(10.0, 10.0, 100.0, 100.0, 8.0, 4.0); // clipped
    comp.pop_clip();
    comp.resolve_scene((800.0, 600.0));

    let layer = comp.layer(LayerId::DEFAULT).unwrap();
    assert_eq!(comp.stats().nodes_culled, 1);
    assert_eq!(layer.backdrop_vertices.len(), 4);
    assert_eq!(layer.sequence().len(), 1);
    assert!(matches!(
        layer.sequence()[0],
        DrawCommand::BackdropBlur {
            clip: Some([0.0, 0.0, 200.0, 200.0]),
            ..
        }
    ));
}

#[test]
fn backdrop_node_participates_in_dirty_tracking() {
    let mut comp = Compositor::new();
    comp.begin_frame();
    comp.draw_backdrop_blur(0.0, 0.0, 100.0, 100.0, 8.0, 12.0);
    comp.resolve_scene((800.0, 600.0));
    comp.mark_layer_clean(LayerId::DEFAULT);

    comp.begin_frame();
    comp.draw_backdrop_blur(0.0, 0.0, 100.0, 100.0, 8.0, 12.0);
    assert!(!comp.needs_render());

    // Different sigma -> re-render.
    comp.begin_frame();
    comp.draw_backdrop_blur(0.0, 0.0, 100.0, 100.0, 8.0, 20.0);
    assert!(comp.needs_render());
}

#[test]
fn layer_with_backdrop_redraws_when_lower_layer_changes() {
    let mut comp = Compositor::new();
    let glass = comp.create_layer(10);

    comp.begin_frame();
    comp.push(rect(0.0, 0.0, 100.0, 100.0)); // default layer (below)
    comp.push_to_layer(
        glass,
        SceneNode::BackdropBlur {
            x: 10.0,
            y: 10.0,
            w: 50.0,
            h: 50.0,
            corner_radius: 8.0,
            sigma: 12.0,
        },
    );
    comp.resolve_scene((800.0, 600.0));
    comp.mark_layer_clean(LayerId::DEFAULT);
    comp.mark_layer_clean(glass);

    // Lower layer changes; the glass layer's own scene is identical but it
    // must re-encode because its backdrop samples the layer below.
    comp.begin_frame();
    comp.push(rect(0.0, 0.0, 200.0, 100.0)); // CHANGED
    comp.push_to_layer(
        glass,
        SceneNode::BackdropBlur {
            x: 10.0,
            y: 10.0,
            w: 50.0,
            h: 50.0,
            corner_radius: 8.0,
            sigma: 12.0,
        },
    );
    comp.resolve_scene((800.0, 600.0));
    assert!(comp.layer(LayerId::DEFAULT).unwrap().is_dirty());
    assert!(comp.layer(glass).unwrap().is_dirty());

    // And when nothing below changes, the glass layer stays clean.
    comp.mark_layer_clean(LayerId::DEFAULT);
    comp.mark_layer_clean(glass);
    comp.begin_frame();
    comp.push(rect(0.0, 0.0, 200.0, 100.0));
    comp.push_to_layer(
        glass,
        SceneNode::BackdropBlur {
            x: 10.0,
            y: 10.0,
            w: 50.0,
            h: 50.0,
            corner_radius: 8.0,
            sigma: 12.0,
        },
    );
    comp.resolve_scene((800.0, 600.0));
    assert!(!comp.layer(LayerId::DEFAULT).unwrap().is_dirty());
    assert!(!comp.layer(glass).unwrap().is_dirty());
}

#[test]
fn merge_text_groups_keeps_one_range_per_group() {
    fn vertex(x: f32) -> crate::text::TextVertex {
        crate::text::TextVertex {
            position: [x, 0.0],
            uv: [0.0, 0.0],
            color: [1.0; 4],
        }
    }

    // Two groups with the SAME clip (split by a rect between them) plus an
    // empty group: ranges must stay 1:1 with groups, never merged.
    let groups = vec![
        (
            vec![vertex(0.0), vertex(1.0), vertex(2.0), vertex(3.0)],
            vec![0, 1, 2, 2, 3, 0],
            None,
        ),
        (Vec::new(), Vec::new(), None),
        (
            vec![vertex(4.0), vertex(5.0), vertex(6.0), vertex(7.0)],
            vec![0, 1, 2, 2, 3, 0],
            None,
        ),
    ];
    let (_, _, ranges) = merge_text_groups(groups);
    assert_eq!(ranges.len(), 3);
    assert_eq!(ranges[0].index_count, 6);
    assert_eq!(ranges[1].index_count, 0);
    assert_eq!(ranges[2].first_index, 6);
    assert_eq!(ranges[2].index_count, 6);
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
