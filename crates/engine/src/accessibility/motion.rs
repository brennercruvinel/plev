//! The system's "reduce motion", read without a new dependency (task-75
//! phase 1), for `animation::set_reduced_motion`.
//!
//! macOS keeps it as `reduceMotion` in the `com.apple.universalaccess`
//! defaults, and `/usr/bin/defaults read` answers it: `1` when it is on,
//! `0` when it was turned off, and an error that says the pair "does not
//! exist" when nobody ever turned it on (off). The call takes about 10 ms,
//! so it runs when the bridge attaches and when the window takes the focus
//! again (the person may have changed it meanwhile), never per frame.
//!
//! Winit 0.30 has no word on it (its `WindowEvent::Focused` is how the
//! bridge knows to read again). The other systems wait for their target or
//! for a dependency the engine does not have: Windows answers through
//! `SystemParametersInfo` (the `windows` crate), GNOME through `gsettings`
//! (`org.gnome.desktop.interface enable-animations`), the browser through
//! `matchMedia` (a `web-sys` feature the engine does not declare), iOS
//! through UIKit and Android through its settings over JNI. There the
//! answer is `None`, and nothing changes.

use crate::animation;

/// Whether the system asks for reduced motion; `None` where it cannot be
/// read (another system, or `defaults` did not answer).
pub fn system_reduces_motion() -> Option<bool> {
    #[cfg(target_os = "macos")]
    {
        let out = std::process::Command::new("/usr/bin/defaults")
            .args(["read", "com.apple.universalaccess", "reduceMotion"])
            .output();
        match out {
            Ok(out) => read_defaults(
                out.status.success(),
                &String::from_utf8_lossy(&out.stdout),
                &String::from_utf8_lossy(&out.stderr),
            ),
            Err(e) => {
                log::debug!("accessibility: reduce motion unread: {e}");
                None
            }
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}

/// What `defaults read com.apple.universalaccess reduceMotion` said: on
/// for `1` (or `true`), off for `0` (or `false`) and for a pair that does
/// not exist (never turned on); `None` for anything else.
#[cfg(any(target_os = "macos", test))]
fn read_defaults(ok: bool, stdout: &str, stderr: &str) -> Option<bool> {
    if ok {
        return match stdout.trim() {
            "1" | "true" | "YES" => Some(true),
            "0" | "false" | "NO" => Some(false),
            _ => None,
        };
    }
    stderr.contains("does not exist").then_some(false)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_defaults_says_is_read() {
        assert_eq!(read_defaults(true, "1\n", ""), Some(true));
        assert_eq!(read_defaults(true, "0\n", ""), Some(false));
        assert_eq!(read_defaults(true, "true", ""), Some(true));
        assert_eq!(read_defaults(true, "maybe", ""), None);
        let never = "2026-09-25 00:23:20.472 defaults[82835:10452456] \n\
                     The domain/default pair of (com.apple.universalaccess, \
                     reduceMotion) does not exist\n";
        assert_eq!(
            read_defaults(false, "", never),
            Some(false),
            "never set: off"
        );
        assert_eq!(read_defaults(false, "", "Could not read domain"), None);
    }

    /// The real `defaults`, on the machine that runs the test: it answers,
    /// whatever the person chose. Nothing is set by it.
    #[cfg(target_os = "macos")]
    #[test]
    fn macos_answers() {
        assert!(system_reduces_motion().is_some());
        assert!(!animation::reduced_motion(), "reading sets nothing");
    }
}
