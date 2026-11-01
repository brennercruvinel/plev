//! The hoff core kit's values for the front's widgets, as written (the
//! private typescript design system), the file:line of each beside it.

use super::{
    AlertTokens, AttachmentTokens, AvatarTokens, BadgePalette, BadgeTokens, BubbleTokens,
    CommentTokens, CursorShape, IntentColors, KindColors, MarkerTokens, MessageStyle, MessageText,
    MessageTokens, NotificationsTokens, ProgressTokens, PromptTokens, ScrollerTokens,
    SkeletonTokens, SpinnerTokens, TagTokens, ToastTokens,
};
use crate::styling::color::Color;
use crate::styling::theme::ShadowSpec;
use crate::styling::theme::hoff_core::{BLUE, GREEN, RED, SHADES};

impl MessageTokens {
    /// The kit as written (the private typescript design system). The
    /// file:line of each value is in the comment beside it.
    pub fn hoff_core() -> Self {
        let [
            paper,
            shade2,
            shade3,
            shade4,
            shade5,
            shade6,
            shade7,
            _,
            ink,
        ] = SHADES;
        Self {
            // components/Comment/Message/index.tsx
            thread: MessageStyle {
                // :13 `flex gap-4 p-4`
                pad: 16.0,
                gap: 16.0,
                // :16 `size-8 rounded-full`
                avatar: 32.0,
                // :24 `text-[0.75rem] leading-[1rem] font-medium`, the
                // ink of app/layout.tsx:108
                name: MessageText {
                    size: 12.0,
                    line_height: 16.0,
                    weight: 500,
                    color: ink,
                },
                // :26 `ml-2 text-[0.6875rem] text-[#7B7B7B]`, leading
                // and weight inherited from :24
                time: MessageText {
                    size: 11.0,
                    line_height: 16.0,
                    weight: 500,
                    color: shade6,
                },
                time_gap: 8.0,
                // :66 of templates/CommentPage `Luna Starfield ✪`: one
                // space at the name's 12 px, as `ml-1`
                seal_gap: 4.0,
                // :30 `mt-1 text-[0.8125rem] leading-[1.22rem]`, in ink
                body: MessageText {
                    size: 13.0,
                    line_height: 19.52,
                    weight: 400,
                    color: ink,
                },
                body_gap: 4.0,
                body_lines: None,
                // the thread row has no like chip
                reaction_gap: 0.0,
            },
            // components/QuickComment/Comment/index.tsx
            quick: MessageStyle {
                // :29 `p-3`, :30 `gap-3`
                pad: 12.0,
                gap: 12.0,
                // :33 `size-9 rounded-full`
                avatar: 36.0,
                // :42 `text-[0.8125rem] leading-[1rem] font-semibold
                // text-[#121212]`
                name: MessageText {
                    size: 13.0,
                    line_height: 16.0,
                    weight: 600,
                    color: ink,
                },
                // :45 `ml-1 text-[0.75rem] leading-[1rem]`. The kit
                // writes `text-[#7B7B7B]/50`; that is 1.9:1 on paper, so
                // the time keeps the full muted tone (4.13:1) until the
                // contrast decision of task-104.
                time: MessageText {
                    size: 12.0,
                    line_height: 16.0,
                    weight: 400,
                    color: shade6,
                },
                time_gap: 4.0,
                // QuickComment/Comment/index.tsx:74 `Tran Mau Tri Tam ✪`
                seal_gap: 4.0,
                // :49 `mt-1 line-clamp-3 text-[0.8125rem]
                // leading-[1.25rem] text-[#7B7B7B]`
                body: MessageText {
                    size: 13.0,
                    line_height: 20.0,
                    weight: 400,
                    color: shade6,
                },
                body_gap: 4.0,
                body_lines: Some(3),
                // :57 `mt-2` on the like chip
                reaction_gap: 8.0,
            },
            // components/Comment/Message/index.tsx:13 `border-b
            // border-[#ECECEC]`
            divider: shade4,
            divider_width: 1.0,
            // components/Comment/Message/index.tsx:30
            mention: BLUE,
            // components/CommentPin/index.tsx
            bubble: BubbleTokens {
                // the pin pads avatars by `p-0.25`; around text the
                // bubble takes the quick comment's `p-3`
                // (QuickComment/Comment/index.tsx:29) across and half
                // of it down
                pad_x: 12.0,
                pad_y: 6.0,
                // :10 `bg-[#FCFCFC] border-[1.5px]`
                border_width: 1.5,
                fill: SHADES[0],
                // :11 read `border-[#ECECEC]`, unread `border-[#3582FF]`
                border: shade4,
                unread: BLUE,
                // :10 `shadow-[0px_2px_2px_0px_rgba(0,0,0,0.15)]`
                shadow: ShadowSpec::new([0.0, 2.0], 2.0, 0.0, Color::rgba(0.0, 0.0, 0.0, 0.15)),
                // the thread's body run (Comment/Message/index.tsx:30)
                text: MessageText {
                    size: 13.0,
                    line_height: 19.52,
                    weight: 400,
                    color: ink,
                },
                // the kit draws no text bubble; four fifths of the row
                // leaves the other side's bubbles their lane
                max_fraction: 0.8,
            },
            // components/Comment/index.tsx:21 `bg-[#FCFCFC] border
            // border-[#ECECEC] rounded-2xl` and its shadow list
            scroller: ScrollerTokens {
                fill: SHADES[0],
                border: shade4,
                border_width: 1.0,
                radius: 16.0,
                shadow: vec![
                    ShadowSpec::new([0.0, 18.0], 24.0, -20.0, Color::rgba(0.0, 0.0, 0.0, 0.13)),
                    ShadowSpec::inset([0.0, 2.0], 0.0, 0.0, Color::rgba(1.0, 1.0, 1.0, 1.0)),
                    ShadowSpec::new([0.0, 8.0], 16.0, -12.0, Color::rgba(0.0, 0.0, 0.0, 0.08)),
                ],
                // the kit's popup has no cap; this is the select panel's
                // (`size.dropdown_max_h`), the tallest a panel grows
                // before it scrolls
                max_h: 320.0,
                // the kit hides its scrollbars (Notifications:46
                // `scrollbar-none`) and draws no sign that a list goes on;
                // the paper fades over a cut edge two body lines deep
                // (Comment/Message:30 `leading-[1.22rem]`), so the last
                // words before it fade out
                edge: 2.0 * 19.52,
                // the quick comment's muted body (QuickComment/Comment:49)
                empty: MessageText {
                    size: 13.0,
                    line_height: 20.0,
                    weight: 400,
                    color: shade6,
                },
            },
            // components/Notification/index.tsx
            alert: AlertTokens {
                // :18 `p-5 pr-4`
                pad: 20.0,
                pad_end: 16.0,
                // :25 `size-12 rounded-full`
                lead: 48.0,
                // :63 `pl-4`
                gap: 16.0,
                // :32 `-right-1 -bottom-1 size-5.5 border-2
                // border-[#FCFCFC]`
                badge: 22.0,
                badge_ring: 2.0,
                badge_out: 4.0,
                // :45 `size-3`
                badge_icon: 12.0,
                ring: paper,
                // :33-41, the badge per kind of notification, read as
                // intents: the comment's violet the neutral one, the
                // video's blue the informational, the invite's green the
                // constructive, the like's red the destructive
                intents: IntentColors {
                    neutral: Color::hex(0x8651F8),
                    informational: Color::hex(0x1BA4F3),
                    constructive: Color::hex(0x55B93E),
                    destructive: RED,
                },
                // :33-41 per kind: image `#AD9000`, invite `#55b93e`, like
                // `#fe5938`, video `#1BA4F3`, comment `#8651F8`
                kinds: KindColors {
                    comment: Color::hex(0x8651F8),
                    video: Color::hex(0x1BA4F3),
                    invite: Color::hex(0x55B93E),
                    like: RED,
                    image: Color::hex(0xAD9000),
                },
                // :65 `text-[0.8125rem] leading-[1rem] font-semibold`
                title: MessageText {
                    size: 13.0,
                    line_height: 16.0,
                    weight: 600,
                    color: ink,
                },
                // :64 `-mt-0.5 mb-1.5`
                title_lift: 2.0,
                title_gap: 6.0,
                // :68 `ml-1 text-[0.75rem] leading-[1rem] font-medium
                // text-[#7B7B7B]/50`. That is 1.9:1 on paper; the time
                // takes shade 7, the darkest gray under ink that still
                // reads as secondary, and passes AAA (task-104 contrast)
                time: MessageText {
                    size: 12.0,
                    line_height: 16.0,
                    weight: 500,
                    color: shade7,
                },
                time_gap: 4.0,
                // :75 `truncate text-[0.8125rem] leading-[1rem]`
                detail: MessageText {
                    size: 13.0,
                    line_height: 16.0,
                    weight: 400,
                    color: ink,
                },
                // :99 `mt-2 line-clamp-2 text-[0.8125rem]
                // leading-[1.2188rem] text-[#7B7B7B]/80`; shade 7 for
                // AAA, as the time
                content: MessageText {
                    size: 13.0,
                    line_height: 19.5,
                    weight: 400,
                    color: shade7,
                },
                content_gap: 8.0,
                content_lines: 2,
                // :104 `mt-4 gap-2`
                actions_gap: 16.0,
                action_gap: 8.0,
                // :72 `top-2.5 size-2`, in the title line's `pr-4`. The
                // kit's green dot (#55b93e) is 2.4:1 on paper, under the
                // 3:1 a state mark needs; the dot takes the kit's unread
                // accent, the comment pin's blue
                dot: 8.0,
                dot_top: 10.0,
                dot_room: 16.0,
                unread: BLUE,
            },
            // components/NewComment/ImagePreview/index.tsx
            attachment: AttachmentTokens {
                // :13 `size-15 rounded-xl`, :9 `gap-2`
                tile: 60.0,
                radius: 12.0,
                gap: 8.0,
                // the add-files row's hover wash
                // (PromptInput/AddFiles/index.tsx:53 `hover:bg-[#f1f1f1]`)
                placeholder: shade3,
                // :20 `-top-1.5 -right-1.5 size-5 bg-gradient-to-b
                // from-[#323232] to-[#121212]`, :22 `size-4 fill-white`
                remove: 20.0,
                remove_out: 6.0,
                remove_icon: 16.0,
                remove_fill: [shade7, ink],
                remove_ink: Color::WHITE,
                // PromptInput/AddFiles/index.tsx:53 `gap-2.5 px-2`,
                // :58 `size-6`, :60 `size-5 fill-[#7b7b7b]`; the tile
                // pads by the image preview's `p-3` cousin, `px-3`
                file_pad: 12.0,
                file_icon: 20.0,
                file_icon_box: 24.0,
                file_gap: 10.0,
                file_icon_color: shade6,
                // Upload/index.tsx:21 `bg-[#FCFCFC] border
                // border-[#E2E2E2]`
                file_border: shade5,
                file_border_width: 1.0,
                file_fill: paper,
                // :53 `text-[0.8125rem] font-medium text-[#121212]`, at
                // the kit's 16 line
                name: MessageText {
                    size: 13.0,
                    line_height: 16.0,
                    weight: 500,
                    color: ink,
                },
                // the notification's time run (Notification:68), in
                // shade 7 for AAA as there
                detail: MessageText {
                    size: 12.0,
                    line_height: 16.0,
                    weight: 500,
                    color: shade7,
                },
                // the kit draws no file tile: it grows with its name up
                // to the tray's width, where the name is ellipsized
                file_max_fraction: 1.0,
            },
            marker: MarkerTokens {
                // components/CommentPin/index.tsx:10 `p-0.25
                // border-[1.5px] bg-[#FCFCFC]
                // shadow-[0px_2px_2px_0px_rgba(0,0,0,0.15)]`, :11 read
                // `#ECECEC`, unread `#3582FF`
                pin_pad: 1.0,
                pin_border_width: 1.5,
                pin_fill: paper,
                pin_read: shade4,
                pin_unread: BLUE,
                pin_shadow: ShadowSpec::new([0.0, 2.0], 2.0, 0.0, Color::rgba(0.0, 0.0, 0.0, 0.15)),
                // :16 `-ml-2.5 border-[1.5px] border-[#FCFCFC]`, :20
                // `size-6`
                avatar: 24.0,
                avatar_ring: 1.5,
                avatar_overlap: 10.0,
                // components/CommentCursor/index.tsx:12 `w-5 h-4.5
                // border-[1.5px] border-[#121212] rounded-md
                // rounded-bl-[1px] bg-[#FCFCFC]
                // shadow-[0px_2px_4px_rgba(0,0,0,0.15)]`, :28 `size-1.75
                // fill-[#7B7B7B]/50`. That plus is 1.9:1 on paper; it
                // takes the full muted gray, 4.1:1
                point: CursorShape {
                    w: 20.0,
                    h: 18.0,
                    radius: 6.0,
                    tail_radius: 1.0,
                    shadow: ShadowSpec::new([0.0, 2.0], 4.0, 0.0, Color::rgba(0.0, 0.0, 0.0, 0.15)),
                    fill: [paper, paper],
                    mark: 7.0,
                    ink: shade6,
                },
                point_border_width: 1.5,
                point_border: ink,
                // :13 `w-7.5 h-7 rounded-[10px] rounded-bl-[2px]
                // shadow-[0px_4px_4px_rgba(18,18,18,0.15)]`, :14
                // `from-[#323232] to-[#121212]`, :29 `size-2.25
                // fill-[#7B7B7B]`
                pinned: CursorShape {
                    w: 30.0,
                    h: 28.0,
                    radius: 10.0,
                    tail_radius: 2.0,
                    shadow: ShadowSpec::new(
                        [0.0, 4.0],
                        4.0,
                        0.0,
                        Color::rgba(18.0 / 255.0, 18.0 / 255.0, 18.0 / 255.0, 0.15),
                    ),
                    fill: [shade7, ink],
                    mark: 9.0,
                    ink: shade6,
                },
                // :16 `from-[#34A7FF] to-[#0D7DFD]`, :30 `fill-[#FFFFFF]`.
                // White is 2.6:1 on #34A7FF; the top takes the kit's
                // accent blue, 3.6:1, the bottom stays
                active: CursorShape {
                    w: 30.0,
                    h: 28.0,
                    radius: 10.0,
                    tail_radius: 2.0,
                    shadow: ShadowSpec::new(
                        [0.0, 4.0],
                        4.0,
                        0.0,
                        Color::rgba(18.0 / 255.0, 18.0 / 255.0, 18.0 / 255.0, 0.15),
                    ),
                    fill: [BLUE, Color::hex(0x0D7DFD)],
                    mark: 9.0,
                    ink: Color::WHITE,
                },
                // :21 `gap-0.5 [&_span]:size-0.5`
                dot: 2.0,
                dot_gap: 2.0,
                // :36, the plus path: bars 1.5 wide on an 8 box
                mark_bar: 1.5 / 8.0,
            },
            // templates/NotificationsPage/index.tsx:115 `bg-[#FCFCFC]
            // border border-[#ECECEC] rounded-[1.25rem] shadow-2xl`
            // (tailwind: 0 25px 50px -12px black/25)
            toast: ToastTokens {
                radius: 20.0,
                fill: paper,
                border: shade4,
                border_width: 1.0,
                shadow: ShadowSpec::new([0.0, 25.0], 50.0, -12.0, Color::rgba(0.0, 0.0, 0.0, 0.25)),
            },
            badge: BadgeTokens::new(
                // components/Card/index.tsx:56 `px-3 py-1 border
                // rounded-xl text-[0.6875rem] leading-[1rem] font-medium`,
                // :58 `bg-[#F1F1F1] border-transparent`, on a gray card :57
                // `bg-[#FCFCFC] border-[#ECECEC]`; the ink of
                // app/layout.tsx:108
                TagTokens {
                    pad_x: 12.0,
                    pad_y: 4.0,
                    radius: 12.0,
                    border_width: 1.0,
                    text: MessageText {
                        size: 11.0,
                        line_height: 16.0,
                        weight: 500,
                        color: ink,
                    },
                    fill: shade3,
                    // the kit's hairline is transparent on a white card;
                    // the chip takes its gray-card #ECECEC, walked to 3:1
                    // on the paper and on the well (`BadgeTokens::new`)
                    border: shade4,
                    raised_fill: paper,
                    raised_border: shade4,
                    // components/Notification/index.tsx:72 `size-2
                    // rounded-full`, the new-item disc; :68 `ml-1`
                    mark: 8.0,
                    mark_gap: 4.0,
                },
                // the kit draws no count: two of the chip's `py-1` across,
                // so one digit sits in a disc
                8.0,
                // components/HeaderSimple/index.tsx:80 `after:top-1
                // after:right-1 after:size-1 after:bg-[#fe5938]
                // after:rounded-full`
                4.0,
                4.0,
                // the intents in the core's accents (the dot's red, the
                // mention blue of Comment/Message:30, its green), the
                // neutral in the kit's muted gray; each moves toward the
                // ink until it reads (`BadgeTokens::new`)
                BadgePalette {
                    intents: IntentColors {
                        neutral: shade6,
                        informational: BLUE,
                        constructive: GREEN,
                        destructive: RED,
                    },
                    surface: paper,
                    // components/Card/index.tsx:52 `bg-[#F8F7F7]`, the
                    // gray card a chip sits in
                    well: shade2,
                    paper,
                    ink,
                },
            ),
            avatar: AvatarTokens {
                // the kit's avatars: CommentPin:20 `size-6`,
                // Comment/Message:16 `size-8`, HeaderSimple:94 `size-10`
                // (the control `Md` step, the basis), UploadAvatar:8
                // `size-12`, ShareProfile:44 `size-20`
                ladder: [24.0, 32.0, 40.0, 48.0, 80.0],
                basis: 40.0,
                // UploadAvatar:8 `bg-[#F8F7F7]`, the disc a photo would
                // fill; without one the initial stands on it in the ink,
                // and the hairline of Comment/Message:13
                // `border-[#ECECEC]` closes it on the paper
                fill: shade2,
                edge: shade4,
                edge_width: 1.0,
                initial: ink,
                // DesignAndAnimationSidebar/Head:11 `-ml-2 size-8 border-2
                // border-[#fcfcfc]`: a 2 px paper ring, 8 of 32 under the
                // one before
                stack_ring: 2.0,
                stack_overlap: 8.0 / 32.0,
                // components/Notification/index.tsx:72 `size-2
                // rounded-full bg-[#55b93e]`, the green mark; :31
                // `border-2 border-[#FCFCFC]`, the paper ring a mark on an
                // avatar takes. #55B93E is 2.4:1 on the paper; it moves
                // toward the ink until it reads (`AvatarTokens::legible`)
                online: 8.0,
                online_ring: 2.0,
                online_fill: Color::hex(0x55B93E),
                ring: paper,
            }
            .legible(),
            // components/SceneCardSkeleton/index.tsx
            skeleton: SkeletonTokens {
                // :3 `mb-2 aspect-[1.309] bg-[#F8F7F7] rounded-2xl`; it
                // breathes toward the lines' #F1F1F1
                block: shade2,
                block_pulse: shade3,
                block_radius: 16.0,
                // :6 `h-2 bg-[#F1F1F1] rounded-[2px]`, breathing toward
                // the hairline's #ECECEC; :5 `mb-1 p-1` around each bar;
                // :9 `w-18` after :6 `w-32`, the last line 72 of 128
                line: 8.0,
                line_radius: 2.0,
                line_fill: shade3,
                line_pulse: shade4,
                line_pad: 4.0,
                line_gap: 4.0,
                last_line: 72.0 / 128.0,
                // :2 `w-65 p-2 border border-[#ECECEC] bg-[#FCFCFC]
                // rounded-3xl`
                card_w: 260.0,
                card_fill: paper,
                card_border: shade4,
                card_border_width: 1.0,
                card_radius: 24.0,
                card_pad: 8.0,
                image_aspect: 1.309,
                image_gap: 8.0,
                // :4 `p-3`
                text_pad: 12.0,
                // :6 `w-32`, :9 `w-18` in a column of 260 - 2 (border) -
                // 16 (`p-2`) - 24 (`p-3`) - 8 (`p-1`) = 210
                card_lines: [128.0 / 210.0, 72.0 / 210.0],
            },
            // components/VideoPlayer/index.tsx:48-53, the rail on paper:
            // :49 `min-w-45 h-0.75 rounded-full bg-[#7B7B7B]/20`, :53
            // `bg-[#7B7B7B]/30`, its head `after:w-1.25
            // after:rounded-full after:bg-[#121212]`. The kit's one rail
            // is neutral; the intents tint it with the core's accents (as
            // the badge) and their heads move toward the ink until they
            // read (`ProgressTokens::new`)
            progress: ProgressTokens::new(
                3.0,
                alpha(shade6, 0.2),
                IntentColors {
                    neutral: alpha(shade6, 0.3),
                    informational: alpha(BLUE, 0.3),
                    constructive: alpha(GREEN, 0.3),
                    destructive: alpha(RED, 0.3),
                },
                5.0,
                IntentColors {
                    neutral: ink,
                    informational: BLUE,
                    constructive: GREEN,
                    destructive: RED,
                },
                180.0,
                paper,
                ink,
            ),
            // the kit draws no spinner: the rail of VideoPlayer:49 wound
            // into a ring, its 3 px and its #7B7B7B/20 track, the arc in
            // the rail's head #121212; a quarter of the ring, so the track
            // still shows around it
            spinner: SpinnerTokens {
                stroke: 3.0,
                track: alpha(shade6, 0.2),
                arc: ink,
                sweep: 0.25,
            },
            prompt: PromptTokens::hoff_core(),
            comment: CommentTokens::hoff_core(),
            notifications: NotificationsTokens::hoff_core(),
        }
    }
}

/// `c` at the css alpha `a` (`bg-[#7B7B7B]/20`).
fn alpha(c: Color, a: f32) -> Color {
    Color([c.0[0], c.0[1], c.0[2], a])
}
