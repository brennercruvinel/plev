//! Renderer configuration knobs.

/// Configuration for the GPU renderer.
///
/// `Default` matches the engine's historical behavior: 4x MSAA, vsync
/// presentation, and 0.1 tessellation tolerance.
#[derive(Clone, Debug)]
pub struct RenderConfig {
    /// MSAA sample count for layer rendering: 1 (off) or 4.
    /// With 1 sample, layers render directly to their texture without an
    /// intermediate MSAA texture or resolve step.
    pub msaa_samples: u32,
    /// Presentation mode for the surface. Falls back to `AutoVsync` when the
    /// surface does not support the requested mode.
    pub present_mode: wgpu::PresentMode,
    /// Default tolerance for path tessellation (lower = more vertices,
    /// smoother curves).
    pub path_tolerance: f32,
    /// Log a compact perf line (`PerfSnapshot::log_line`) every
    /// `perf_log_interval` frames. Off by default: no spam.
    pub perf_log: bool,
    /// Frames between perf log lines when `perf_log` is on.
    pub perf_log_interval: u32,
    /// Draw the perf HUD overlay (top-right, engine-drawn). While on, the
    /// engine `App` renders continuously instead of on demand: the HUD
    /// text changes every frame, and live measurement needs live frames.
    /// That idle cost is the price of measuring; toggle off to return to
    /// render-on-demand.
    pub perf_hud: bool,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            msaa_samples: 4,
            present_mode: wgpu::PresentMode::AutoVsync,
            path_tolerance: 0.1,
            perf_log: false,
            perf_log_interval: 120,
            perf_hud: false,
        }
    }
}

impl RenderConfig {
    /// Clamp the configured sample count to a value wgpu guarantees (1 or 4).
    pub(crate) fn effective_msaa_samples(&self) -> u32 {
        match self.msaa_samples {
            1 | 4 => self.msaa_samples,
            other => {
                log::warn!("Unsupported msaa_samples {other} -- falling back to 4");
                4
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_matches_legacy_behavior() {
        let config = RenderConfig::default();
        assert_eq!(config.msaa_samples, 4);
        assert_eq!(config.present_mode, wgpu::PresentMode::AutoVsync);
        assert_eq!(config.path_tolerance, 0.1);
        assert_eq!(crate::path::default_tolerance(), 0.1);
        // Perf instrumentation is opt-in: no log spam, no HUD by default.
        assert!(!config.perf_log);
        assert_eq!(config.perf_log_interval, 120);
        assert!(!config.perf_hud);
    }

    #[test]
    fn effective_msaa_clamps_unsupported_values() {
        let mut config = RenderConfig::default();
        config.msaa_samples = 1;
        assert_eq!(config.effective_msaa_samples(), 1);
        config.msaa_samples = 4;
        assert_eq!(config.effective_msaa_samples(), 4);
        config.msaa_samples = 2;
        assert_eq!(config.effective_msaa_samples(), 4);
        config.msaa_samples = 0;
        assert_eq!(config.effective_msaa_samples(), 4);
    }
}
