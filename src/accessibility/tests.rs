#[cfg(test)]
mod tests {
    use crate::accessibility::focus::{FocusDirection, FocusGraph};
    use crate::accessibility::id_map::{ROOT_NODE_ID, node_id_to_view_id, view_id_to_node_id};
    use crate::accessibility::state::{AccessibilityState, IntentNodeParams};
    use crate::input::{HitRegion, ViewId};
    use accesskit::Role;

    #[test]
    fn view_id_to_node_id_round_trip() {
        for i in [0u64, 1, 42, 1000, u64::MAX - 2] {
            let view = ViewId(i);
            let node = view_id_to_node_id(view);
            let back = node_id_to_view_id(node);
            assert_eq!(back, Some(view), "round trip failed for ViewId({})", i);
        }
    }

    #[test]
    fn root_node_id_has_no_view_id() {
        assert_eq!(node_id_to_view_id(ROOT_NODE_ID), None);
    }

    #[test]
    fn empty_tree_update() {
        let state = AccessibilityState::new();
        let update = state.build_tree_update(None);
        assert_eq!(update.nodes.len(), 1); // just root
        assert!(update.tree.is_some());
    }

    #[test]
    fn tree_with_nodes() {
        let mut state = AccessibilityState::new();
        state.activate();
        state.begin_frame();

        state.push_node(
            ViewId(0),
            Role::Button,
            Some("Click me"),
            [10.0, 10.0, 100.0, 40.0],
            true,
            None,
        );
        state.push_node(
            ViewId(1),
            Role::Label,
            Some("Hello"),
            [10.0, 60.0, 200.0, 20.0],
            false,
            None,
        );

        let update = state.build_tree_update(Some(ViewId(0)));
        assert_eq!(update.nodes.len(), 3);
        assert_eq!(update.focus, view_id_to_node_id(ViewId(0)));
    }

    #[test]
    fn tree_with_hierarchy() {
        let mut state = AccessibilityState::new();
        state.begin_frame();

        state.push_node(
            ViewId(0),
            Role::Group,
            Some("Container"),
            [0.0, 0.0, 300.0, 200.0],
            false,
            None,
        );
        state.push_node(
            ViewId(1),
            Role::Button,
            Some("OK"),
            [10.0, 10.0, 80.0, 30.0],
            true,
            Some(ViewId(0)),
        );
        state.push_node(
            ViewId(2),
            Role::Button,
            Some("Cancel"),
            [100.0, 10.0, 80.0, 30.0],
            true,
            Some(ViewId(0)),
        );

        let update = state.build_tree_update(None);
        assert_eq!(update.nodes.len(), 4); // root + group + 2 buttons
    }

    #[test]
    fn focus_graph_next_previous() {
        let regions = vec![
            HitRegion {
                view_id: ViewId(0),
                x: 0.0,
                y: 0.0,
                w: 100.0,
                h: 40.0,
                focusable: true,
                layer_visible: true,
                layer_opacity: 1.0,
            },
            HitRegion {
                view_id: ViewId(1),
                x: 0.0,
                y: 50.0,
                w: 100.0,
                h: 40.0,
                focusable: true,
                layer_visible: true,
                layer_opacity: 1.0,
            },
            HitRegion {
                view_id: ViewId(2),
                x: 0.0,
                y: 100.0,
                w: 100.0,
                h: 40.0,
                focusable: true,
                layer_visible: true,
                layer_opacity: 1.0,
            },
        ];

        let graph = FocusGraph::from_hit_regions(&regions);

        assert_eq!(graph.next(ViewId(0), FocusDirection::Next), Some(ViewId(1)));
        assert_eq!(graph.next(ViewId(1), FocusDirection::Next), Some(ViewId(2)));
        assert_eq!(graph.next(ViewId(2), FocusDirection::Next), Some(ViewId(0)));

        assert_eq!(
            graph.next(ViewId(0), FocusDirection::Previous),
            Some(ViewId(2))
        );
        assert_eq!(
            graph.next(ViewId(2), FocusDirection::Previous),
            Some(ViewId(1))
        );
    }

    #[test]
    fn focus_graph_directional() {
        let regions = vec![
            HitRegion {
                view_id: ViewId(0),
                x: 0.0,
                y: 0.0,
                w: 50.0,
                h: 50.0,
                focusable: true,
                layer_visible: true,
                layer_opacity: 1.0,
            },
            HitRegion {
                view_id: ViewId(1),
                x: 100.0,
                y: 0.0,
                w: 50.0,
                h: 50.0,
                focusable: true,
                layer_visible: true,
                layer_opacity: 1.0,
            },
            HitRegion {
                view_id: ViewId(2),
                x: 0.0,
                y: 100.0,
                w: 50.0,
                h: 50.0,
                focusable: true,
                layer_visible: true,
                layer_opacity: 1.0,
            },
        ];

        let graph = FocusGraph::from_hit_regions(&regions);

        assert_eq!(
            graph.next(ViewId(0), FocusDirection::Right),
            Some(ViewId(1))
        );
        assert_eq!(graph.next(ViewId(0), FocusDirection::Down), Some(ViewId(2)));
        assert_eq!(graph.next(ViewId(1), FocusDirection::Left), Some(ViewId(0)));
        assert_eq!(graph.next(ViewId(2), FocusDirection::Up), Some(ViewId(0)));
    }

    #[test]
    fn focus_graph_skips_non_focusable() {
        let regions = vec![
            HitRegion {
                view_id: ViewId(0),
                x: 0.0,
                y: 0.0,
                w: 50.0,
                h: 50.0,
                focusable: true,
                layer_visible: true,
                layer_opacity: 1.0,
            },
            HitRegion {
                view_id: ViewId(1),
                x: 50.0,
                y: 0.0,
                w: 50.0,
                h: 50.0,
                focusable: false,
                layer_visible: true,
                layer_opacity: 1.0,
            },
            HitRegion {
                view_id: ViewId(2),
                x: 100.0,
                y: 0.0,
                w: 50.0,
                h: 50.0,
                focusable: true,
                layer_visible: true,
                layer_opacity: 1.0,
            },
        ];

        let graph = FocusGraph::from_hit_regions(&regions);
        assert_eq!(graph.next(ViewId(0), FocusDirection::Next), Some(ViewId(2)));
    }

    #[test]
    fn begin_frame_clears_state() {
        let mut state = AccessibilityState::new();
        state.push_node(
            ViewId(0),
            Role::Button,
            Some("test"),
            [0.0, 0.0, 10.0, 10.0],
            true,
            None,
        );
        assert_eq!(state.nodes.len(), 1);

        state.begin_frame();
        assert_eq!(state.nodes.len(), 0);
        assert!(state.root_children.is_empty());
    }

    #[test]
    fn push_node_with_intent_destructive() {
        use crate::theme::{Intent, Theme};
        let mut state = AccessibilityState::new();
        state.begin_frame();
        let theme = Theme::dark();
        state.push_node_with_intent(
            ViewId(0),
            IntentNodeParams {
                intent: Some(Intent::Destructive),
                theme: &theme,
                label: Some("Delete"),
                bounds: [0.0, 0.0, 80.0, 30.0],
                focusable: true,
                parent: None,
            },
        );
        let update = state.build_tree_update(None);
        assert_eq!(update.nodes.len(), 2);
    }

    #[test]
    fn push_node_with_intent_informational() {
        use crate::theme::{Intent, Theme};
        let mut state = AccessibilityState::new();
        state.begin_frame();
        let theme = Theme::dark();
        state.push_node_with_intent(
            ViewId(0),
            IntentNodeParams {
                intent: Some(Intent::Informational),
                theme: &theme,
                label: Some("Status"),
                bounds: [0.0, 0.0, 200.0, 20.0],
                focusable: false,
                parent: None,
            },
        );
        let update = state.build_tree_update(None);
        assert_eq!(update.nodes.len(), 2);
    }

    #[test]
    fn push_node_no_intent_focusable_is_button() {
        use crate::theme::Theme;
        let mut state = AccessibilityState::new();
        state.begin_frame();
        let theme = Theme::dark();
        state.push_node_with_intent(
            ViewId(0),
            IntentNodeParams {
                intent: None,
                theme: &theme,
                label: Some("Click"),
                bounds: [0.0, 0.0, 80.0, 30.0],
                focusable: true,
                parent: None,
            },
        );
        let update = state.build_tree_update(None);
        assert_eq!(update.nodes.len(), 2);
    }
}
