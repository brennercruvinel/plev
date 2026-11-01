//! The system's text size, for `Prefs::text_scale` (ADR 0086), read where
//! each system says it and without a new dependency, as `preference`
//! reads reduce motion:
//!
//! | system  | where |
//! |---------|-------|
//! | Linux   | the desktop portal's `org.gnome.desktop.interface text-scaling-factor` (`gdbus`; GNOME's portal backend republishes the key, so a sandbox reads it), else `gsettings get` of the same key; 0.5 to 3.0, "Large Text" writes 1.25 |
//! | Windows | `reg query HKCU\SOFTWARE\Microsoft\Accessibility /v TextScaleFactor`, a percentage (100 to 225); never set is 100 |
//! | web     | the root element's computed `font-size` over 16 px: desktop browsers fold their default font size into it |
//! | macOS   | none: macOS has no Dynamic Type (HIG), so the size is the app's (a launch argument); `None` |
//! | iOS, Android | not read yet (`UIContentSizeCategory`, the `sp` curve need platform bindings); `None` |
//!
//! The answer is the raw factor; `TextScale::new` holds it to its range.
//! A command takes a few ms: a shell reads when its window first shows
//! and when it takes the focus again, never per frame.

/// The factor the system asks text to be drawn at, `None` where it
/// cannot be read.
pub fn system_text_scale() -> Option<f32> {
    #[cfg(all(
        unix,
        not(any(target_os = "macos", target_os = "ios", target_os = "android"))
    ))]
    {
        portal().or_else(gnome)
    }
    #[cfg(target_os = "windows")]
    {
        windows()
    }
    #[cfg(target_arch = "wasm32")]
    {
        web()
    }
    #[cfg(any(target_os = "macos", target_os = "ios", target_os = "android"))]
    {
        None
    }
}

#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "ios", target_os = "android"))
))]
fn portal() -> Option<f32> {
    let (ok, out, _) = crate::control::accessibility::preference::run(
        "gdbus",
        &[
            "call",
            "--session",
            "--dest",
            "org.freedesktop.portal.Desktop",
            "--object-path",
            "/org/freedesktop/portal/desktop",
            "--method",
            "org.freedesktop.portal.Settings.ReadOne",
            "org.gnome.desktop.interface",
            "text-scaling-factor",
        ],
    )?;
    if !ok {
        return None;
    }
    read_variant(&out)
}

#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "ios", target_os = "android"))
))]
fn gnome() -> Option<f32> {
    let (ok, out, _) = crate::control::accessibility::preference::run(
        "gsettings",
        &["get", "org.gnome.desktop.interface", "text-scaling-factor"],
    )?;
    if !ok {
        return None;
    }
    read_number(&out)
}

/// The double the portal answers, `(<1.25>,)` (or `(<<1.25>>,)` from the
/// older `Read`).
#[cfg(any(
    all(
        unix,
        not(any(target_os = "macos", target_os = "ios", target_os = "android"))
    ),
    test
))]
pub(super) fn read_variant(stdout: &str) -> Option<f32> {
    let inner = stdout.trim().trim_start_matches('(').trim_end_matches(",)");
    let inner = inner.trim_start_matches('<').trim_end_matches('>');
    read_number(inner.trim_start_matches("double "))
}

/// A positive, finite number as a command prints it (`1.25`).
#[cfg(any(
    all(
        unix,
        not(any(target_os = "macos", target_os = "ios", target_os = "android"))
    ),
    test
))]
pub(super) fn read_number(s: &str) -> Option<f32> {
    match s.trim().parse::<f32>() {
        Ok(v) if v.is_finite() && v > 0.0 => Some(v),
        _ => None,
    }
}

#[cfg(target_os = "windows")]
fn windows() -> Option<f32> {
    let value = "TextScaleFactor";
    let (ok, out, _) = crate::control::accessibility::preference::run(
        "reg",
        &[
            "query",
            r"HKCU\SOFTWARE\Microsoft\Accessibility",
            "/v",
            value,
        ],
    )?;
    if !ok {
        // never set: the system's 100%
        return Some(1.0);
    }
    read_percent(&crate::control::accessibility::preference::read_reg(
        &out, value,
    )?)
}

/// A `REG_DWORD` percentage as `reg query` prints it (`0x7d`, 125%), as
/// a factor.
#[cfg(any(target_os = "windows", test))]
pub(super) fn read_percent(data: &str) -> Option<f32> {
    let data = data.trim();
    let percent = match data.strip_prefix("0x") {
        Some(hex) => u32::from_str_radix(hex, 16).ok()?,
        None => data.parse::<u32>().ok()?,
    };
    (percent > 0).then(|| percent as f32 / 100.0)
}

#[cfg(target_arch = "wasm32")]
fn web() -> Option<f32> {
    let window = web_sys::window()?;
    let root = window.document()?.document_element()?;
    let style = window.get_computed_style(&root).ok()??;
    let size = style.get_property_value("font-size").ok()?;
    let px = size.trim().strip_suffix("px")?.trim().parse::<f32>().ok()?;
    (px.is_finite() && px > 0.0).then_some(px / 16.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_portal_answers_a_double() {
        assert_eq!(read_variant("(<1.25>,)\n"), Some(1.25));
        assert_eq!(read_variant("(<<1.5>>,)"), Some(1.5), "the older Read");
        assert_eq!(read_variant("(<double 2.0>,)"), Some(2.0));
        assert_eq!(read_variant("Error: GDBus.Error: not found"), None);
    }

    #[test]
    fn gsettings_prints_the_factor() {
        assert_eq!(read_number("1.25\n"), Some(1.25));
        assert_eq!(read_number("1.0"), Some(1.0));
        assert_eq!(read_number("'x'"), None);
        assert_eq!(read_number("0"), None);
    }

    #[test]
    fn the_registry_holds_a_percentage() {
        assert_eq!(read_percent("0x7d"), Some(1.25));
        assert_eq!(read_percent("0xe1"), Some(2.25));
        assert_eq!(read_percent("150"), Some(1.5));
        assert_eq!(read_percent("0x0"), None);
    }

    /// The real sources, on the machine that runs the test: asking never
    /// panics, whatever is installed.
    #[test]
    fn asking_the_system_never_fails() {
        let _ = system_text_scale();
    }
}
