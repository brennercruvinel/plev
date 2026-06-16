---
project: phi
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-13
domain: task-tracking
---

# task-37: animation enhancements, P1, done

## objetivo
adicionar keyframesequence, repeat/reverse/delay, step/hold easing, e const-generic interpolate. 4 patterns de keyframe/mina.

## justificativa
maior feature gap no sistema de animacao. tween<t> so suporta 2 pontos. sem repeat, sem delay, sem animacoes multi-step.

## dependencias
- task-27 (animation system), concluida
- task-35 (spring fix), idealmente primeiro, mas independente

## referencia
- patterns c2, c3, c4, c5, c6 em `mission/knowledge/extracted-patterns.md`

## estimativa
~400 LOC

## checklist

### c5: const-generic array interpolate (~20 LOC)
- [x] substituir 3 impls manuais de interpolate para [f32;2/3/4] por uma const-generic
- [x] substituir 3 impls manuais de springinterpolate por uma const-generic
- [x] `std::array::from_fn(|i| self[i].lerp(&target[i], t))`

### c6: step/hold easing (~10 LOC)
- [x] adicionar `Easing::Step` (snap em t=0.5)
- [x] adicionar `Easing::Hold` (retorna 0.0, snap para 1.0 em t=1.0)
- [x] match arms no `apply()` do easing enum

### c4: tween repeat/reverse/delay (~80 LOC)
- [x] adicionar campos: `delay: Duration`, `repeat: Repeat` (none/times(u32)/infinite), `reverse: bool`
- [x] implementar logica em `tick()`: skip durante delay, cycle count, ping-pong
- [x] builder methods: `.with_delay()`, `.with_repeat()`, `.with_reverse()`

### c2: keyframesequence (~250 LOC)
- [x] struct `KeyframeSequence<T: Interpolate>` com `Vec<Keyframe<T>>`
- [x] cada `Keyframe<T>` tem: valor, timestamp (f32 0.0-1.0), easing
- [x] `advance_by(dt)`, `advance_and_reverse(dt)`, `advance_and_wrap(dt)`
- [x] `now() -> T` retorna valor interpolado atual
- [x] builder: `KeyframeSequence::new().keyframe(value, time, easing).keyframe(...).build()`
- [x] testes: multi-step animation, reverse, wrap, easing per-segment

### c3: animationstate (lightweight, opcional)
- [x] `AnimationState<S: Hash+Eq, T: Interpolate>` mapeia estados para tweens/keyframesequences
- [x] `set_state(state)` faz blend-on-transition (override keyframe 0% com valor atual)
- [x] pode ser adiada se c2+c4 ja cobrem os use cases

## criterios de aceite
1. keyframesequence com 3+ keyframes e easing diferente por segmento
2. tween com repeat infinite + reverse (ping-pong)
3. tween com delay de 500ms funciona
4. step/hold easing funciona
5. const-generic: [f32; 5] interpola corretamente
6. zero regressao nos 35 testes existentes
7. exemplo atualizado ou novo demonstrando keyframesequence
