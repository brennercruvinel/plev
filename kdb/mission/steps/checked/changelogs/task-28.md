---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-13
domain: changelog
---

# task-28 changelog: editable text

## fase a: textbuffer
- [x] textbuffer com cursor, selection, char-aware movement
- [x] insert/delete/move/select operations
- [x] testes (24 unit)

## fase b: cursor-pixel mapping
- [x] cursor_to_x / x_to_cursor (approximate, font_size * 0.6)
- [x] testes (5 unit)

## fase c: textinput component
- [x] textinput com focus, blink (530ms), placeholder, rendering
- [x] keyboard handling (char, backspace, delete, arrows, home, end, select_all)
- [x] mouse click cursor positioning
- [x] build_scene() generates scenenodes (bg, border, text, cursor, selection)
- [x] testes (13 unit)

## fase d: IME integration
- [x] handle_ime(committed, preedit) bridge method
- [x] testes (2 unit)

## fase e: builder API, skipped
- skipped: textinput already generates scenenodes via build_scene(), sufficient for proof of life
- builder integration deferred to future task

## fase f: example + validation
- [x] examples/text_input_demo.rs: 3 fields, tab cycling, click focus, cursor blink, live preview
- [x] cargo test --workspace: 325 tests passing
- [x] cargo check --workspace --examples: zero warnings
