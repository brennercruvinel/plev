#[cfg(hot_reload)]
impl super::App {
    /// Poll for narrate source changes; returns true when any file was
    /// reprocessed (the caller should invalidate the compositor).
    pub(crate) fn check_narrate_reload(&mut self) -> bool {
        let Some(changed_paths) = self.narrate_watcher.as_ref().and_then(|w| w.poll_changes())
        else {
            return false;
        };
        for path in &changed_paths {
            crate::runtime::hot_reload::process_narrate_file(path);
        }
        !changed_paths.is_empty()
    }
}
