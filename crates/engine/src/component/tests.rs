#[cfg(test)]
// The inner module only carries the cfg(test) gate for this
// tests.rs file; the same-name nesting is deliberate.
#[allow(clippy::module_inception)]
mod tests {
    use crate::component::{Component, Lifecycle};
    use crate::compositor::{SceneNode, TextNodeKey};
    use crate::view::ViewContext;
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

    // -- Minimal test lifecycle --

    struct TestCounter;

    impl Lifecycle for TestCounter {
        type State = u64;

        fn initial_state(&self) -> u64 {
            0
        }

        fn on_update(&self, count: &mut u64) {
            *count += 1;
        }

        fn render(&self, count: &u64, _cx: &mut ViewContext) -> Vec<SceneNode> {
            vec![SceneNode::Rect {
                x: 0.0,
                y: 0.0,
                w: 100.0,
                h: 50.0,
                color: [*count as f32, 0.0, 0.0, 1.0],
            }]
        }
    }

    fn test_cx() -> ViewContext {
        ViewContext::new(800.0, 600.0)
    }

    #[test]
    fn initial_state_set_on_new() {
        let comp = Component::new(TestCounter);
        assert_eq!(*comp.state(), 0);
    }

    #[test]
    fn on_mount_fires_on_first_render() {
        struct MountTracker;
        impl Lifecycle for MountTracker {
            type State = bool;
            fn initial_state(&self) -> bool {
                false
            }
            fn on_mount(&self, state: &mut bool) {
                *state = true;
            }
            fn render(&self, state: &bool, _cx: &mut ViewContext) -> Vec<SceneNode> {
                assert!(*state, "state should be true after mount");
                vec![]
            }
        }

        let mut comp = Component::new(MountTracker);
        assert!(!*comp.state());
        comp.render(&mut test_cx());
        assert!(*comp.state());
    }

    #[test]
    fn on_update_fires_on_subsequent_renders() {
        let mut comp = Component::new(TestCounter);
        let mut cx = test_cx();
        comp.render(&mut cx);
        assert_eq!(*comp.state(), 0);
        comp.render(&mut cx);
        assert_eq!(*comp.state(), 1);
        comp.render(&mut cx);
        assert_eq!(*comp.state(), 2);
    }

    #[test]
    fn render_produces_nodes_from_state() {
        let mut comp = Component::new(TestCounter);
        let mut cx = test_cx();
        let nodes = comp.render(&mut cx);
        assert_eq!(nodes.len(), 1);
        let SceneNode::Rect { color, .. } = &nodes[0] else {
            panic!("Expected Rect, got {:?}", &nodes[0]);
        };
        assert_eq!(color[0], 0.0);
    }

    #[test]
    fn state_survives_between_frames() {
        let mut comp = Component::new(TestCounter);
        let mut cx = test_cx();
        for _ in 0..100 {
            comp.render(&mut cx);
        }
        assert_eq!(*comp.state(), 99);
    }

    #[test]
    fn state_mut_allows_external_mutation() {
        let mut comp = Component::new(TestCounter);
        *comp.state_mut() = 42;
        assert_eq!(*comp.state(), 42);
    }

    #[test]
    fn on_unmount_fires_on_drop() {
        static UNMOUNTED: AtomicBool = AtomicBool::new(false);
        struct UnmountTracker;
        impl Lifecycle for UnmountTracker {
            type State = ();
            fn initial_state(&self) {}
            fn on_unmount(&self, _state: &mut ()) {
                UNMOUNTED.store(true, Ordering::SeqCst);
            }
            fn render(&self, _state: &(), _cx: &mut ViewContext) -> Vec<SceneNode> {
                vec![]
            }
        }
        UNMOUNTED.store(false, Ordering::SeqCst);
        {
            let mut comp = Component::new(UnmountTracker);
            comp.render(&mut test_cx());
            assert!(!UNMOUNTED.load(Ordering::SeqCst));
        }
        assert!(UNMOUNTED.load(Ordering::SeqCst));
    }

    #[test]
    fn unmount_not_called_if_never_mounted() {
        static UNMOUNTED: AtomicBool = AtomicBool::new(false);
        struct UnmountTracker2;
        impl Lifecycle for UnmountTracker2 {
            type State = ();
            fn initial_state(&self) {}
            fn on_unmount(&self, _state: &mut ()) {
                UNMOUNTED.store(true, Ordering::SeqCst);
            }
            fn render(&self, _state: &(), _cx: &mut ViewContext) -> Vec<SceneNode> {
                vec![]
            }
        }
        UNMOUNTED.store(false, Ordering::SeqCst);
        {
            let _comp = Component::new(UnmountTracker2);
        }
        assert!(!UNMOUNTED.load(Ordering::SeqCst));
    }

    #[test]
    fn complex_state_struct() {
        struct Dashboard;
        #[derive(Debug)]
        struct DashState {
            count: u32,
            label: String,
        }
        impl Lifecycle for Dashboard {
            type State = DashState;
            fn initial_state(&self) -> DashState {
                DashState {
                    count: 0,
                    label: "init".to_string(),
                }
            }
            fn on_mount(&self, state: &mut DashState) {
                state.label = "mounted".to_string();
            }
            fn on_update(&self, state: &mut DashState) {
                state.count += 1;
                state.label = format!("frame {}", state.count);
            }
            fn render(&self, state: &DashState, _cx: &mut ViewContext) -> Vec<SceneNode> {
                vec![SceneNode::Text {
                    key: TextNodeKey::new(&state.label, 16.0, 20.0, None),
                    x: 0.0,
                    y: 0.0,
                    color: [1.0, 1.0, 1.0, 1.0],
                }]
            }
        }
        let mut comp = Component::new(Dashboard);
        let mut cx = test_cx();
        comp.render(&mut cx);
        assert_eq!(comp.state().label, "mounted");
        assert_eq!(comp.state().count, 0);
        comp.render(&mut cx);
        assert_eq!(comp.state().label, "frame 1");
        comp.render(&mut cx);
        assert_eq!(comp.state().label, "frame 2");
    }

    #[test]
    fn cache_returns_same_nodes_without_state_change() {
        struct StaticWidget;
        impl Lifecycle for StaticWidget {
            type State = ();
            fn initial_state(&self) {}
            fn render(&self, _state: &(), _cx: &mut ViewContext) -> Vec<SceneNode> {
                vec![SceneNode::Rect {
                    x: 10.0,
                    y: 20.0,
                    w: 100.0,
                    h: 50.0,
                    color: [1.0, 0.0, 0.0, 1.0],
                }]
            }
        }
        let mut comp = Component::new(StaticWidget);
        let mut cx = test_cx();
        let nodes1 = comp.render(&mut cx);
        let nodes2 = comp.render(&mut cx);
        assert_eq!(nodes1.len(), nodes2.len());
        let SceneNode::Rect { x: x1, y: y1, .. } = &nodes1[0] else {
            panic!("Expected Rect");
        };
        let SceneNode::Rect { x: x2, y: y2, .. } = &nodes2[0] else {
            panic!("Expected Rect");
        };
        assert_eq!(*x1, *x2);
        assert_eq!(*y1, *y2);
    }

    #[test]
    fn cache_invalidated_by_state_mut() {
        struct Counter;
        impl Lifecycle for Counter {
            type State = u32;
            fn initial_state(&self) -> u32 {
                0
            }
            fn render(&self, state: &u32, _cx: &mut ViewContext) -> Vec<SceneNode> {
                vec![SceneNode::Rect {
                    x: *state as f32,
                    y: 0.0,
                    w: 10.0,
                    h: 10.0,
                    color: [1.0; 4],
                }]
            }
        }
        let mut comp = Component::new(Counter);
        let mut cx = test_cx();
        let nodes = comp.render(&mut cx);
        let SceneNode::Rect { x, .. } = &nodes[0] else {
            panic!("Expected Rect");
        };
        assert_eq!(*x, 0.0);
        *comp.state_mut() = 42;
        let nodes = comp.render(&mut cx);
        let SceneNode::Rect { x, .. } = &nodes[0] else {
            panic!("Expected Rect");
        };
        assert_eq!(*x, 42.0);
    }

    #[test]
    fn invalidate_forces_rerender() {
        static RENDER_COUNT: AtomicU32 = AtomicU32::new(0);
        struct RenderTracker;
        impl Lifecycle for RenderTracker {
            type State = ();
            fn initial_state(&self) {}
            fn render(&self, _state: &(), _cx: &mut ViewContext) -> Vec<SceneNode> {
                RENDER_COUNT.fetch_add(1, Ordering::SeqCst);
                vec![]
            }
        }
        RENDER_COUNT.store(0, Ordering::SeqCst);
        let mut comp = Component::new(RenderTracker);
        let mut cx = test_cx();
        comp.render(&mut cx);
        assert_eq!(RENDER_COUNT.load(Ordering::SeqCst), 1);
        comp.render(&mut cx);
        assert_eq!(RENDER_COUNT.load(Ordering::SeqCst), 1);
        comp.invalidate();
        comp.render(&mut cx);
        assert_eq!(RENDER_COUNT.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn render_with_theme_in_context() {
        use crate::theme::Theme;
        struct ThemeReader;
        impl Lifecycle for ThemeReader {
            type State = bool;
            fn initial_state(&self) -> bool {
                false
            }
            fn render(&self, _state: &bool, cx: &mut ViewContext) -> Vec<SceneNode> {
                assert!(cx.theme.is_some(), "theme should be present in ViewContext");
                let theme = cx.theme.as_ref().unwrap();
                let color = theme.colors.accent.to_array();
                vec![SceneNode::Rect {
                    x: 0.0,
                    y: 0.0,
                    w: 100.0,
                    h: 50.0,
                    color,
                }]
            }
        }
        let mut comp = Component::new(ThemeReader);
        let mut cx = ViewContext::new(800.0, 600.0).with_theme(Theme::dark());
        let nodes = comp.render(&mut cx);
        assert_eq!(nodes.len(), 1);
        if let SceneNode::Rect { color, .. } = &nodes[0] {
            let accent = Theme::dark().colors.accent.to_array();
            assert_eq!(*color, accent);
        }
    }
}
