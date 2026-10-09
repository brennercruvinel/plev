//! Component<L> -- wraps a Lifecycle impl with persistent state.

use crate::compositor::SceneNode;
use crate::view::ViewContext;

use super::Lifecycle;

pub struct Component<L: Lifecycle> {
    inner: L,
    state: L::State,
    mounted: bool,
    cached_nodes: Option<Vec<SceneNode>>,
    needs_render: bool,
}

impl<L: Lifecycle> Component<L> {
    pub fn new(inner: L) -> Self {
        let state = inner.initial_state();
        Self {
            inner,
            state,
            mounted: false,
            cached_nodes: None,
            needs_render: true,
        }
    }

    pub fn render(&mut self, cx: &mut ViewContext) -> Vec<SceneNode> {
        if !self.mounted {
            self.inner.on_mount(&mut self.state);
            self.mounted = true;
            self.needs_render = true;
        } else {
            self.inner.on_update(&mut self.state);
        }

        if !self.needs_render
            && let Some(ref cached) = self.cached_nodes
        {
            return cached.clone();
        }

        let nodes = self.inner.render(&self.state, cx);
        self.cached_nodes = Some(nodes.clone());
        self.needs_render = false;
        nodes
    }

    pub fn state(&self) -> &L::State {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut L::State {
        self.needs_render = true;
        &mut self.state
    }

    /// Mark the component as needing a re-render on the next frame.
    pub fn invalidate(&mut self) {
        self.needs_render = true;
    }
}

impl<L: Lifecycle> Drop for Component<L> {
    fn drop(&mut self) {
        if self.mounted {
            self.inner.on_unmount(&mut self.state);
        }
    }
}
