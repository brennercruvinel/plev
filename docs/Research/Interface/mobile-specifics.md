---
id: platform-mobile-specifics
title: "mobile specifics - safe areas, IME, lifecycle"
project: plev
type: reference
status: reference
date: 2021-08-30
date_basis: reconstructed
recorded: 2026-03-08
topics: [mobile, input]
relations: []
source: docs/ref/mobile-specifics.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# mobile specifics - safe areas, IME, lifecycle

Records the mobile platform points that need handling beyond winit 0.30: safe areas, IME with the virtual keyboard, surface lifecycle and scale factor. The result is a table of the APIs verified per platform, with known gaps (for example, `inner_position()` returns an error on iOS and Android) and the keyboard height estimated at 40% of the screen. The record matters because it separates what the engine reads from each platform from what is still a heuristic.

## safe area insets
- android: `WindowExtAndroid::content_rect()` retorna `Rect { left, top, right, bottom }` - insets = outer_size - content_rect
- ios: `inner_position()` retorna `Err(NotSupportedError)` no winit 0.30 - zeros por enquanto
- desktop/WASM: zeros sempre
- recomputar em cada `WindowEvent::Resized` (orientação muda insets)

## IME (input method editor)
- winit expõe `WindowEvent::Ime` com: enabled, preedit(string, option<(usize,usize)>), commit(string), disabled
- `set_ime_allowed(true)` mostra teclado virtual; `set_ime_cursor_area()` posiciona sugestões
- keyboard height: winit não expõe. heurística 40% da tela em mobile, 0 em desktop
- android `content_rect()` muda quando teclado abre - possível refinar comparando antes/depois

## lifecycle
- winit 0.30 `ApplicationHandler` já tem `suspended()` e `memory_warning()` como default methods
- android destrói surface no suspend -> `drop_surface()` + `recreate_surface()` no resume
- ios não destrói metal surface no background
- `memory_warning()` deve fazer purge de caches (shaping + glyph atlas)

## scale factor
- `Window::scale_factor()` retorna f64
- muda em runtime: `WindowEvent::ScaleFactorChanged`
- não cachear - sempre ler do evento ou window

## apis verificadas (winit 0.30.13)
| API | status |
|-----|--------|
| `Window::inner_position()` | ios: err, android: err, desktop: ok |
| `WindowExtAndroid::content_rect()` | android only, retorna rect |
| `Window::set_ime_allowed(bool)` | funciona em todas as plataformas |
| `Window::set_ime_cursor_area()` | funciona em todas as plataformas |
| `WindowEvent::Ime` | funciona em todas as plataformas |
| `Window::scale_factor()` | funciona em todas as plataformas |
