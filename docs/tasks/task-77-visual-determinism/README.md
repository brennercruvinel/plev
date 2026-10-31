---
id: task-77
title: "task-77: visual determinism"
project: plev
type: task
status: done
date: 2026-09-25
date_basis: header
topics: [determinism, rendering, platform, mobile, web-wasm]
relations: [adr-0024, adr-0064, task-18, task-19, task-20, task-66, task-74]
source: docs/log/steps/task-77-visual-determinism.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
evidence: [2dd2bc0d, c779fca6, 65d45235, 8af26af5, 35a47b26, a69cd464, 7127f65a, f6c688b5, adbf948b]
---

# task-77: visual determinism

Records the task that measured visual determinism across desktop, web, iOS and android and classified each difference between targets. The desktop came out bit-identical with itself across the gallery's 82 pages, in 4 themes and 4 scales (1312 frames); phase 1 found that the blur does not follow the scale, and phase 1b fixed it, keeping scale 1 byte-for-byte equal to the baseline. At closure, web had 79 pages identical in every byte and 3 silenced, iOS had 78 with a difference expected in the status bar and 4 silenced, all with a written cause, and android stayed at most 2/255 from the desktop; the `colorspace` of the metal layer, the window against headless, and the CI job remained open.

## objetivo

mesma UI, mesmo layout, mesmas fontes, mesmos pixels em macOS, Windows,
Linux, browser e mobile, na medida em que isso serve a previsibilidade,
teste e reprodutibilidade. e, onde a igualdade não acontece, o plev sabe
dizer se a diferença foi pedida pelo produto ou apareceu por divergência
silenciosa entre plataformas.

## ligação com a tese

seção 23. o mantra das anotações: "one pixel-identical frame on every
target", sabendo que fonts, drivers, GPU, DPI, text rasterization e color
management tornam isso infernal. e a correção da mentoria: determinismo
serve previsibilidade, não vira dogma.

## estado atual no código

- contratos de cor: srgb decodificado uma vez ao entrar na GPU, codificado
  uma vez na escrita da surface (`surface_render_view`, `to_linear_array`);
  mix de cor em espaço linear pré-multiplicado nos shaders.
- raster de glifo determinístico, só fontes embutidas (ADR
  `glyph-raster-identity-and-atlas-isolation`,
  `rust/engine/src/drawing/text/fonts.rs`).
- web declarado igual ao desktop; a validação visual do wasm está em
  andamento (task-20); android e ios em andamento (tasks 18 e 19); linux e
  windows pendentes (`docs/ATLAS.toml`, targets).
- depois da 66: render headless no desktop.

## escopo

1. harness de captura por alvo, sempre da mesma página, tema, viewport e
   escala: desktop pelo headless (66); web por navegador com WebGPU
   controlado por automação sobre o build do trunk; iOS pelo simulador
   (`tool/tasks/mobileapp.sh ios-run` já tira screenshot); Android pelo emulador.
2. comparação com classificação: igual em bit; diferente por decisão do
   produto (safe area, escala, chrome de plataforma) declarada numa tabela
   versionada; diferente por divergência silenciosa (o que precisa de
   causa).
3. investigação das divergências silenciosas por classe: antialiasing de
   borda em SDF (derivadas e `fwidth` por driver), dithering de gradiente,
   color management (espaço de cor da layer Metal, displays P3), escala
   fracionária de DPI, raster de texto.
4. ADR definindo o que "idêntico" quer dizer por classe: bit a bit onde é
   possível, diferença limitada e medida (delta de cor por pixel, número de
   pixels) onde não é, e a lista do que é decisão de produto.
5. o CI roda a comparação nos alvos que o CI consegue rodar e registra os
   outros como manuais.

## fora de escopo

bring-up de linux e windows (tarefa própria quando o alvo for priorizado);
fontes de sistema (decisão da task-74).

## dependências

66 integrada. consome os resultados das tasks 18, 19 e 20 sem duplicá-las.

## superfície sob responsabilidade

o harness de captura e comparação (exemplo ou binário), scripts de
captura em `script/`, correções de shader quando a causa for encontrada
(com aprovação, porque shader é superfície compartilhada), ADR
[`0064-what-identical-means.md`](../../adr/workflows/0064-what-identical-means.md).

## interfaces que consome

headless (66), os scripts de mobile e web existentes.

## interfaces que pode alterar

shaders e caminho de cor só com a causa demonstrada e aprovação.

## critérios de aceitação

- relatório de divergência entre desktop e web e entre desktop e iOS
  simulador para todas as páginas do showcase, com cada diferença
  classificada.
- toda divergência silenciosa encontrada tem causa escrita ou está
  registrada como aberta.
- ADR com a definição de idêntico.
- `./tool/tasks/fullcheck.sh` verde.

## testes esperados

o comparador (classificação com imagens sintéticas), e a comparação real
como job.

## riscos e perguntas abertas

- driver de GPU de CI (software) diferente do de desenvolvimento.
- automação de navegador com WebGPU em CI pode não estar disponível;
  nesse caso a captura web fica manual e registrada.

## commits esperados

1. `task-77: comparador com classificação`
2. `task-77: captura por alvo (desktop, web, iOS, Android)`
3. `task-77: relatório e tabela de diferenças pedidas`
4. `task-77: <causa>` por divergência resolvida
5. `task-77: ADR do que idêntico quer dizer`

## pare e reporte se

- a correção de uma divergência exigir mudar o contrato de cor;
- a causa estiver no driver e não houver contorno no plev.

## Phases

1. [phase 1 (task-77a): the probe](phase-1a-probe.md): the probe of the targets, the desktop capture against itself (1312 equal frames), the divergences this machine reproduces and the phase handoff.
2. [handoff of phase 1b (task-77b): the blur follows the scale](phase-1b-blur.md): the blur-by-scale fix, the three options measured, the chosen one, the tests, the cost before and after, and the recaptured frames.
3. [closure (2026-09-25, orchestrator)](closing.md): the closure against the acceptance criterion, with web, iOS and android, the silenced differences and their causes, and the open items.
