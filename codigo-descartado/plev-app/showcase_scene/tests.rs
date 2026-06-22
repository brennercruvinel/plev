#[cfg(test)]
mod tests {
    use crate::showcase_scene::helpers::clear_color;
    use crate::theme::Theme;

    #[test]
    fn clear_color_from_dark_theme() {
        let theme = Theme::dark();
        let cc = clear_color(&theme);
        // All channels should be in [0,1] range
        for ch in &cc {
            assert!(*ch >= 0.0 && *ch <= 1.0, "channel out of range: {}", ch);
        }
        // Alpha should be 1.0
        assert_eq!(cc[3], 1.0);
    }
}
