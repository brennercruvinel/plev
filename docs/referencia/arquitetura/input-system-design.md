---
id: arquitetura-input-system-design
title: "input system design (task-09)"
project: plev
type: referencia
status: referencia
date: 2021-05-27
date_basis: cabecalho
topics: [input]
relations: []
source: docs/ref/input-system-design.md
source_revision: ec0a224c
revised: 2026-10-30
review: pendente
---

# input system design (task-09)

## event queue vs closures
closures em inputstate criam problemas de borrow checker (app possui inputstate que possui closures que querem mutar app). event queue resolve: eventos são coletados durante window_event(), drenados e processados no início do render().

## hit-testing
- linear scan reverso sobre vec<hitregion>
- último registrado = maior z-order (mais à frente)
- o(n) aceitável para <100 views
- sem dependência de layout tree - bounds vêm direto do rect

## focus
- click em hitregion focusable seta focus
- escape limpa focus
- click fora de qualquer hit region limpa focus
- keyboard events só despachados para view focada

## hover tracking
- cursormoved atualiza hovered_view via hit_test
- gera hoverevent(entered=true/false) quando hovered_view muda
- cursorleft gera hoverevent(entered=false) e limpa cursor_position

## borrow checker pattern no example
process_events() e compositor.begin_frame() devem ser chamados antes do destructuring do gpustate, senão self está emprestado mutavelmente duas vezes.

## coordenadas
winit cursormoved entrega physicalposition<f64>. projeção ortográfica do plev usa pixels físicos. logo position.x as f32 mapeia diretamente - sem conversão DPI.

## viewid
resetado a cada frame (begin_frame reseta next_id para 0). viewid é estável entre frames desde que a ordem de registro não mude (immediate mode).
