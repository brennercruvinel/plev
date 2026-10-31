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
        Self {
            accordion,
            collapsible,
            motion: kit.motion,
            item,
            kbd,
            aspect,
            carousel,
            pager,
        }
    }
}
