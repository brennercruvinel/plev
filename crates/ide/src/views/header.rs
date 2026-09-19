//! Top bar: the panel-header proportions (one `Xl` control tall), app
//! name in the title ramp at the active tone with a glass tag, repo and
//! branch centered in base-2r at the faint tone, and the theme toggle as
//! a small glass pill on the right.

use super::workspace::ThemeMode;
use comps::action::{Badge, Button, ButtonSize};
use comps::prelude::{EventResult, Rect, WidgetEvent};
use engine::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::text::TextMeasurer;
use engine::theme::{ControlSize, Theme};

pub struct Header {
    theme_btn: Button,
    theme_btn_rect: Rect,
}

impl Header {
    pub fn new() -> Self {
        Self {
            theme_btn: Button::new("Light").size(ButtonSize::Sm),
            theme_btn_rect: Rect::default(),
        }
    }

    /// Header height: one `Xl` control.
    pub fn height(theme: &Theme) -> f32 {
        theme.control.height(ControlSize::Xl)
    }

    /// Route a pointer event to the theme toggle; `clicked` means toggle.
    pub fn handle_event(&mut self, event: &WidgetEvent) -> EventResult {
        self.theme_btn.handle_event(event, self.theme_btn_rect)
    }

    /// Hit-test for the theme toggle button.
    pub fn hit_test_theme_btn(&self, cx: f32, cy: f32) -> bool {
        self.theme_btn_rect.contains(cx, cy)
    }

    pub fn render(
        &mut self,
        compositor: &mut Compositor,
        theme: &Theme,
        theme_mode: ThemeMode,
        vw: f32,
        sidebar_w: f32,
        repo_label: &str,
        branch_label: &str,
    ) {
        let h = Self::height(theme);
        let pad = theme.spacing.md;
        let x = sidebar_w;
        let w = vw - sidebar_w;

        // Bar surface, same as the sidebar, with a hairline edge below.
        compositor.push(SceneNode::Rect {
            x: 0.0,
            y: 0.0,
            w: vw,
            h,
            color: theme.colors.surface.0,
        });
        compositor.push(SceneNode::Rect {
            x: 0.0,
            y: h - theme.control.edge_width,
            w: vw,
            h: theme.control.edge_width,
            color: theme.colors.divider.0,
        });

        // App name: one style measures and draws, so the tag placed after
        // it never overlaps.
        let title = theme.typography.title();
        let name = "plev ide";
        compositor.push(SceneNode::Text {
            key: TextNodeKey::from_style(name, &title, None),
            x: x + pad,
            y: TextMeasurer::vertical_center(&title, h),
            color: theme.glass.text_active.0,
        });
        let (name_w, _) = TextMeasurer::measure_styled(name, &title, None);
        let tag = Badge::tag("plev");
        let (_, th) = tag.preferred_size(theme);
        tag.render(
            compositor,
            Rect::new(x + pad + name_w + pad, (h - th) / 2.0, 0.0, 0.0),
            theme,
        );

        // Repo name + current branch, centered.
        let center = if branch_label.is_empty() {
            repo_label.to_string()
        } else {
            format!("{repo_label} \u{00B7} {branch_label}")
        };
        let center_style = theme.typography.base_2r();
        let (center_w, _) = TextMeasurer::measure_styled(&center, &center_style, None);
        compositor.push(SceneNode::Text {
            key: TextNodeKey::from_style(&center, &center_style, None),
            x: x + (w - center_w) / 2.0,
            y: TextMeasurer::vertical_center(&center_style, h),
            color: theme.glass.text_faint.0,
        });

        // Theme toggle on the right.
        self.theme_btn.label = match theme_mode {
            ThemeMode::Dark => "Light",
            ThemeMode::Light => "Dark",
        }
        .to_string();
        let (bw, bh) = self.theme_btn.preferred_size(theme);
        self.theme_btn_rect = Rect::new(vw - pad - bw, (h - bh) / 2.0, bw, bh);
        self.theme_btn
            .render(compositor, self.theme_btn_rect, theme);
    }
}
