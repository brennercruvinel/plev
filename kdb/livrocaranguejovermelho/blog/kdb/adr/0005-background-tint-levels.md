# 0005. background tint levels and the readability tradeoff

- status: accepted
- date: 2026-06-23

## context

two complaints, opposite directions. light backgrounds derived at 20% accent looked like a heavy pastel and hurt the reading of text, links, code, and tables. dark backgrounds derived at 10% were almost pure black, so every palette looked the same in dark mode.

## decision

light tint dropped to 6%, so the page is near-white and the accent carries the identity through links and titles instead of flooding the background. dark tint raised from 10% to 15%, enough to show the palette hue while staying dark enough for white body text at AA. secondary text was darkened to clear AA on the new backgrounds: muted-4 from 0.5 to 0.56 in light, 0.5 to 0.6 in dark; muted-5 to 0.66 and 0.7.

all of it was run through the contrast script from adr 0002 before shipping.

## consequences

- light mode reads cleanly, no more washed pastel.
- dark mode shows color instead of flat black.
- the tint lives in `_variables.scss`, so it is global. it also shifts the original orange, which i decided looks cleaner anyway, not worse.
