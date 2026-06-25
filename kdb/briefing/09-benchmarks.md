---
title: benchmarks e notebooks
status: aguardando revisao
tags: [benchmark, jupyter, ciencia, paper]
---

# benchmarks e notebooks

## o que ja existe

a engine ja tem benchmarks criterion (harness false) em varias crates, rodados
no m4 mac, documentados em `kdb/adr/benchmark-results.md`:

- push_rects: 159-222 milhoes de rects/s
- dirty tracking: 3.3us / 1000 layers
- tessellation: 1.5-3.7us / shape
- signals: 67ns / cycle

benches: engine (scene_build), rope (edit), monster (codec), lot (convert),
parser (transpile). e o paper arxiv ja tem outline e draft
(`arxiv-paper-outline.md`, `arxiv-paper-draft.md`).

## o que voce quer

um capitulo grande de benchmark, gerado por multiplos agentes, com um jupyter
notebook por benchmark importante. cada notebook serve para tres coisas ao mesmo
tempo: virar resumo no livro, gerar grafico/chart, e ser base para um possivel
paper cientifico nosso no futuro.

## regra dura: testar o notebook antes de entregar

agente em paralelo tende a entregar notebook que nao roda. entao o hook do
benchmark (`hooks/HOOK-benchmark.md`) exige:

- o notebook executa do topo ao fim sem erro (kernel limpo).
- toda celula que afirma um numero mostra como o numero foi obtido (comando
  criterion, csv de saida, ou medicao real), nunca um numero digitado a mao.
- o grafico gera a partir do dado, nao e imagem colada.
- o notebook declara o hardware, o so, a versao do rust, e a versao da crate.
- divergencia entre rodadas fica registrada, nao mascarada.

## estrutura proposta

| notebook | mede | crate/bench |
|----------|------|-------------|
| nb-scene-build | custo de montar a scene | engine/scene_build |
| nb-rect-throughput | rects/s, o numero de capa | engine (push_rects) |
| nb-dirty-tracking | custo do dirty tracking por layer | engine |
| nb-rope-edit | build + insert/delete roundtrip | rope/edit |
| nb-monster-codec | encode/decode/optimize | monster/codec |
| nb-lot-convert | lottie json -> .monster | lot/convert |
| nb-parser-transpile | transpile gpui end to end | parser/transpile |
| nb-tessellation | us por shape (lyon) | engine |
| nb-signals | ns por cycle | engine |

## diretrizes de alto nivel (para o livro e para o paper)

- comparar com terceiros (P5: ripgrep, deno/bun, vector) so como contraste de
  metodo, deixando claro que cenarios diferentes nao sao comparaveis. nunca
  vender o numero do plev contra o numero de outro projeto medido em outra maquina.
- honestidade de marketing vs ganho real: dizer onde o ganho e grande e
  comprovado e onde e marginal (so seguranca de memoria). isso e o tom do livro.
- cada notebook vira uma figura e um paragrafo no capitulo 5 do livro, e uma
  secao de evaluation no paper.

## execucao (fase 4)

um agente por notebook, em background/cloud. cada um roda o bench, gera o dado,
escreve o notebook, executa o notebook do zero, e so entao entrega. um agente
verificador re-executa um subconjunto para confirmar reprodutibilidade. eu valido
antes de o capitulo 5 consumir os resultados.
