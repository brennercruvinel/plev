---
type: adr
status: proposed
tags: [fonts, text, noto, cosmic-text, fontdb, fallback, subset, wasm, mobile, determinism, cjk, arabic, hebrew, devanagari, generative-runtime]
date: 2026-09-25
---

# embed Noto Sans subsets for the non-Latin scripts

## context

the audit of task-74 phase 1 (`docs/mission/steps/ongoing/task-74-text-and-editing.md`,
"relatório da fase 1") measured that the embedded path, what wasm,
android and ios ship, shaped every non-Latin script as tofu: 24 of 27
glyphs in the Arabic line, 5 of 5 in the Chinese one. the desktop hid it
behind the system fonts, a different face per script and per machine,
and not every one of them (fontdb does not find PingFang on macOS 26). a
tofu has its own advance, so the layout was wrong before any pixel. the
choice left for the user was between embedding subsets (size),
system fallback (no fonts at all on wasm, and a different product per
machine) or both behind a setting.

## decision

brenner, 2026-09-25: "Noto, amigo. Noto, sans". the engine embeds Noto
Sans for six scripts, Regular and Bold: Arabic, Hebrew, Devanagari,
Simplified Chinese (SC), Japanese (JP) and Korean (KR). emoji is not part
of it (see below).

### the subsets

`script/font-subset` (python, stdlib plus fontTools 4.64.0, no network)
reads the sources from outside the repo (`target/fonts-noto/`), checks
each one's sha256 against the value in the script (with the URL it came
from: the hinted TTFs of notofonts.github.io and `Sans/SubsetOTF` of
noto-cjk 2.004), and writes the subsets to `crates/engine/assets/fonts/`
with `LICENSE-Noto-OFL.txt` (the copyright and trademark records of every
source, then the OFL 1.1 as shipped). a second run writes the same bytes
(`--check` rebuilds into a temporary directory and compares).

- Arabic, Hebrew, Devanagari: every code point of the source, every layout
  feature, the hinting.
- SC: GB2312 level 1, 3,755 hanzi. JP: JIS X 0208 kana (rows 4 and 5) and
  level 1, 2,965 kanji. KR: the 2,350 hangul syllables of KS X 1001. each
  adds row 1 (punctuation and symbols) and row 3 (full-width digits and
  letters) of its own standard. the sets come from the stdlib codecs
  (`gb2312`, `euc_jp`, `euc_kr`), so they are the same on every machine.
- the CJK subsets keep GSUB, GPOS and GDEF with fontTools' default feature
  list (what a shaper applies on its own, jamo forms included) minus the
  vertical features, and drop the vertical metrics (`vhea`, `vmtx`,
  `VORG`): the engine sets no vertical text. what goes is the alternates
  only an optional feature reaches (`aalt`, `jp78`, `jp83`, `jp90`,
  `nlck`, `fwid`, `hwid`, `pwid`, `ruby`, `hist`, `dlig`); JP drops from
  1,032,976 to 794,888 bytes with it.

| file | source bytes | subset bytes | gzip -9 | code points |
|---|---|---|---|---|
| NotoSansArabic-Regular.ttf | 234,892 | 217,320 | 89,922 | 1,250 |
| NotoSansArabic-Bold.ttf | 261,460 | 243,820 | 94,292 | 1,250 |
| NotoSansHebrew-Regular.ttf | 26,860 | 25,204 | 12,894 | 147 |
| NotoSansHebrew-Bold.ttf | 26,860 | 25,204 | 12,927 | 147 |
| NotoSansDevanagari-Regular.ttf | 243,520 | 225,236 | 92,059 | 281 |
| NotoSansDevanagari-Bold.ttf | 250,176 | 231,752 | 95,078 | 281 |
| NotoSansSC-Regular.otf | 8,331,336 | 946,616 | 834,212 | 3,945 |
| NotoSansSC-Bold.otf | 8,543,168 | 956,892 | 848,976 | 3,945 |
| NotoSansJP-Regular.otf | 4,533,028 | 794,888 | 701,813 | 3,290 |
| NotoSansJP-Bold.otf | 4,656,448 | 813,168 | 718,786 | 3,290 |
| NotoSansKR-Regular.otf | 4,644,748 | 350,204 | 264,886 | 2,536 |
| NotoSansKR-Bold.otf | 4,816,044 | 361,128 | 273,631 | 2,536 |
| total | 40,568,540 | 5,191,432 | 4,039,476 | |

embedded fonts before: 1,744,532 bytes in nine TTFs (834,203 with gzip
-9). after: 6,935,964 bytes in 21 files (4,873,679 with gzip -9). the
`include_bytes!` goes into every binary and the wasm as is; CFF barely
compresses (SC 946 to 834 KB), so the transfer of the wasm grows by about
4 MB. the Noto faces are borrowed from the binary (`Source::Binary` over
the `&'static [u8]`), not copied into each font system.

### how the fallback reaches them

no family name points at them. cosmic-text 0.18.2 tries, per word, the
requested family, then the per-script list of its `Fallback`, then the
common list (both only at the exact weight), then every face in the
database sorted by weight distance, stretch, style and fontdb id
(`FontFallbackIter::next_item`, `FontMatchKey`). the embedded path builds
its `FontSystem` with `PlatformFallback`: empty lists on wasm and android,
the Linux lists on ios (which name "Noto Sans Arabic", "Hebrew" and
"Devanagari" and match at 400 and 700, but name "Noto Sans CJK SC/JP/KR"
for the CJK scripts, not the subsets' "Noto Sans SC/JP/KR"). so a word the
Latin face cannot shape lands in the last step, where the Noto face that
covers the script is the first hit on every target. that step ignores the
weight filter, so 500 lands on Regular and 600 on Bold; the audit holds
the six scripts to zero `.notdef` at 400, 500, 600 and 700 in both text
families. the Noto faces load after every Latin face, so a glyph the Latin
faces carry never reaches them, and the Latin shaping and pixels did not
move (the audit golden's Latin rows and the 176 regress baselines are
unchanged).

### the order of the unified Han

in the last step the Noto faces of one weight tie on everything but the
id, so the load order decides: SC, then JP, then KR. SC first is the
choice cosmic-text's own Han tables make for any locale other than `ja`,
`ko`, `zh-HK` and `zh-TW`, and the embedded locale is `en-US`. KR carries
no hanja. the consequence: a Chinese word comes out in SC; a Japanese
word takes its kanji and CJK punctuation from SC and its kana from JP
("こんにちは、世界" is JP and SC in the audit). that is also what
cosmic-text does on the desktop for a Japanese word that starts with a
kanji under `en-US`.

a `Fallback` of our own (kana to JP, hangul to KR, Han by locale) would
route each script to its face before the last step, but its
`script_fallback` takes `unicode_script::Script`, which cosmic-text does
not re-export and the engine does not depend on. `unicode-script` 0.5.8 is
already in Cargo.lock; making it a direct dependency of the engine is the
next step if Japanese text should never borrow SC forms.

### the desktop

unchanged. the macOS lists name system faces per script (Geeza Pro, Arial,
Devanagari Sangam MN, Hiragino Sans, Apple SD Gothic Neo, and Arial
Unicode MS where PingFang is not found), and they come before the last
step; at 500 and 600, where the lists find no exact weight, the system
faces still win the last step because they load first. the faces the
desktop picked for the eight audit lines at 400, 500, 600 and 700 were the
same before and after this change on the probe's machine. the desktop and
the embedded path therefore still draw these scripts with different faces;
making the desktop prefer Noto is a separate decision (system fonts as an
option by setting was out of scope here).

the text probe gained what measuring the embedded path in pixels on a
desktop needed: one font-system constructor in `text/fonts.rs` shared by
`TextSystem`, `TextMeasurer` and the probe, and a per-thread face set that
`probe::render_with(Faces::Embedded, ...)` switches around a headless
render. nothing outside `src/text` changed; on wasm, android and ios the
switch has nothing to switch.

### what stays for emoji

task-74 phase 5. an emoji needs two things: a face with emoji and the
color path in the glyph atlas (COLR, CBDT or sbix; `rasterize_and_upload`
returns `None` for anything that is not a mask today). a color emoji
font embedded before that path exists reserves the advance and draws
nothing, the hole the phase 1 probe measured on the desktop. the audit's
emoji rows are still tofu on the embedded path (four for the ZWJ family,
two for the thumb with skin tone) and are not held to zero.

## consequences

- wasm, android and ios draw Arabic, Hebrew, Devanagari, Chinese, Japanese
  and Korean; the widths the layout measures are the real ones (the
  Chinese line measured 49.4 px of tofu, now 80.0 px of ideographs).
- about 5.2 MB more in every binary and wasm; every further weight of a
  CJK face costs about as much as all the Latin faces together.
- the coverage is a set, not the script: a hanzi outside GB2312 level 1
  or a kanji outside JIS level 1 falls to the other CJK face if it has it
  and to tofu if not; Korean compatibility jamo (ㅋ, U+3131 to U+318E) are
  not in the KR subset.
- widening a set, adding a weight or a script is an edit to
  `script/font-subset` and a rerun; the sha256 and the stdlib codecs keep
  the result reproducible.

## alternatives considered

- system fallback only: no fonts on wasm, and the desktop output depends
  on the machine (the phase 1 audit). rejected by the decision.
- whole fonts: SC alone is 8.3 MB per weight, JP and KR 4.5 MB.
- Regular only: cosmic-text does not synthesize bold, so 600 and 700 text
  in these scripts would have drawn Regular next to a bold Latin face.
