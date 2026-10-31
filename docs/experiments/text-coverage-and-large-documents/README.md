---
id: experiment-text-coverage-and-large-documents
title: "text coverage per writing system, and the cost of large documents in the editor"
project: plev
type: experiment
status: reference
date: 2026-09-24
date_basis: text
recorded: 2026-10-31
topics: [text, fonts, text-editing, performance]
relations: [task-74, task-74/phase-1-audit, task-74/phase-7-large-documents]
evidence: [df270334, ad6d0d6a, 36ac4257, e7427780, 71e0a6f2, 8b4c58e9, 727c378e]
source: reconstruction
source_revision: c2e3f8cd
revised: 2026-10-31
review: reviewed
---

# text coverage per writing system, and the cost of large documents in the editor

Two questions of task-74, the text task, were measured in two runs. On 2026-09-24, before any correction, eleven corpus files (one per writing system) were shaped and drawn with the embedded fonts and with the desktop fonts: on the embedded path every non-Latin script comes out as tofu, and no embedded face has the `liga` feature, while bidi, mark composition and kerning work without a font. On 2026-09-25, a release build with one million lines showed that line positions were wrong from line 100 thousand on and that opening copied the whole text; after three changes, opening to the first frame took 21,0 ms, the peak heap on opening fell from 121,3 MiB to 65,0 MiB, and the worst frame measured with one million lines was below 1,3 ms at p99.

## Question

Two questions, both from task-74 (README, "escopo"). The first is scope item 1, the coverage audit, in a translation made for this page: "corpus per writing system (Latin with diacritics, Arabic, Hebrew, Devanagari, CJK, emoji with ZWJ, combining marks, mixed RTL and LTR), headless render and measurement: what is missing and what comes out wrong. Output: a report." The second is scope item 7, large documents: "open, scroll and edit a file of one million lines; line shaping with a cache, virtualized layout; benchmarks with numbers." The acceptance criterion that depends on the second is "benchmark de um milhão de linhas com números" (task-74 README, "critérios de aceitação").

## Setup

### Coverage audit (phase 1)

- Date 2026-09-24. Branch `task/74a-text-audit` on `97634db6`. Measurement only: no render correction, and no font came in or out (task-74 phase 1, header and "data").
- Corpus: `rust/engine/tests/fixtures/text_corpus/`, eleven files, one per writing system, with phrases written by hand for this corpus (phase 1, "corpus").
- Method: one line per writing system at 16 px, Inclusive Sans, with `probe::shape_report` for shaping and `probe::render` at scale 2 for ink. Shaping was also run with Inter, with wrap at 80 px. Codicons are not part of the text families (phase 1, "medição por shaping").
- "Embedded" means shaping with the nine faces in the repository, which is what wasm, android and ios have. "Desktop" means the same shaping with the system fonts of the Mac the audit ran on, which is what `TextSystem` and `TextMeasurer` use on desktop. The desktop locale is the system's, `en-BR` on that machine; the embedded one is `en-US` (phase 1, "a sonda" and "o que as folhas mostram").
- Machine: the gate ran "nesta máquina (metal, macOS 26.6.2)" (phase 1, "gate e contagem"). The chip is not recorded. Build profile: not recorded.
- Test: `rust/engine/tests/text_audit.rs`, with the golden `rust/engine/tests/fixtures/text_audit.txt`. The pixel test ran in the gate with a GPU, with no `#[ignore]`.

### Large documents (phase 7)

- Date 2026-09-25. Release profile, Apple M4, 16 GB, macOS 26.6.2 (task-74 phase 7, "a sonda" and "o benchmark").
- Command, as the page gives it: `cargo test --offline --release -p engine --test editor_large_bench -- --ignored --nocapture`. The same page names `rust/engine/tests/editor.rs` as the benchmark file, in its probe and its handoff. The file and the cargo target name are not the same in the page; both are shown here as the source gives them.
- Document: generated, with lines of 44 to 65 bytes, all different. One million lines are 57 MiB of text.
- Scenarios, each reported as average / p99 in ms: the wheel moves 3 lines per frame for 2000 frames; the scrollbar drag goes from top to end in 300 frames, each in a new viewport; the wheel with everything selected runs 1000 frames after `select_all`; the edit inserts and deletes one character in the middle line, 200 times each, measured until the next frame is built.
- `scene` stops at `Compositor::resolve_scene`, with no GPU. `frame` goes through `headless::Renderer::draw`, which shapes, rasterizes and submits.
- Memory: the CPU heap, counted by an allocator inside the test. The GPU glyph atlas is not counted.
- Each table is the second of two runs in a row.

## Results

### Coverage: what is missing and what comes out wrong, per writing system

Source: [task-74 phase 1](../../tasks/task-74-text-and-editing/phase-1-audit.md), sections "a sonda" and "medição por shaping". Decimals are written as in the source, with a comma; widths are in px.

| writing system | embedded: glyphs / notdef / width | desktop: glyphs / notdef / width / face that filled it | what comes out wrong on the embedded path |
|---|---|---|---|
| Latin with diacritics | 60 / 0 / 457,4 | 60 / 0 / 457,4 / Inclusive Sans only | nothing: 7 lines in 80 px |
| Arabic | 27 / 24 / 251,7 | 26 / 0 / 150,9 / Geeza Pro | 24 of 27 glyphs are tofu (only the 3 spaces are not); 4 lines in 80 px against 2; no lam-alef (27 glyphs against 26 with Geeza Pro); the tofu is 9,9 px per letter, and a real letter is 4 to 7 px |
| Hebrew | 9 / 8 / 83,9 | 9 / 0 / 66,2 / Arial | 8 of 9 tofu; with niqqud, 7 of 7: the marks are zero-advance tofu (`0/0.0`) that stamp over the box before them |
| Devanagari | 13 / 12 / 123,5 | 10 / 0 / 74,8 / Devanagari Sangam MN | 12 of 13 tofu; no conjunct forms (13 glyphs against 10); the pre-base vowel does not reorder; क्षत्रिय is 8 tofu against 4 real glyphs |
| Chinese | 5 / 5 / 49,4 | 5 / 0 / 80,0 / Arial Unicode MS | 5 of 5 tofu; 49,4 px against 80,0 px real: the tofu is narrower than the ideograph |
| Japanese | 8 / 8 / 79,1 | 8 / 0 / 128,0 / Hiragino Sans and Arial Unicode MS | 8 of 8 and 4 of 4 tofu; in Inter (84,0 px) the phrase こんにちは、世界 breaks in the middle of a word at 80 px (byte 21, between 世 and 界), because there is no space and the wrap is `WordOrGlyph` |
| Korean | 9 / 7 / 77,8 | 9 / 0 / 105,4 / Apple SD Gothic Neo | 7 of 9 tofu (the comma and the space come from Latin) |
| Emoji, ZWJ family (7 code points) | 7 / 4 / 39,6 | 1 / 0 / 16,0 / Apple Color Emoji, no ink | 4 tofu of 9,9 px, which make 39,6 px; the 3 ZWJ are hidden (glyph 1, advance 0) and do not turn into tofu |
| Latin decomposed (17 code points) | 11 / 0 / 84,4 | 11 / 0 / 84,4 / Inclusive Sans only | nothing: the 17 code points become 11 glyphs with 0 notdef; the shaper composes é, ã, ö, ñ and ệ into precomposed glyphs |
| Bidi, "abc אבג def" | 11 / 3 / 92,4 | 11 / 0 / 86,8 / Arial | the visual order is right (L0:0..4 R1:4..10 L0:10..14); only the tofu is wrong |

Findings that are not in the table, as the same source records them:

- Ligatures: no embedded face has `liga`. Inclusive Sans has `ccmp`, `kern` and `mark`; Inter has `calt`, `dlig`, `kern`, `mark` and `mkmk`. "office waffle fjord affix flow" is 30 glyphs for 30 characters in both faces, so no ligature forms.
- Kerning works: "AVATAR WAVY Type LT" measures 163,0 against 168,4 for the loose glyphs in Inclusive Sans (-5,4 px), and 175,4 against 184,8 in Inter (-9,4 px).
- Combining marks: 17 code points become 11 glyphs with 0 notdef. A stacked mark without a precomposed form was not measured, because Inclusive Sans has no `mkmk`.
- Bidi, "hello مرحبا 123 world": L0:0..6 L2:17..20 R1:6..17 L0:20..26. The digits sit at level 2 inside the RTL run, as the UBA prescribes; the only error is the tofu (3 and 5 notdef).
- Emoji: the thumb with a skin tone (U+1F44D U+1F3FD) becomes 2 tofu. U+2764 with VS16 finds the heart in Inter through the fallback inside the embedded faces, and the VS16 is hidden.
- Warnings: the probe rendered the corpus four times (two families, two rounds), each round with a new `Renderer`. It counted 30 "text fallback" warnings: "8 faces na rodada Inclusive Sans (...) + 7 na rodada Inter (...), vezes duas".

Pixel measurement, on the desktop path at scale 2 (phase 1, "medição por pixel"): 38 lines in the two families, of which 32 are "ok". The other 6 are "largura medida sem tinta": the three colour emoji (the ZWJ family, the thumb with skin tone and U+1F600) in both families. The shaping reserves 16,0 px of advance, the swash returns `SwashContent::Color`, and `rasterize_and_upload` in `atlas.rs` returns `None` for everything that is not a `Mask`, so the quad does not come out: "na tela isso é um buraco do tamanho de um emoji". No line has ink beyond its measured width or before its origin. The katakana テキスト is the one that falls furthest short: 61,0 of 64,0 px. The determinism test renders each sheet twice and compares the RGBA bytes; it is green for both families.

Font size, embedded: 1.744.532 bytes (1,7 MB) in nine TTF files: Inclusive Sans in five weights (372.656 bytes in total), Inter in three weights (1.248.684 in total), and codicons (123.192).

Estimate of Noto subsets for phase 2. The source says these are "ordens de grandeza, não medidas": they come from memory, and no font was downloaded.

| writing system | font | approximate size, one weight | confidence |
|---|---|---|---|
| Arabic | Noto Sans Arabic | 150 to 200 KB | medium |
| Hebrew | Noto Sans Hebrew | 40 to 60 KB | medium |
| Devanagari | Noto Sans Devanagari | 200 to 250 KB | medium |
| Chinese, simplified | Noto Sans SC, subset of the 3.500 common hanzi | 1,2 to 2 MB; the full GB2312 (6.763) 2 to 3 MB; the full font 10 to 16 MB | low |
| Japanese | Noto Sans JP, kana and JIS level 1 (2.965 kanji) | 1,5 to 2 MB; full 5 to 6 MB | low |
| Korean | Noto Sans KR, 2.350 KS X 1001 syllables | 500 to 700 KB; 11.172 syllables 3 to 4 MB | low |
| Emoji | Noto Color Emoji | full CBDT 10 to 11 MB; COLRv1 8 to 9 MB; a subset of 300 emoji in COLR 0,5 to 1 MB | low; depends on phase 5 |
| Greek and Cyrillic | already in Inter (105 and 248 code points); Inclusive Sans has none | 0 | high |

The source adds that Arabic, Hebrew and Devanagari together, at one weight, are "cerca de 0,4 a 0,5 MB, um quarto do que já embutimos", and that each CJK script costs more than everything embedded today, even as a subset.

What the audit handed to phase 2 (phase 1, "o que a fase 2 precisa saber"):

1. The embedded path (wasm, android, ios) shows tofu in every non-Latin script. The desktop hides it with system fonts, per face and per machine, and fontdb does not find all of them (PingFang). The two are different products today.
2. The tofu has its own width, 9,9 px in Inclusive Sans and 10,5 px in Inter at 16 px, so the layout measure is wrong before the pixel. A mark without a glyph has zero advance and stamps over the box before it.
3. Bidi, mark composition and kerning do not depend on the font decision. For RTL, only the glyph is missing.
4. Emoji needs a font with emoji and the colour path in the atlas (phase 5). Embedding an emoji font before phase 5 gives advance without ink, the hole the probe measured on desktop.
5. Ligatures (phase 6): none of the embedded faces has `liga`. Phase 6 either chooses another Latin face or accepts that there is nothing to ligate.
6. Determinism between desktop and wasm needs a `TextSystem` and a `TextMeasurer` that can start without system fonts. Today this is a platform `cfg` in `system.rs` and `measure.rs`, with no runtime key.
7. The locale enters the choice of the CJK face (han unification). It comes from the system on desktop and is fixed to `en-US` on the embedded path.

Gate, phase 1 (phase 1, "gate e contagem"): `cargo test --workspace` went from 2451 passed, 0 failed, 16 ignored in 113 test binaries on the base `97634db6`, to 2454 passed, 0 failed, 16 ignored in 114 binaries (three tests of `text_audit` and one binary). The urnaui leg had 82 passed, 0 failed, 1 ignored in 6 binaries. The total gate: 2536 passed, 0 failed, 17 ignored.

### Large documents: the cost before the three changes

Source: [task-74 phase 7](../../tasks/task-74-text-and-editing/phase-7-large-documents.md), section "a sonda", table "custo medido antes de mudar" (average / p99, ms).

| lines | measured up to | open | wheel, 3 lines per frame | scrollbar drag | wheel, all selected | edit in the middle | kept heap (MiB) | peak on open (MiB) |
|---|---|---|---|---|---|---|---|---|
| 10 mil | scene | 0,3 | 0,021 / 0,026 | 0,055 / 0,060 | 0,589 / 0,629 | 0,028 / 0,033 | 0,6 | 1,1 |
| 100 mil | scene | 2,0 | 0,021 / 0,025 | 0,088 / 0,099 | 0,590 / 0,636 | 0,030 / 0,034 | 6,2 | 11,8 |
| 1 milhão | scene | 22,8 | 0,021 / 0,025 | 0,092 / 0,101 | 0,590 / 0,628 | 0,029 / 0,034 | 64,2 | 121,3 |
| 10 mil | frame | 1,0 | 0,364 / 0,585 | 0,701 / 0,907 | 0,932 / 1,063 | 0,262 / 0,341 | 0,6 | 1,5 |
| 100 mil | frame | 3,1 | 0,357 / 0,573 | 1,007 / 1,119 | 0,919 / 1,001 | 0,280 / 0,393 | 6,2 | 11,8 |
| 1 milhão | frame | 23,3 | 0,357 / 0,544 | 1,194 / 2,884 | 0,946 / 2,026 | 0,311 / 1,071 | 64,2 | 121,3 |

What the probe measured before any change:

- Line positions were f32 against the top of the document. With the theme's 19,6 px line, the step between two lines was 19,5 at line 99.990 and 20 at line 499.963. A click at 31,1 px from the top, at the end of the document, fell on line 999.972 instead of 999.971. These numbers come from the new tests in `editor/tests_large.rs` (the probe's name for them), run before the fix.
- Opening copies the text before building the rope (`rust/editor/src/document/mod.rs:51-62`). The peak on opening is "quase o dobro do que fica": 121,3 MiB against 64,2 MiB.
- With everything selected, the probe says the scene costs "28 vezes a da roda simples", the cause being `clusters_styled` shaping each line on screen again every frame.
- The editor was virtualized already: only the lines that cut the viewport, plus `overscan_lines` (8) on each side, are emitted, and the total height comes from the line count.

### Large documents: the cost after the three changes

Source: [task-74 phase 7](../../tasks/task-74-text-and-editing/phase-7-large-documents.md), section "o benchmark" (the same machine, after the three changes).

| lines | measured up to | open | wheel, 3 lines per frame | scrollbar drag | wheel, all selected | edit in the middle | kept heap (MiB) | peak on open (MiB) |
|---|---|---|---|---|---|---|---|---|
| 10 mil | scene | 0,2 | 0,021 / 0,025 | 0,054 / 0,064 | 0,146 / 0,177 | 0,019 / 0,022 | 0,6 | 0,6 |
| 100 mil | scene | 1,6 | 0,021 / 0,025 | 0,086 / 0,099 | 0,146 / 0,159 | 0,020 / 0,024 | 6,2 | 6,2 |
| 1 milhão | scene | 17,5 | 0,021 / 0,029 | 0,095 / 0,126 | 0,148 / 0,185 | 0,020 / 0,040 | 64,2 | 64,2 |
| 10 mil | frame | 0,8 | 0,365 / 0,593 | 0,673 / 0,826 | 0,495 / 0,607 | 0,304 / 0,771 | 0,6 | 1,5 |
| 100 mil | frame | 3,1 | 0,368 / 0,625 | 1,019 / 1,227 | 0,481 / 0,558 | 0,258 / 0,317 | 6,2 | 7,1 |
| 1 milhão | frame | 21,0 | 0,360 / 0,587 | 1,042 / 1,241 | 0,482 / 0,568 | 0,268 / 0,343 | 64,2 | 65,0 |

The three changes (phase 7, "o que mudou"):

1. Line positions in f64. `line_y` in `rust/engine/src/widgets/editor/view.rs` computes the distance from the top of the document in f64; `line_top`, `hit_test_point` and `scroll_to_cursor` use it, and only the small difference on screen goes back to f32. The wheel's snapping to whole lines in `input.rs` uses the same arithmetic. For a small document at zero scroll, the output is bit for bit the same as before; the 176 baselines confirm it.
2. `Document::load` (`rust/editor/src/document/mod.rs`) builds the rope straight from the `&str`, and copies only when there is a `\r` to normalize.
3. A per-line shaping cache for `TextMeasurer::clusters_styled` (`rust/engine/src/drawing/text/cluster_cache.rs`, new). The key is the text and the style, with the exact width and no line index, so an edit invalidates only the lines whose text changed. The budget is 4 MiB per entry, counting key and clusters; the least used entry leaves, and a line bigger than the whole budget does not enter.

No public API changed, and nothing was removed.

Handoff numbers for one million lines, from the phase 7 table "antes e depois": opening to the first frame went from 23,3 ms to 21,0 ms; the peak heap on opening from 121,3 MiB to 65,0 MiB; the edit in the middle until the next frame from 0,311 / 1,071 ms to 0,268 / 0,343 ms. The integration note rounds some of these to "abrir 21 ms", "65 MiB" and "0,27 ms" (task-74 phase 7, "fase 7 integrada"); the values above are the table's.

Tests and gate (phase 7, "testes" and "gate e regress"): eight new tests and one ignored benchmark. The large-document tests in `rust/engine/src/widgets/editor/tests/large.rs` run one million lines, in debug and in the gate; each frame checks that each text node is the document line at that position. The four of them (open and scroll to the end, scroll through eleven points, click near the end, and insert, delete and break a line in the middle) failed before the f64 correction. `cargo test --workspace` went from 3136 passed and 27 ignored to 3144 passed, 0 failed, 28 ignored. The urnaui leg had 85 passed, 0 failed, 1 ignored. After touching `tool/bench/regress/src/main.rs`, `regress -- check` gave 176 unchanged, 0 changed. Outside the gate, `cargo check` for `aarch64-apple-ios-sim` with `-D warnings` was green, and `cargo ndk -t arm64-v8a check -p uigallery --lib` compiled with two old dead-code warnings.

## Limits

Coverage audit (phase 1, "limitações e dívidas declaradas"):

- The pixel measurement covers the desktop path only. The ink of a tofu box was not rendered, because that needs a `TextSystem` that starts without system fonts (item 6 above).
- The corpus has no stacked marks without a precomposed form, no kinsoku (CJK line-break rules), and no Arabic-Indic digits.
- The desktop results depend on the machine's fonts. fontdb does not find PingFang on this macOS 26: it lives in a private framework, so the Han glyphs come from Arial Unicode MS.
- The Noto sizes are from memory, and no font was downloaded.
- `shape_report` repeats the `Attrs` assembly of `measure.rs` and `system.rs`, so there are three copies. The "text fallback" warning in `atlas.rs` carries a dash inherited from its original text. Both are recorded as debt.
- Ligatures (scope item 6) have no phase page and no recorded execution (task-74 README, "Phases"). The audit found no `liga` in either embedded face.

Large documents (phase 7, "pendências"):

- `ScrollState` keeps the offset in f32, in the engine, outside this phase. With one million lines of 19,6 px, near the end it moves in steps of 2 px: a trackpad delta under 1 px is lost, and the end of the document is up to 1 px from the edge. Lines relative to each other are right. Fixing it changes the public API, which is a decision.
- With everything selected, the scene still costs 7 times the simple wheel (0,148 against 0,021 ms). The shaping no longer repeats, but `VisualLine` rebuilds the caret stops of each line every frame.
- A very long line (a minified file of megabytes on one line) was not measured. There is no horizontal virtualization, and a line over 4 MiB of clusters does not enter the cache.
- The editor does not wrap lines: the virtualization relies on a fixed height per line. Soft wrap in a large document needs a height index. The benchmark was measured without wrap, as the briefing asked.
- Opening is still O(n) in the text (17,5 ms of scene for 57 MiB), and the caller holds the `String` and the rope at the same time while opening. Loading in chunks straight from the file was not done.
- The memory figures are the CPU heap only; the GPU atlas is not counted.
- Between rounds, the averages varied by up to about 15%, and the frame's p99 by up to double. The first round before the change gave a p99 of 1,973 ms for the wheel with 10 mil lines; the second gave 0,585.
- The IME item (phase 4) is still open and needs brenner to run the script (task-74 README, "critérios de aceitação"; phase 7, "fase 7 integrada").

Two sources disagree on file names and are shown above as they are: the benchmark file is `rust/engine/tests/editor.rs` in the phase 7 probe and handoff, and the cargo target in its commands is `editor_large_bench`. The large-document tests are `editor/tests_large.rs` in the probe and `rust/engine/src/widgets/editor/tests/large.rs` in the handoff.

## Records

- [task-74 record](../../tasks/task-74-text-and-editing/README.md): the task's scope in nine items, its phases, its status (in progress) and its acceptance criteria.
- [task-74 phase 1, coverage audit](../../tasks/task-74-text-and-editing/phase-1-audit.md): the corpus, the shaping and pixel measurements, the font estimates, the gate counts and the list for phase 2.
- [task-74 phase 7, large documents](../../tasks/task-74-text-and-editing/phase-7-large-documents.md): the editor probe, the three changes, the benchmark before and after, the gate and the pending items.
