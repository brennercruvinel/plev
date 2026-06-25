---
title: livro caranguejo vermelho (estrutura)
status: aguardando revisao
tags: [livro, estrutura, capitulos]
destino: kdb/caranguejovermelho/livro
---

# livro caranguejo vermelho

## parametros fixos

- extensao: 569 a 963 paginas. ~30 paginas deixadas abertas para o experimento
  mon (lottie/swf/flash/motion ui), que e trabalho vivo.
- diadico: cada capitulo abre acessivel (uma crianca de 13 anos acompanha) e
  aprofunda ate doer (um engenheiro de ml nao acha raso). a tecnica e camada: o
  conceito primeiro em linguagem humana, depois o codigo real, depois o porque
  arquitetural com o adr e o numero do benchmark.
- ancoragem: nenhuma afirmacao solta. cada capitulo linka pelo menos um diff/
  commit, um adr de `kdb/adr/`, e quando couber um numero de benchmark. e a
  diferenca entre um livro de opiniao e um livro que aconteceu.
- create, never copy. revolucionar, nao portar. o livro credita as fundacoes
  (makepad, zed/gpui, bevy, flutter, linebender) e explica o que foi feito
  diferente e por que.
- voz: brennerwritter, perfil "artigo cientifico/timeline" para os capitulos
  tecnicos, perfil "blog" para os capitulos de jornada. primeira pessoa onde
  couber, humildade sempre.

## estrutura proposta (partes -> capitulos)

| parte | titulo | conteudo | ancora real | pag aprox |
|-------|--------|----------|-------------|-----------|
| 0 | origem | aurora, legado, por que rust sobrevive as ai, humildade e creditos | `00-visao.md`, claude2026 (historia phi) | 30-50 |
| 1 | rust de verdade | ownership, borrow, traits, edition 2024, async, ensinados pelo codigo real do plev | crates/engine, rust-conventions | 90-150 |
| 2 | a engine, por dentro | gpu, compositor, text, layout, input, animation, signal, perf, view, builder | ~50 adrs em `kdb/adr/index.md` | 120-200 |
| 3 | um codebase, varios mundos | macos/metal, web/webgpu, android, ios, wasm, hidpi, os asteriscos honestos | adr render-into-srgb, wasm-webgpu-validation, mobile-specifics | 60-110 |
| 4 | o experimento mon | lottie -> .monster, swf/flash, motion ui, design system universal | crates lot/monster/parser, notes.md | ~30 (abertas) |
| 5 | medir, nao achar | benchmarks criterion, jupyter notebooks, o caminho do paper arxiv | benchmark-results, arxiv-paper-draft, `09-benchmarks.md` | 50-90 |
| 6 | seo, wasm e desafios | descoberta por ai, json-ld @graph, crawlability de wasm, ssr/pre-render | P9, @graph do blog | 40-80 |
| 7 | onde isto se encaixa | panorama rust, ecossistema, lacuna editorial, por que este livro | P4, P7, P8 | 40-70 |
| A | apendices | pessoas e projetos (P1-P3), glossario, indice de diffs/commits/adrs | refs/ | 40-90 |

soma a faixa-alvo com folga e respeita as 30 paginas abertas da parte 4.

## como cada capitulo e construido (na fase 5)

1. um agente leitor levanta o material da ancora (adr + codigo + diff + bench) e
   devolve um digest estruturado com file:line.
2. um agente escritor (hook do escritor + brennerwritter) entrega primeiro o
   sumario do passo 0 para mim. eu reviso o recorte e o tom.
3. so depois ele escreve o corpo, com os trechos de codigo reais (compilaveis,
   conferidos contra a versao do cargo.toml), os links de diff/commit/adr, e os
   numeros de benchmark.
4. um agente verificador adversarial confere: a afirmacao tecnica bate com o
   codigo? o link existe? o numero veio de fonte? rodou o benchmark de taboos?
5. eu valido a entrega antes de liberar o proximo capitulo. nada acumula para o
   final.

## links e rastreabilidade

cada capitulo termina com um bloco "rastros": os commits, diffs, adrs e
benchmarks que sustentam o capitulo, mais o post de blog correspondente (a serie
building plev). o leitor pode ir do paragrafo ao diff. esse e o diferencial do
livro: tudo verificavel.
