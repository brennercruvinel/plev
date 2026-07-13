---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2022-12-16
domain: changelog
---

# task-27 changelog: animation system

## fase a: infra de tempo
- [ ] web-time dependency adicionada
- [ ] animationtick struct
- [ ] dt/elapsed no render loop (window.rs)

## fase b: easing
- [ ] 30+ penner easings implementados
- [ ] cubicbezier custom
- [ ] testes por variante

## fase c: tween
- [ ] interpolate trait + impls (f32, [f32;2], [f32;3], [f32;4])
- [ ] tween<t> com set_target/tick/get
- [ ] testes

## fase d: spring (opcional)
- [ ] spring<t> damped harmonic oscillator

## fase e: example + validacao
- [ ] examples/animation_demo.rs
- [ ] cargo test --workspace
- [ ] cargo check --target wasm32-unknown-unknown
