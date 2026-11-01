//! The system's display preferences a person sets once for every app:
//! "reduce motion" and "increase contrast", read where the system says
//! them, without a new dependency (the platform's own command, or the
//! browser's media query).
//!
//! | system  | reduce motion | more contrast |
//! |---------|---------------|---------------|
//! | macOS   | `defaults read com.apple.universalaccess reduceMotion` | the same domain, `increaseContrast` |
//! | Linux   | the desktop portal's `org.freedesktop.appearance reduced-motion` (`gdbus`), else GNOME's `org.gnome.desktop.interface enable-animations` (`gsettings`, off means reduce) | the portal's `contrast` (1 is high), else GNOME's `org.gnome.desktop.a11y.interface high-contrast` |
//! | Windows | `reg query` of `HKCU\Control Panel\Desktop\WindowMetrics` `MinAnimate` (`0`: the "Animation effects" switch is off) | `HKCU\Control Panel\Accessibility\HighContrast` `Flags`, bit `HCF_HIGHCONTRASTON` (a contrast theme is on) |
//! | web     | `matchMedia("(prefers-reduced-motion: reduce)")` | `matchMedia("(prefers-contrast: more)")` |
//! | iOS, Android | not read: UIKit's `isReduceMotionEnabled` and Android's `ANIMATOR_DURATION_SCALE` need the platform bindings the engine does not have; the answer is `None` | the same (`isDarkerSystemColorsEnabled`, `isHighTextContrastEnabled`) |
//!
//! A command takes a few ms, so a shell reads when its window first shows
//! and when it takes the focus again (the person may have changed it
//! meanwhile), never per frame. A command that is missing or fails, or a
//! portal without the key (older than 1.20 for `reduced-motion`), gives
//! `None` for that source and the next one is asked. On Windows,
//! `MinAnimate` is the registry's side of the switch; the documented API
//! (`SystemParametersInfo(SPI_GETCLIENTAREAANIMATION)`) needs the
//! `windows` crate, which the engine does not declare.

/// A display preference of the system.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preference {
    /// Motion ends at once (`animation::set_reduced_motion`).
    ReduceMotion,
    /// Higher contrast between text, edges and their background.
    MoreContrast,
}

thread_local! {
    static MORE_CONTRAST: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// This thread's contrast preference: what the system said at the last
/// [`follow_contrast`], or what a test or a flag set. Nothing reacts to
/// it yet: the theme's contrast mode (`Contrast::More`, ADR 0086) is
/// front F's, and reads it here when it lands.
pub fn more_contrast() -> bool {
    MORE_CONTRAST.with(std::cell::Cell::get)
}

/// Sets this thread's contrast preference ([`more_contrast`]).
pub fn set_more_contrast(on: bool) {
    MORE_CONTRAST.with(|c| c.set(on));
}

/// Reads the system's contrast and makes it this thread's; where it
/// cannot be read the thread's stays as it was. The answer, for the log.
pub fn follow_contrast() -> Option<bool> {
    let more = read(Preference::MoreContrast);
    if let Some(on) = more {
        if on != more_contrast() {
            log::info!(
                "accessibility: more contrast {}",
                if on { "on" } else { "off" }
            );
        }
        set_more_contrast(on);
    }
    more
}

/// What the system says of `p`: `None` where it cannot be read (no
/// source on this system, or none answered).
pub fn read(p: Preference) -> Option<bool> {
    #[cfg(target_os = "macos")]
    {
        macos(p)
    }
    #[cfg(all(
        unix,
        not(any(target_os = "macos", target_os = "ios", target_os = "android"))
    ))]
    {
        portal(p).or_else(|| gnome(p))
    }
    #[cfg(target_os = "windows")]
    {
        windows(p)
    }
    #[cfg(target_arch = "wasm32")]
    {
        web(p)
    }
    #[cfg(any(target_os = "ios", target_os = "android"))]
    {
        let _ = p;
        None
    }
}

/// A command's answer, `(succeeded, stdout, stderr)`; `None` when it
/// could not run.
#[cfg(not(any(target_arch = "wasm32", target_os = "ios", target_os = "android")))]
fn run(program: &str, args: &[&str]) -> Option<(bool, String, String)> {
    match std::process::Command::new(program).args(args).output() {
        Ok(out) => Some((
            out.status.success(),
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        )),
        Err(e) => {
            log::debug!("accessibility: {program} did not run: {e}");
            None
        }
    }
}

#[cfg(target_os = "macos")]
fn macos(p: Preference) -> Option<bool> {
    let key = match p {
        Preference::ReduceMotion => "reduceMotion",
        Preference::MoreContrast => "increaseContrast",
    };
    let (ok, out, err) = run(
        "/usr/bin/defaults",
        &["read", "com.apple.universalaccess", key],
    )?;
    read_defaults(ok, &out, &err)
}

/// What `defaults read com.apple.universalaccess <key>` said: on for `1`
/// (or `true`), off for `0` (or `false`) and for a key that does not
/// exist (never turned on: "does not exist" up to macOS 26, "Could not
/// find key" from macOS 27); `None` for anything else.
#[cfg(any(target_os = "macos", test))]
pub(super) fn read_defaults(ok: bool, stdout: &str, stderr: &str) -> Option<bool> {
    if ok {
        return match stdout.trim() {
            "1" | "true" | "YES" => Some(true),
            "0" | "false" | "NO" => Some(false),
            _ => None,
        };
    }
    (stderr.contains("does not exist") || stderr.contains("Could not find key")).then_some(false)
}

#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "ios", target_os = "android"))
))]
fn portal(p: Preference) -> Option<bool> {
    let key = match p {
        Preference::ReduceMotion => "reduced-motion",
        Preference::MoreContrast => "contrast",
    };
    let (ok, out, _) = run(
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
            "org.freedesktop.appearance",
            key,
        ],
    )?;
    if !ok {
        return None;
    }
    read_portal(&out)
}

/// The portal's answer, `(<uint32 1>,)` (or `(<<uint32 1>>,)` from the
/// older `Read`): both keys are `0` for no preference and `1` for reduced
/// motion, or for high contrast.
#[cfg(any(
    all(
        unix,
        not(any(target_os = "macos", target_os = "ios", target_os = "android"))
    ),
    test
))]
pub(super) fn read_portal(stdout: &str) -> Option<bool> {
    let at = stdout.find("uint32")? + "uint32".len();
    let digits: String = stdout[at..]
        .trim_start()
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    match digits.parse::<u32>().ok()? {
        0 => Some(false),
        1 => Some(true),
        _ => None,
    }
}

#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "ios", target_os = "android"))
))]
fn gnome(p: Preference) -> Option<bool> {
    let (schema, key, on_means) = match p {
        Preference::ReduceMotion => ("org.gnome.desktop.interface", "enable-animations", false),
        Preference::MoreContrast => ("org.gnome.desktop.a11y.interface", "high-contrast", true),
    };
    let (ok, out, _) = run("gsettings", &["get", schema, key])?;
    if !ok {
        return None;
    }
    read_gsettings(&out).map(|value| value == on_means)
}

/// A boolean as `gsettings get` prints it.
#[cfg(any(
    all(
        unix,
        not(any(target_os = "macos", target_os = "ios", target_os = "android"))
    ),
    test
))]
pub(super) fn read_gsettings(stdout: &str) -> Option<bool> {
    match stdout.trim() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

#[cfg(target_os = "windows")]
fn windows(p: Preference) -> Option<bool> {
    let (path, value) = match p {
        Preference::ReduceMotion => (r"HKCU\Control Panel\Desktop\WindowMetrics", "MinAnimate"),
        Preference::MoreContrast => (r"HKCU\Control Panel\Accessibility\HighContrast", "Flags"),
    };
    let (ok, out, _) = run("reg", &["query", path, "/v", value])?;
    if !ok {
        // never set: the system's default, animations on and no
        // contrast theme
        return Some(false);
    }
    let data = read_reg(&out, value)?;
    match p {
        Preference::ReduceMotion => Some(data.trim() == "0"),
        // HCF_HIGHCONTRASTON
        Preference::MoreContrast => Some(data.trim().parse::<u32>().ok()? & 0x1 != 0),
    }
}

/// The data of `value` in what `reg query <key> /v <value>` printed: the
/// line `    <value>    REG_SZ    <data>`.
#[cfg(any(target_os = "windows", test))]
pub(super) fn read_reg(stdout: &str, value: &str) -> Option<String> {
    stdout.lines().find_map(|line| {
        let mut parts = line.split_whitespace();
        (parts.next()? == value).then_some(())?;
        let _kind = parts.next()?;
        Some(parts.collect::<Vec<_>>().join(" "))
    })
}

#[cfg(target_arch = "wasm32")]
fn web(p: Preference) -> Option<bool> {
    let query = match p {
        Preference::ReduceMotion => "(prefers-reduced-motion: reduce)",
        Preference::MoreContrast => "(prefers-contrast: more)",
    };
    let list = web_sys::window()?.match_media(query).ok()??;
    Some(list.matches())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_portal_says_one_for_on_and_zero_for_no_preference() {
        assert_eq!(read_portal("(<uint32 1>,)\n"), Some(true));
        assert_eq!(read_portal("(<uint32 0>,)\n"), Some(false));
        assert_eq!(
            read_portal("(<<uint32 1>>,)\n"),
            Some(true),
            "the older Read"
        );
        assert_eq!(read_portal("(<uint32 7>,)"), None);
        assert_eq!(read_portal("Error: GDBus.Error: not found"), None);
    }

    #[test]
    fn gsettings_prints_a_boolean() {
        assert_eq!(read_gsettings("false\n"), Some(false));
        assert_eq!(read_gsettings("true"), Some(true));
        assert_eq!(read_gsettings("'x'"), None);
    }

    #[test]
    fn reg_query_gives_the_values_data() {
        let out = "\r\nHKEY_CURRENT_USER\\Control Panel\\Desktop\\WindowMetrics\r\n    \
                   MinAnimate    REG_SZ    0\r\n\r\n";
        assert_eq!(read_reg(out, "MinAnimate").as_deref(), Some("0"));
        let hc = "\r\nHKEY_CURRENT_USER\\Control Panel\\Accessibility\\HighContrast\r\n    \
                  Flags    REG_SZ    127\r\n";
        assert_eq!(read_reg(hc, "Flags").as_deref(), Some("127"));
        assert_eq!(read_reg(hc, "MinAnimate"), None);
    }

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
        let never_27 = "Error: Could not find key 'reduceMotion' in domain \
                        'com.apple.universalaccess'.\n";
        assert_eq!(
            read_defaults(false, "", never_27),
            Some(false),
            "macOS 27: off"
        );
        assert_eq!(read_defaults(false, "", "Could not read domain"), None);
    }

    /// The real sources, on the machine that runs the test: asking never
    /// panics, whatever is installed, and sets nothing.
    #[test]
    fn asking_the_system_never_fails() {
        let _ = read(Preference::ReduceMotion);
        let _ = read(Preference::MoreContrast);
        assert!(!crate::compose::animation::reduced_motion());
    }
}
