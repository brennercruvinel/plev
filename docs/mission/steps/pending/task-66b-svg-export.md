---
project: plev
audience: [ai-agents, contributors]
status: pending
last-updated: 2026-09-23
domain: generative-runtime
task: task-66b
wave: trilha
depends: [task-66, task-59]
branch: task/66b-svg-export
---

# task-66b: exportação SVG e `--semantics` no headless

## objetivo

fechar o que a task-66 deixou de fora: exportar um frame como SVG com
equivalência honesta e reportada, e ligar `showcase --render --semantics`
agora que a projeção semântica existe (task-59 fase a).

## decisão já tomada (orquestrador, 2026-09-23)

texto vira contorno de glifo (opção A do handoff da task-66): o shaping
do `TextSystem` ganha um ponto `pub(crate)` que devolve, para um
`TextNodeKey`, os glifos posicionados, e o SVG pede o contorno de cada um
ao `SwashCache` (`get_outline_commands`). a string do run vai no
`aria-label` do grupo. nenhum segundo caminho de shaping.

## escopo

- `engine::headless::svg` (novo): rect, rounded rect (borda como stroke
  inset), gradiente linear (interpolação em linearRGB, reportada), path
  tessellado agrupado por cor, texto como contorno, sombra como
  `feGaussianBlur` aproximado, backdrop blur descartado; tudo que não é
  exato entra num relatório devolvido junto (como o importador de svg faz
  no sentido contrário).
- `showcase --render <page> --svg f.svg` e `--semantics f.json` (a
  projeção da página, quando ela implementa `Semantics`; hoje o todo).

## superfície

`crates/engine/src/headless/**`, `crates/engine/src/text/system.rs` (só o
ponto de shaping `pub(crate)`), `crates/engine/src/text/fonts.rs` (só se o
contorno precisar dos bytes), `crates/engine/src/gpu/image.rs` (só um
acessor de pixels para imagem embutida), o CLI do showcase.

## aceite

SVG de três páginas do showcase abre num navegador com o mesmo layout do
png; o relatório lista cada aproximação; o texto do SVG tem os mesmos
glifos do png (comparar a largura medida de três runs); `--semantics` do
todo igual à projeção em processo; gate verde.
