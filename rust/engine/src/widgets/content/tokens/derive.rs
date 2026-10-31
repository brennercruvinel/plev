//! Every other theme's values: the kit's geometry and motion in the
//! theme's palette (`derive`).

use super::*;

impl DisclosureTokens {
    /// The kit's geometry and motion in a theme's palette: ink is the
    /// text, the gray is the dim text, the line is the divider, the wash
    /// is the hover tone, the paper is the surface.
    pub fn derive(theme: &Theme) -> Self {
        let colors = &theme.colors;
        let kit = Self::hoff_core();
        let wash = DisclosureSurface {
            fill: colors.bg_hover,
            border: NONE,
        };
        let accordion = DisclosureRecipe {
            label: colors.text,
            chevron_color: colors.text_dim,
            divider: colors.divider,
            body: colors.text_dim,
            grid: kit.accordion.grid.map(|grid| GridRecipe {
                fill: colors.bg_hover,
                ring_outer: (grid.ring_outer.0, tint(colors.text_dim, 0.5)),
                ring_inner: (grid.ring_inner.0, colors.surface),
                glyph_color: colors.text_dim,
                ..grid
            }),
            ..kit.accordion
        };
        let collapsible = DisclosureRecipe {
            hover: wash,
            open: wash,
            current: DisclosureSurface {
                fill: colors.bg_hover,
                border: colors.divider,
            },
            label: colors.text,
            chevron_color: colors.text_dim,
            tile: kit.collapsible.tile.map(|tile| DisclosureTile {
                fill_current: colors.surface,
                shadow_current: ShadowSpec {
                    color: tint(colors.text, 0.10),
                    ..tile.shadow_current
                },
                icon_rest: colors.text_dim,
                icon_current: colors.text,
                ..tile
            }),
            // The counter's shadows are black and white washes and stay;
            // its fill and its text follow the palette.
            counter: kit.collapsible.counter.clone().map(|counter| CapRecipe {
                fill: colors.bg_hover,
                text: colors.text_dim,
                ..counter
            }),
            rail: kit.collapsible.rail.map(|rail| DisclosureRail {
                color: colors.divider,
                link_rest: colors.text_dim,
                link_active: colors.text,
                ..rail
            }),
            body: colors.text_dim,
            ..kit.collapsible
        };
        // The hover border sits between the line and the dim text.
        let between = Color([
            (colors.divider.0[0] + colors.text_dim.0[0]) / 2.0,
            (colors.divider.0[1] + colors.text_dim.0[1]) / 2.0,
            (colors.divider.0[2] + colors.text_dim.0[2]) / 2.0,
            (colors.divider.0[3] + colors.text_dim.0[3]) / 2.0,
        ]);
        let item = ItemRecipe {
            row_hover: wash,
            row_selected: DisclosureSurface {
                fill: colors.bg_hover,
                border: colors.divider,
            },
            icon_color: colors.text_dim,
            label: colors.text,
            meta: colors.text_dim,
            tile_rest: DisclosureSurface {
                fill: colors.bg_hover,
                border: colors.divider,
            },
            tile_hover: DisclosureSurface {
                fill: colors.surface,
                border: between,
            },
            tile_selected: DisclosureSurface {
                fill: colors.surface,
                border: colors.text,
            },
            add_fill: [colors.accent_dim, colors.accent],
            add_glyph_color: colors.bg,
            ..kit.item
        };
        let kbd = kit.kbd.map(|cap| CapRecipe {
            fill: colors.bg_hover,
            text: colors.text_dim,
            ..cap
        });
        let aspect = AspectRecipe {
            rest: DisclosureSurface {
                fill: colors.bg_panel,
                border: colors.divider,
            },
            hover: DisclosureSurface {
                fill: colors.bg_hover,
                border: colors.divider,
            },
            glyph_color: colors.text_dim,
            ..kit.aspect
        };
        let carousel = CarouselRecipe {
            fill: colors.bg_hover,
            glyph_color: colors.text_dim,
            ring_outer: (kit.carousel.ring_outer.0, colors.text_dim),
            ring_inner: (kit.carousel.ring_inner.0, colors.surface),
            counter: colors.text_dim,
            bar_fill: colors.surface,
            bar_border: colors.divider,
            arrow_on: colors.text,
            arrow_off: colors.text_dim,
            tick_on: colors.text,
            tick_off: tint(colors.text, 0.30),
            ..kit.carousel
        };
        let pager = PagerRecipe {
            hover: wash,
            current: DisclosureSurface {
                fill: colors.bg_hover,
                border: colors.divider,
            },
            label: colors.text,
            label_dim: colors.text_dim,
            ..kit.pager
        };
        let (list, table) = Self::rows(theme);
        let tree = Self::tree(theme);
        Self {
            accordion,
            collapsible,
            motion: kit.motion,
            item,
            kbd,
            aspect,
            carousel,
            pager,
            list,
            table,
            tree,
        }
    }

    /// The tree every theme drew before the kit, value for value: body
    /// text rows padded `xs` above and below, `sm` before the first
    /// glyph, an `lg` indent, `Sm` icons an `xs` apart and two before the
    /// label; the glass washes under a row in the nav radius, inset half
    /// a gap and an edge; the label mid, the selected one in the text
    /// tone, the chevron dim, the icon mid.
    fn tree(theme: &Theme) -> TreeRecipe {
        let (colors, glass) = (&theme.colors, &theme.glass);
        let body = theme.typography.base_2r();
        let icon = theme.control.icon(crate::styling::theme::IconSize::Sm);
        let gap = theme.spacing.xs;
        TreeRecipe {
            row_h: (body.line_height + gap * 2.0).round(),
            pad_x: theme.spacing.sm,
            indent: theme.spacing.lg,
            chevron: icon,
            chevron_gap: gap,
            icon,
            tile: None,
            label_gap: gap * 2.0,
            radius: theme.shape.nav,
            border: 0.0,
            seam: [gap / 2.0, theme.control.edge_width],
            hover: DisclosureSurface {
                fill: glass.surface_hover,
                border: NONE,
            },
            selected: DisclosureSurface {
                fill: glass.surface_active,
                border: NONE,
            },
            font_size: body.font_size,
            line_height: body.line_height,
            font_weight: body.font_weight,
            label: colors.text_mid,
            label_selected: colors.text,
            chevron_color: colors.text_dim,
            icon_color: colors.text_mid,
            icon_selected: colors.text_mid,
        }
    }

    /// The rows and the table every theme drew before the kit, value for
    /// value, so the apps on them (urnadbapp's lists, `screen`'s tables)
    /// keep their pixels: the hovered and selected rows in the glass
    /// washes, square, the focus ring in the nav radius; the header band
    /// in the tabs glass and the nav radius, caption titles in the faint
    /// glass text, the divider under it; `md` cell padding, body text,
    /// the lead column in the mid tone, the others faint.
    fn rows(theme: &Theme) -> (RowRecipe, TableRecipe) {
        let (colors, glass, type_) = (&theme.colors, &theme.glass, &theme.typography);
        let list = RowRecipe {
            radius: 0.0,
            border: 0.0,
            hover: DisclosureSurface {
                fill: glass.surface_hover,
                border: NONE,
            },
            selected: DisclosureSurface {
                fill: glass.surface_active,
                border: NONE,
            },
            ring_radius: theme.shape.nav,
            rule: 0.0,
            rule_color: NONE,
        };
        let caption = type_.caption_sm();
        let body = type_.base_2r();
        let table = TableRecipe {
            frame: 0.0,
            frame_color: NONE,
            header_fill: glass.tabs,
            header_radius: theme.shape.nav,
            header_size: caption.font_size,
            header_line_height: caption.line_height,
            header_weight: caption.font_weight,
            header_text: glass.text_faint,
            rule: theme.control.edge_width,
            rule_color: colors.divider,
            cell_pad: theme.spacing.md,
            cell_size: body.font_size,
            cell_line_height: body.line_height,
            cell_weight: body.font_weight,
            cell: colors.text_mid,
            cell_dim: glass.text_faint,
            rows: list,
        };
        (list, table)
    }
}
