#[cfg(feature = "hot-reload")]
use super::state::GpuState;

#[cfg(feature = "hot-reload")]
impl super::App {
    pub(crate) fn check_shader_reload(&mut self) {
        let changed_paths = match self.shader_watcher.as_ref().and_then(|w| w.poll_changes()) {
            Some(paths) => paths,
            None => return,
        };

        let changed: Vec<String> = changed_paths
            .iter()
            .filter_map(|p| p.file_name())
            .filter_map(|f| f.to_str())
            .filter(|f| f.ends_with(".wgsl"))
            .map(String::from)
            .collect();

        if changed.is_empty() {
            return;
        }

        log::info!("Shader files changed: {:?}", changed);

        let GpuState::Ready {
            ref mut gpu,
            ref mut effect_processor,
            ..
        } = self.state
        else {
            return;
        };

        for filename in &changed {
            let source = crate::hot_reload::shader_source(filename);
            gpu.reload_shader(filename, &source);
            effect_processor.reload_shader(&gpu.device, filename, &source);
        }
    }

    pub(crate) fn check_narrate_reload(&mut self) {
        let changed_paths = match self.narrate_watcher.as_ref().and_then(|w| w.poll_changes()) {
            Some(paths) => paths,
            None => return,
        };

        for path in &changed_paths {
            crate::hot_reload::process_narrate_file(path);
        }
    }
}
