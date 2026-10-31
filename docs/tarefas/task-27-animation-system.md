---
id: task-27
title: "task-27: animation system"
project: plev
type: tarefa
status: concluido
date: 2022-12-21
date_basis: cabecalho
topics: [animacao, sinais, renderizacao]
relations: [task-04]
source: [docs/log/steps/task-27.md, docs/log/changes/task-27.md]
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# task-27: animation system

Registra o sistema de animação do plev: um `Tween<T>` que interpola valores de signals a partir de um `AnimationTick` (dt e elapsed) lido no render loop, com funções de easing e, como etapa opcional, uma mola. O Registro de entrega lista as cinco fases, da infraestrutura de tempo ao exemplo `animation_demo`, e a tarefa está marcada como concluída. Importa porque o paper precisava mostrar o reactive system em uso prático.

## objetivo
sistema de animação baseado em signals, interpolação temporal de valores para transições, hover effects e movimentos. mínimo necessário para demos convincentes e para o paper mostrar o reactive system em uso prático.

## dependências
- task-04 (signal system)
- render loop com `request_redraw()` contínuo

## contexto técnico
- o render loop já roda continuamente (winit `request_redraw()`)
- signals não auto-triggeram re-render, mas views leem `.get()` a cada frame
- existe frame counter (`ReadSignal<u64>`) mas **não há delta_time**
- `std::time::Instant` não existe no WASM, precisa de solução cross-platform
- builder `.bind()` é stub, animações devem funcionar diretamente com signals

## design

### abordagem: `Tween<T>` + `AnimationTick`

```rust
// Infra no render loop
struct AnimationTick {
    dt: f32,          // seconds since last frame
    elapsed: f32,     // total seconds since app start
}

// API para o usuário
let opacity = create_tween(0.0f32, Duration::from_millis(300), Easing::EaseOut);
opacity.set_target(1.0);  // inicia animação rumo a 1.0

// No render:
let current = opacity.get(); // valor interpolado no frame atual
div().opacity(current)
```

### alternativa avaliada e descartada
`transition(property, duration, easing)` no builder, descartada porque:
- builder elements são efêmeros (recriados a cada frame), não retêm estado
- animação precisa de estado persistente entre frames -> signals são o lugar certo

## checklist

### fase a, infra de tempo
- [ ] escolher crate para tempo cross-platform (`web-time` ou `instant`), pesquisar compatibilidade com wgpu 28 e WASM
- [ ] adicionar `AnimationTick { dt: f32, elapsed: f32 }` ao render loop em `window.rs`
- [ ] expor `dt` e `elapsed` como signals ou via `ViewContext`
- [ ] garantir que funciona em WASM (sem `std::time::Instant`)
- [ ] teste: dt > 0 em frames consecutivos (native)

### fase b, easing functions
- [ ] módulo `src/easing.rs` com enum `Easing`
- [ ] implementar: linear, easein, easeout, easeinout (cubic)
- [ ] implementar: cubicbezier(f32, f32, f32, f32) para custom
- [ ] função `ease(t: f32, easing: Easing) -> f32` onde t ∈ [0, 1]
- [ ] testes unitários para cada curva (valores em t=0, t=0.5, t=1)

### fase c, tween<t>
- [ ] struct `Tween<T>` em `src/animation.rs`
- [ ] `create_tween(initial: T, duration: Duration, easing: Easing) -> Tween<T>`
- [ ] `.set_target(value: T)`, inicia animação do valor atual para target
- [ ] `.get() -> T`, retorna valor interpolado baseado em elapsed time
- [ ] `.is_animating() -> bool`
- [ ] trait `Interpolate` para tipos animáveis: `f32`, `[f32; 4]` (cores), `(f32, f32)` (posições)
- [ ] integração com signal runtime: tween lê `elapsed` do tick e interpola
- [ ] testes: tween de 0->1 em 1s, verificar valor em t=0.5

### fase d, spring (opcional, stretch goal)
- [ ] `create_spring(initial: T, stiffness, damping)`, animação física
- [ ] útil para gestos (drag release -> snap back)
- [ ] pode ser adiado para depois do paper

### fase e, integration + example
- [ ] `examples/animation_demo.rs` mostrando:
  - fade in/out (opacity tween)
  - slide (position tween)
  - color transition (color tween)
  - hover effect (mouse enter -> animate, leave -> animate back)
- [ ] verificar: animações rodam suave a 60fps no macos
- [ ] verificar: `cargo check --target wasm32-unknown-unknown` compila
- [ ] documentar API no módulo (doc comments)

## estimativa
fase a-c: ~400-600 LOC. fase d: +200 LOC. fase e: ~150 LOC.

## riscos
- performance: se muitos tweens ativos, o overhead por frame deve ser negligível (são só lerps)
- WASM time: `web-time` crate já é padrão da comunidade, baixo risco
- borrow checker: tween precisa ler tick global, usar signal ou `thread_local!` como o runtime de signals

## Registro de entrega

### fase a: infra de tempo
- [ ] web-time dependency adicionada
- [ ] animationtick struct
- [ ] dt/elapsed no render loop (window.rs)

### fase b: easing
- [ ] 30+ penner easings implementados
- [ ] cubicbezier custom
- [ ] testes por variante

### fase c: tween
- [ ] interpolate trait + impls (f32, [f32;2], [f32;3], [f32;4])
- [ ] tween<t> com set_target/tick/get
- [ ] testes

### fase d: spring (opcional)
- [ ] spring<t> damped harmonic oscillator

### fase e: example + validacao
- [ ] examples/animation_demo.rs
- [ ] cargo test --workspace
- [ ] cargo check --target wasm32-unknown-unknown
