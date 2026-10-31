//! The kit's values, class list by class list (`hoff_core`).

use super::*;
use crate::styling::theme::hoff_core::SHADES;

impl DisclosureTokens {
    /// The kit, class list by class list.
    pub fn hoff_core() -> Self {
        let ink = hex(0x121212);
        let gray = hex(0x7B7B7B);
        let line = hex(0xECECEC);
        let wash = hex(0xF1F1F1);
        let paper = hex(0xFCFCFC);

        // Accordion/index.tsx: the section `border-t border-[#ECECEC]
        // first:border-t-0` (:22); the header `px-4 py-3 text-[0.75rem]
        // font-semibold text-[#121212]` (:27), no fill in any state; the
        // chevron `size-4 fill-[#7B7B7B]` in a `size-6` box, pushed to the
        // end by `justify-between` (:31-33); the body `p-4 pt-0` (:45).
        // Line height 18: the 0.75rem label inherits the preflight's 1.5.
        let accordion = DisclosureRecipe {
            pad: [12.0, 16.0, 12.0, 16.0],
            radius: 0.0,
            border_width: 0.0,
            rest: CLEAR,
            hover: CLEAR,
            open: CLEAR,
            current: CLEAR,
            font_size: 12.0,
            line_height: 18.0,
            font_weight: 600,
            label: ink,
            chevron_box: 24.0,
            chevron: 16.0,
            chevron_gap: 0.0,
            chevron_color: gray,
            tile: None,
            counter: None,
            rail: None,
            // The grid: `flex flex-wrap -mt-2 -mx-1`, each thumbnail
            // `w-[calc(50%-0.5rem)] mx-1 mt-2 aspect-square p-1.5
            // bg-[#F1F1F1] rounded-2xl`, hovered `shadow-[0_0_0_1.5px
            // rgba(123,123,123,0.5)_inset,0px_0px_0px_4px_#fcfcfc_inset]`,
            // its image `size-16` or `size-22` (:46-55).
            grid: Some(GridRecipe {
                columns: 2,
                gap: 8.0,
                pad: 6.0,
                radius: 16.0,
                fill: wash,
                ring_outer: (1.5, tint(gray, 0.5)),
                ring_inner: (4.0, paper),
                image: 64.0,
                image_large: 88.0,
                glyph_color: gray,
            }),
            divider: line,
            divider_width: 1.0,
            content_pad: [0.0, 16.0, 16.0, 16.0],
            // The kit's sections hold thumbnails, not text: a text body
            // takes the Dropdown's list text (Dropdown :73-76).
            body_size: 12.0,
            body_line_height: 18.0,
            body_weight: 600,
            body: gray,
        };

        // The counter `px-1.5 py-0.5 bg-[#f1f1f1] rounded-md
        // text-[0.6875rem] font-medium text-[#7b7b7b]` and its shadow
        // `0 0 0 1px black/11, inset 0 2px 0.8px white/27, inset 0 -1px
        // 0.6px black/20, 0 1px 4.2px -1px black/25` (:52).
        let cap = CapRecipe {
            pad: [6.0, 2.0],
            radius: 6.0,
            fill: wash,
            shadow: vec![
                ShadowSpec::new([0.0, 0.0], 0.0, 1.0, black(0.11)),
                ShadowSpec::inset([0.0, 2.0], 0.8, 0.0, white(0.27)),
                ShadowSpec::inset([0.0, -1.0], 0.6, 0.0, black(0.20)),
                ShadowSpec::new([0.0, 1.0], 4.2, -1.0, black(0.25)),
            ],
            font_size: 11.0,
            line_height: 16.5,
            font_weight: 500,
            text: gray,
        };

        // Dropdown/index.tsx: the trigger `p-0.75 pr-3 border rounded-xl
        // text-[0.75rem] font-semibold text-[#121212] hover:bg-[#f1f1f1]`,
        // filled while open, `bg-[#f1f1f1] border-[#ececec]` while
        // current, `border-transparent` otherwise (:25-30); the tile
        // `size-8 mr-3 rounded-lg`, raised `bg-[#fcfcfc]
        // shadow-[0_0_4px_0_rgba(18,18,18,0.10)] fill-[#121212]` while
        // current, `fill-[#7b7b7b]` otherwise, its icon `size-5` (:35-42);
        // the chevron `size-4 ml-3 fill-[#7b7b7b]` (:58); the list
        // `pt-0.5` (:70), its items `text-[0.75rem] font-semibold
        // text-[#7b7b7b]` (:73-76).
        let collapsible = DisclosureRecipe {
            pad: [3.0, 12.0, 3.0, 3.0],
            radius: 12.0,
            border_width: 1.0,
            rest: CLEAR,
            hover: DisclosureSurface {
                fill: wash,
                border: NONE,
            },
            open: DisclosureSurface {
                fill: wash,
                border: NONE,
            },
            current: DisclosureSurface {
                fill: wash,
                border: line,
            },
            font_size: 12.0,
            line_height: 18.0,
            font_weight: 600,
            label: ink,
            chevron_box: 16.0,
            chevron: 16.0,
            chevron_gap: 12.0,
            chevron_color: gray,
            tile: Some(DisclosureTile {
                size: 32.0,
                radius: 8.0,
                gap: 12.0,
                icon: 20.0,
                fill_current: paper,
                shadow_current: ShadowSpec::new([0.0, 0.0], 4.0, 0.0, tint(ink, 0.10)),
                icon_rest: gray,
                icon_current: ink,
            }),
            counter: Some(cap.clone()),
            grid: None,
            // The list: the rail `before:left-4.75 before:bottom-8
            // before:w-[1.5px] before:bg-[#ececec]` (:70); each link `h-10
            // pl-12 pr-3`, `text-[#121212]` when current or hovered,
            // `text-[#7b7b7b]` otherwise, its branch `before:left-4.75
            // before:bottom-5 before:w-4 before:border-l-[1.5px]
            // before:border-b-[1.5px] before:rounded-bl-lg` (:73), the
            // current one's chevron `size-4 -rotate-90` (:83).
            rail: Some(DisclosureRail {
                x: 19.0,
                width: 1.5,
                color: line,
                end: 32.0,
                branch_w: 16.0,
                branch_h: 20.0,
                branch_radius: 8.0,
                link_h: 40.0,
                link_pad: [48.0, 12.0],
                link_rest: gray,
                link_active: ink,
                link_chevron: 16.0,
            }),
            divider: NONE,
            divider_width: 0.0,
            content_pad: [2.0, 0.0, 0.0, 0.0],
            body_size: 12.0,
            body_line_height: 18.0,
            body_weight: 600,
            body: gray,
        };

        // Folder/index.tsx: `p-0.75 rounded-xl text-[0.75rem]
        // font-semibold border border-transparent hover:bg-[#f1f1f1]`, the
        // icon in a `size-8 mr-3` box (:23-24); selected takes the
        // Dropdown's current look (Dropdown :28). AssetItem/index.tsx: the
        // tile `size-15 rounded-[0.625rem] bg-[#F1F1F1] border
        // border-[#ECECEC] hover:border-[#C4C4C4] hover:bg-[#fcfcfc]`, its
        // image `size-11` (:12-19); the add button `size-4 top-0.5
        // right-0.5 rounded-full from-[#323232] to-[#121212]`, its glyph
        // `size-3 fill-white` (:22-24).
        let item = ItemRecipe {
            row_pad: 3.0,
            row_radius: 12.0,
            row_border: 1.0,
            row_rest: CLEAR,
            row_hover: DisclosureSurface {
                fill: wash,
                border: NONE,
            },
            row_selected: DisclosureSurface {
                fill: wash,
                border: line,
            },
            icon_box: 32.0,
            icon: 20.0,
            gap: 12.0,
            icon_color: gray,
            font_size: 12.0,
            line_height: 18.0,
            font_weight: 600,
            label: ink,
            meta_size: 11.0,
            meta_line_height: 16.5,
            meta_weight: 500,
            meta: gray,
            tile: 60.0,
            tile_radius: 10.0,
            tile_border: 1.0,
            tile_rest: DisclosureSurface {
                fill: wash,
                border: line,
            },
            tile_hover: DisclosureSurface {
                fill: paper,
                border: hex(0xC4C4C4),
            },
            tile_selected: DisclosureSurface {
                fill: paper,
                border: ink,
            },
            thumb: 44.0,
            add: 16.0,
            add_inset: 2.0,
            add_fill: [hex(0x323232), ink],
            add_glyph: 12.0,
            add_glyph_color: hex(0xFFFFFF),
        };

        // The key caps: FileMenu/index.tsx's `text-[0.6875rem]
        // leading-[1rem]` (:42) and Search/index.tsx's `text-[0.75rem]
        // leading-[1rem]` (:42), on the counter's cap.
        let kbd = [
            CapRecipe {
                line_height: 16.0,
                ..cap.clone()
            },
            CapRecipe {
                font_size: 12.0,
                line_height: 16.0,
                ..cap
            },
        ];

        // AssetCard/index.tsx: `aspect-square p-6 bg-[#F8F7F7] border
        // border-[#ECECEC] rounded-[1.25rem] hover:bg-[#F1F1F1]` (:10);
        // the shade `h-24 bg-gradient-to-t from-[#171717]/100
        // group-hover:opacity-30` (:20). The image fills the well: a
        // placeholder glyph takes half of it, in the dim tone.
        let aspect = AspectRecipe {
            pad: 24.0,
            radius: 20.0,
            border: 1.0,
            rest: DisclosureSurface {
                fill: hex(0xF8F7F7),
                border: line,
            },
            hover: DisclosureSurface {
                fill: wash,
                border: line,
            },
            glyph: 0.5,
            glyph_color: gray,
            shade_h: 96.0,
            shade: tint(hex(0x171717), 0.30),
        };

        // Variations/index.tsx: the counter `text-[0.75rem]
        // leading-[1.125rem] text-[#7B7B7B]` under `mb-3` (:11-17); the
        // tiles `w-[calc(50%-0.5rem)] mx-1 mt-2 rounded-2xl` in a `w-52
        // p-4` panel (an 84 px edge), ringed `0_0_0_1.5px_#7b7b7b_inset,
        // 0px_0px_0px_3px_#fcfcfc_inset` hovered or chosen (:30-40).
        // ViewController/index.tsx: the bar `h-11 px-5 gap-3 bg-[#fcfcfc]
        // border border-[#ececec] rounded-full` and its shadow (:8; the
        // kit's third layer reads `0_1px_1p_ 0`, taken as `0 1px 1px 0`),
        // `mb-4` above the slides; the arrows `size-4`, `fill-[#7b7b7b]`
        // at the start, ink otherwise; the ticks `w-[0.09375rem] h-2
        // bg-[#121212]/30`, `h-3.5 bg-[#121212]` current, `gap-2`
        // (:14-60).
        let carousel = CarouselRecipe {
            slide_min: 84.0,
            gap: 8.0,
            radius: 16.0,
            fill: wash,
            glyph: 0.5,
            glyph_color: gray,
            ring_outer: (1.5, gray),
            ring_inner: (3.0, paper),
            counter_size: 12.0,
            counter_line_height: 18.0,
            counter: gray,
            counter_gap: 12.0,
            bar_h: 44.0,
            bar_pad: 20.0,
            bar_gap: 12.0,
            bar_fill: paper,
            bar_border: line,
            bar_shadow: vec![
                ShadowSpec::new([0.0, 6.0], 3.0, 0.0, black(0.01)),
                ShadowSpec::new([0.0, 3.0], 3.0, 0.0, black(0.02)),
                ShadowSpec::new([0.0, 1.0], 1.0, 0.0, black(0.02)),
            ],
            bar_space: 16.0,
            arrow: 16.0,
            arrow_on: ink,
            arrow_off: gray,
            tick_w: 1.5,
            tick_h: 8.0,
            tick_h_on: 14.0,
            tick_gap: 8.0,
            tick_on: ink,
            tick_off: tint(ink, 0.30),
        };
        let pager = PagerRecipe {
            cell: 32.0,
            radius: 8.0,
            gap: 4.0,
            border: 1.0,
            rest: CLEAR,
            hover: DisclosureSurface {
                fill: wash,
                border: NONE,
            },
            current: DisclosureSurface {
                fill: wash,
                border: line,
            },
            font_size: 12.0,
            line_height: 18.0,
            font_weight: 600,
            label: ink,
            label_dim: gray,
            arrow: 16.0,
        };

        // A list's rows are the Folder row (Folder/index.tsx:23: `rounded-xl
        // border border-transparent hover:bg-[#f1f1f1]`), selected in the
        // Dropdown's current look (Dropdown :28), as the Item's row.
        let list = RowRecipe {
            radius: item.row_radius,
            border: item.row_border,
            hover: item.row_hover,
            selected: item.row_selected,
            ring_radius: item.row_radius,
            rule: 0.0,
            rule_color: NONE,
        };

        // The kit's one table, the typography page's spec sheet
        // (templates/TypographyPage/index.tsx:8): `border border-[#ECECEC]
        // text-[0.6875rem] leading-[1rem] font-medium text-[#7B7B7B]`,
        // cells `p-5`, the header `text-left font-medium opacity-50` on no
        // fill, every data cell `border-t border-[#ECECEC]`, the first
        // `text-[#121212]`. Its gray fails the AAA contrast the tables
        // meet (#7B7B7B is 4.13 on paper, at opacity 50 far less): the
        // header and the other cells take shade 7, the theme's darkest
        // gray that passes (`theme/tests/contrast.rs`). The kit has no
        // hover nor selection on it: a row takes the AssetCard well's two
        // washes (AssetCard/index.tsx:10, `bg-[#F8F7F7]
        // hover:bg-[#F1F1F1]`), the paler hovered, the deeper selected,
        // square as the table's rules. Heights stay the controls' (one
        // `Lg` row, one `Xs` header), so the geometry of every table is
        // the same in every theme.
        let passing = SHADES[6];
        let table = TableRecipe {
            frame: 1.0,
            frame_color: line,
            header_fill: NONE,
            header_radius: 0.0,
            header_size: 11.0,
            header_line_height: 16.0,
            header_weight: 500,
            header_text: passing,
            rule: 1.0,
            rule_color: line,
            cell_pad: 20.0,
            cell_size: 11.0,
            cell_line_height: 16.0,
            cell_weight: 500,
            cell: ink,
            cell_dim: passing,
            rows: RowRecipe {
                radius: 0.0,
                border: 0.0,
                hover: DisclosureSurface {
                    fill: hex(0xF8F7F7),
                    border: NONE,
                },
                selected: DisclosureSurface {
                    fill: wash,
                    border: NONE,
                },
                ring_radius: 0.0,
                rule: 1.0,
                rule_color: line,
            },
        };

        // The kit has no tree; its nested rows are the scene list's
        // (SceneAndAssetsSidebar/Scene/Item/index.tsx:45-64): `gap-3 p-0.75
        // border rounded-xl hover:bg-[#f1f1f1]`, selected `bg-[#f1f1f1]
        // border-[#e2e2e2]`; the tile `size-8 rounded-lg bg-[#f1f1f1]`,
        // raised `bg-[#fcfcfc] shadow-[0_0px_4px_0px_rgba(18,18,18,0.10)]`
        // when selected, its icon `size-4`; the label `text-[0.75rem]
        // font-medium text-[#121212]` (:82). A row is 40 px: the tile, the
        // padding and the border on each side. The chevron is the
        // Accordion's (`size-4 fill-[#7B7B7B]`), a `gap-3` before the
        // tile; a level indents by the chevron and its gap, so a child's
        // chevron sits under its parent's tile.
        let pad = 3.0 + item.row_border;
        let tile = 32.0;
        let tree = TreeRecipe {
            row_h: tile + pad * 2.0,
            pad_x: pad,
            indent: 16.0 + 12.0,
            chevron: 16.0,
            chevron_gap: 12.0,
            icon: 16.0,
            tile: Some(TreeTile {
                size: tile,
                radius: 8.0,
                rest: wash,
                selected: paper,
                shadow_selected: ShadowSpec::new([0.0, 0.0], 4.0, 0.0, tint(ink, 0.10)),
            }),
            label_gap: 12.0,
            radius: 12.0,
            border: item.row_border,
            seam: [0.0, 0.0],
            hover: DisclosureSurface {
                fill: wash,
                border: NONE,
            },
            selected: DisclosureSurface {
                fill: wash,
                border: hex(0xE2E2E2),
            },
            font_size: 12.0,
            line_height: 18.0,
            font_weight: 500,
            label: ink,
            label_selected: ink,
            chevron_color: gray,
            icon_color: ink,
            icon_selected: ink,
        };

        // The kit's panel, the inspector's (DesignAndAnimationSidebar/
        // index.tsx:23, `border border-[#ECECEC] rounded-[1.25rem]`), its
        // sections `px-4` (:25); a group's label `text-[0.75rem]
        // leading-[1rem] font-medium text-[#7b7b7b]/70` (GuidelineSidebar/
        // index.tsx:59), whose gray fails AAA and takes shade 7.
        let panel = PanelRecipe {
            radius: 20.0,
            pad: 16.0,
            label_size: 12.0,
            label_line_height: 16.0,
            label_weight: 500,
            label: passing,
        };
        // The Notifications head (Notifications/index.tsx:26-28): `px-5 py-4
        // border-b border-[#ECECEC]`, the title `text-[0.875rem]
        // leading-[1.25rem] font-semibold`; a second line in the
        // notification's `text-[0.75rem] leading-[1rem] font-medium
        // text-[#7B7B7B]/50` (Notification/index.tsx:68), in shade 7 for
        // AAA, right under the title's row, whose `py-4` parts them, and
        // `py-4` below it; the tabs `gap-1` (:30) before the badge.
        let header = HeaderRecipe {
            pad_x: 20.0,
            row: 16.0 + 20.0 + 16.0,
            blurb_gap: 0.0,
            blurb_bottom: 16.0,
            title_size: 14.0,
            title_line_height: 20.0,
            title_weight: 600,
            title: ink,
            blurb_size: 12.0,
            blurb_line_height: 16.0,
            blurb_weight: 500,
            blurb: passing,
            badge_gap: 4.0,
            rule: 1.0,
            rule_color: line,
        };

        Self {
            accordion,
            collapsible,
            motion: DisclosureMotion::hoff_core(),
            item,
            kbd,
            aspect,
            carousel,
            pager,
            list,
            table,
            tree,
            panel,
            header,
        }
    }
}
