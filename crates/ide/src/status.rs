//! File status colors: git domain semantics mapped onto the theme's
//! intent colors. The design system is monochrome with three chromatic
//! accents, so file states map onto those instead of a foreign
//! yellow/blue/purple palette.

use engine::color::Color;
use engine::theme::Theme;

pub struct StatusColors {
    pub modified: Color,
    pub added: Color,
    pub deleted: Color,
    pub renamed: Color,
    pub untracked: Color,
}

impl StatusColors {
    pub fn of(theme: &Theme) -> Self {
        Self {
            modified: theme.colors.warning,
            added: theme.colors.success,
            deleted: theme.colors.danger,
            renamed: theme.colors.info,
            untracked: theme.glass.text_faint,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_colors_reuse_the_theme_accents() {
        let t = Theme::hoff();
        let s = StatusColors::of(&t);
        assert_eq!(s.added.0, t.colors.success.0);
        assert_eq!(s.deleted.0, t.colors.danger.0);
        assert_eq!(s.modified.0, t.colors.warning.0);
        assert_eq!(s.renamed.0, t.colors.info.0);
        assert_eq!(s.untracked.0, t.glass.text_faint.0);
    }
}
