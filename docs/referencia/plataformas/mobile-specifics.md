---
id: plataforma-mobile-specifics
title: "mobile specifics - safe areas, IME, lifecycle"
project: plev
type: referencia
status: referencia
date: 2021-08-30
date_basis: cabecalho
topics: [mobile, input]
relations: []
source: docs/ref/mobile-specifics.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# mobile specifics - safe areas, IME, lifecycle

Registra os pontos de plataforma móvel que pedem tratamento além do winit 0.30: safe areas, IME com teclado virtual, ciclo de vida da surface e fator de escala. O resultado é uma tabela das APIs verificadas por plataforma, com lacunas conhecidas (por exemplo, `inner_position()` retorna erro no iOS e no Android) e a altura do teclado estimada em 40% da tela. O registro importa porque separa o que o engine lê de cada plataforma do que ainda é heurística.

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
