---
id: task-77
title: "task-77: determinismo visual"
project: plev
type: tarefa
status: concluido
date: 2026-09-25
date_basis: cabecalho
topics: [determinismo, renderizacao, plataforma, mobile, web-wasm]
relations: [adr-0024, adr-0064, task-18, task-19, task-20, task-66, task-74]
evidence: [2dd2bc0d, c779fca6, 65d45235, 8af26af5, 35a47b26, a69cd464, 7127f65a, f6c688b5, adbf948b]
source: docs/log/steps/task-77-visual-determinism.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# task-77: determinismo visual

Registra a tarefa que mediu o determinismo visual entre desktop, web, iOS e android e classificou cada diferença entre alvos. O desktop saiu igual em bit consigo mesmo nas 82 páginas da galeria, em 4 temas e 4 escalas (1312 frames); a fase 1 achou que o blur não segue a escala, e a fase 1b corrigiu isso mantendo a escala 1 igual byte a byte à base. No fechamento, web teve 79 páginas iguais em todo byte e 3 caladas, iOS teve 78 com diferença pedida na barra de status e 4 caladas, todas com causa escrita, e android ficou a no máximo 2/255 do desktop; o `colorspace` da layer metal, a janela contra o headless e o job de CI ficaram abertos.

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
[`0064-what-identical-means.md`](../decisoes/processos/0064-what-identical-means.md).

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

## Fases

1. [fase 1 (task-77a): a sonda](task-77-visual-determinism/fase-1a-sonda.md): a sonda dos alvos, a captura do desktop contra ele mesmo (1312 frames iguais), as divergências que esta máquina reproduz e o handoff da fase.
2. [handoff da fase 1b (task-77b): o blur segue a escala](task-77-visual-determinism/fase-1b-blur.md): a correção do blur por escala, as três opções medidas, a escolhida, os testes, o custo antes e depois e a captura refeita.
3. [fechamento (orquestrador, 2026-09-25)](task-77-visual-determinism/fase-fechamento.md): o fechamento pelo critério de aceite, com web, iOS e android, as diferenças caladas e suas causas, e as pendências.
