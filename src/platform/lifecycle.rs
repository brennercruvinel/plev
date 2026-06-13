/// Application lifecycle state, unified across platforms.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppState {
    /// App is in the foreground and interactive.
    Active,
    /// App is transitioning to background (iOS/Android).
    Background,
    /// App is fully suspended — GPU surface may be invalid (Android).
    Suspended,
}

impl std::fmt::Display for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppState::Active => write!(f, "Active"),
            AppState::Background => write!(f, "Background"),
            AppState::Suspended => write!(f, "Suspended"),
        }
    }
}

/// Manages application lifecycle state and transition callbacks.
pub struct LifecycleManager {
    state: AppState,
    callbacks: Vec<Box<dyn Fn(AppState, AppState)>>,
    memory_callbacks: Vec<Box<dyn Fn()>>,
}

impl Default for LifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LifecycleManager {
    pub fn new() -> Self {
        Self {
            state: AppState::Active,
            callbacks: Vec::new(),
            memory_callbacks: Vec::new(),
        }
    }

    pub fn state(&self) -> AppState {
        self.state
    }

    /// Transition to a new state. Fires registered callbacks with (old, new).
    pub fn transition_to(&mut self, new_state: AppState) {
        if self.state == new_state {
            return;
        }
        let old = self.state;
        self.state = new_state;
        log::info!("Lifecycle: {old:?} -> {new_state:?}");
        for cb in &self.callbacks {
            cb(old, new_state);
        }
    }

    /// Register a callback for state transitions.
    pub fn on_transition(&mut self, cb: impl Fn(AppState, AppState) + 'static) {
        self.callbacks.push(Box::new(cb));
    }

    /// Fire memory warning callbacks.
    pub fn fire_memory_warning(&self) {
        log::warn!("Memory warning received");
        for cb in &self.memory_callbacks {
            cb();
        }
    }

    /// Register a callback for memory warnings.
    pub fn on_memory_warning(&mut self, cb: impl Fn() + 'static) {
        self.memory_callbacks.push(Box::new(cb));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn initial_state_is_active() {
        let lm = LifecycleManager::new();
        assert_eq!(lm.state(), AppState::Active);
    }

    #[test]
    fn transition_changes_state() {
        let mut lm = LifecycleManager::new();
        lm.transition_to(AppState::Suspended);
        assert_eq!(lm.state(), AppState::Suspended);
    }

    #[test]
    fn same_state_transition_is_noop() {
        let called = Rc::new(Cell::new(false));
        let called_clone = called.clone();
        let mut lm = LifecycleManager::new();
        lm.on_transition(move |_, _| called_clone.set(true));
        lm.transition_to(AppState::Active);
        assert!(!called.get());
    }

    #[test]
    fn transition_fires_callback() {
        let called = Rc::new(Cell::new(false));
        let called_clone = called.clone();
        let mut lm = LifecycleManager::new();
        lm.on_transition(move |old, new| {
            assert_eq!(old, AppState::Active);
            assert_eq!(new, AppState::Suspended);
            called_clone.set(true);
        });
        lm.transition_to(AppState::Suspended);
        assert!(called.get());
    }

    #[test]
    fn memory_warning_fires_callback() {
        let called = Rc::new(Cell::new(false));
        let called_clone = called.clone();
        let mut lm = LifecycleManager::new();
        lm.on_memory_warning(move || called_clone.set(true));
        lm.fire_memory_warning();
        assert!(called.get());
    }

    #[test]
    fn app_state_display() {
        assert_eq!(format!("{}", AppState::Active), "Active");
        assert_eq!(format!("{}", AppState::Background), "Background");
        assert_eq!(format!("{}", AppState::Suspended), "Suspended");
    }
}
