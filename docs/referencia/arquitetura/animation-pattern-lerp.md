---
id: arquitetura-animation-pattern-lerp
title: "animation pattern, frame-based exponential lerp"
project: plev
type: referencia
status: referencia
date: 2022-02-24
date_basis: cabecalho
topics: [animacao, renderizacao, texto]
relations: [task-27]
source: docs/ref/animation-pattern-lerp.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# animation pattern, frame-based exponential lerp

Registra a técnica de animação por quadro com lerp exponencial, usada no message dock antes do sistema formal de animação. O resultado é um easing exponencial sem infraestrutura de tempo, com snap de pixel obrigatório para evitar tremor de subpixel e fonte de tamanho fixo durante a animação, para não re-shapear a cada quadro. A limitação é a dependência da taxa de quadros, que a task-27 deve resolver com delta-time e curvas de easing.

## contexto
antes de task-27 (animation system formal), o messagedock example (`examples/message_dock.rs`) usa animações por frame com lerp exponencial. padrão simples que funciona sem infraestrutura de tempo.

## técnica
```rust
/// Para posições em pixels — snap quando < 0.5px do target
fn smooth(current: f32, target: f32, speed: f32) -> f32 {
    let diff = target - current;
    if diff.abs() < 0.5 { target }
    else { current + diff * speed }
}

/// Para valores normalizados (opacity, scale: 0..1)
fn smooth_n(current: f32, target: f32, speed: f32) -> f32 {
    let diff = target - current;
    if diff.abs() < 0.005 { target }
    else { current + diff * speed }
}
```

cada frame: `current = smooth(current, target, 0.10..0.15)`. produz easing exponencial natural, rápido no início, desacelera ao final.

## pixel-snap obrigatório
```rust
fn px(v: f32) -> f32 { v.round() }
```
toda coordenada (x, y, w, h) passa por `px()` antes de ir ao compositor. sem isso, retângulos animados exibem shimmer/jitter por sub-pixel aliasing, edges oscilam entre linhas de pixel adjacentes.

## font size fixo durante animação
escalar `font_size` proporcionalmente ao scale de um avatar causa re-shaping a cada frame (textnodekey diferente = hash miss = nova geometria de glifos). usar font size fixo (ex: 18.0) e só animar opacity/posição do texto.

## velocidades recomendadas
| propriedade | speed | sensação |
|------------|-------|----------|
| dock width | 0.10 | suave, imponente |
| character x slide | 0.12 | médio |
| hover y lift | 0.15 | responsivo |
| opacity fade | 0.12 | suave |
| color transition | 0.10 | gradual |

## limitações
- dependente de frame rate (60fps -> speed 0.12 é bom, 30fps -> seria 2x mais lento)
- sem curvas de easing customizáveis (sempre exponencial)
- sem duração fixa (converge assintoticamente)
- task-27 resolverá com delta-time, curvas de easing, e tween<t> com duração
