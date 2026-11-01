//! The system's "reduce motion", for `animation::set_reduced_motion`
//! (task-75 phase 1; every system since task-104k). Where and how each
//! system says it is `preference`'s ([`super::preference::read`]): macOS
//! `defaults`, the Linux desktop portal or GNOME's settings, the Windows
//! registry, the browser's `prefers-reduced-motion`; iOS and Android are
//! not read yet (`None`, nothing changes). The read takes a few ms, so it
//! runs when the bridge attaches and when the window takes the focus
//! again (the person may have changed it meanwhile), never per frame.
//! Winit 0.30 has no word on it (its `WindowEvent::Focused` is how the
//! bridge knows to read again).

use crate::compose::animation;

/// Whether the system asks for reduced motion; `None` where it cannot be
/// read (a system without a source, or none answered).
pub fn system_reduces_motion() -> Option<bool> {
    super::preference::read(super::preference::Preference::ReduceMotion)
}

/// Reads the system's preference and makes it this thread's motion
/// (`animation::set_reduced_motion`); where it cannot be read, the
/// thread's motion stays as it was. The answer, for the log.
pub fn follow_system() -> Option<bool> {
    let reduced = system_reduces_motion();
    if let Some(on) = reduced {
        if on != animation::reduced_motion() {
            log::info!(
                "accessibility: reduce motion {}",
                if on { "on" } else { "off" }
            );
        }
        animation::set_reduced_motion(on);
    }
    reduced
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    /// The real `defaults`, on the machine that runs the test: it answers,
    /// whatever the person chose. Nothing is set by it.
    #[test]
    fn macos_answers() {
        assert!(system_reduces_motion().is_some());
        assert!(!animation::reduced_motion(), "reading sets nothing");
    }
}
