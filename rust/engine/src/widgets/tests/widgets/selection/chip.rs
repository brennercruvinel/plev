//! Selection tests: the chip's intents.

use super::*;

/// An informational chip and a constructive one stand apart on the color
/// wheel under every theme: the hoff's info is a green, as its success,
/// so its chip takes the kit's blue.
#[test]
fn an_informational_chip_is_not_a_constructive_one() {
    use crate::widgets::selection::tokens::hue_sextant;
    for name in ["core", "hoff", "hoff-light", "plev", "catppuccin"] {
        let theme = Theme::named(name).unwrap();
        let info = theme.selection.chip.info;
        assert!(hue_sextant(info).is_some(), "{name}: a hue");
        assert_ne!(
            hue_sextant(info),
            hue_sextant(theme.colors.success),
            "{name}"
        );
    }
    assert_eq!(
        Theme::hoff_core().selection.chip.info,
        Theme::hoff_core().colors.info
    );
    assert_eq!(
        hue_sextant(crate::styling::color::Color::hex(0x808080)),
        None
    );
}
