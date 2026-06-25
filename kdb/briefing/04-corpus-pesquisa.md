---
title: corpus de pesquisa (inventario e roteamento)
status: aguardando revisao
tags: [pesquisa, refs, roteamento]
---

# corpus de pesquisa

voce colou muita pesquisa (saidas de perplexity, listas, um documento longo de
seo/json-ld). nada disso pode entrar no livro como esta: e bruto, tem tells de
llm, tem links a validar, tem dado a conferir. este arquivo inventaria o que
chegou e roteia cada bloco. a limpeza e a fase 3.

## blocos recebidos

| id | bloco | volume aprox | destino primario |
|----|-------|--------------|------------------|
| P1 | 50+ pessoas rust + wasm (nucleo, frameworks, educadores) | 2 tabelas grandes | refs/ + apendice "ecossistema" do livro + creditos |
| P2 | 50+ projetos de grafos/charts/visualizacao em wasm | 1 tabela grande | refs/ + capitulo viz/wasm |
| P3 | conexao pessoa<->projeto + contexto brasil | 2 tabelas | refs/ + apendice + secao brasil |
| P4 | 369+ aplicacoes rust (top 100, 100-200, por dominio) | varias tabelas | refs/ + capitulo "panorama rust" + tabelas do livro |
| P5 | benchmarks de terceiros (ripgrep, deno/bun/node, vector) | tabelas | capitulo benchmark (como contraste com os NOSSOS) |
| P6 | uso em producao (github, cloudflare, aws, vercel) | prosa | capitulo "rust em producao" |
| P7 | livros rust + wasm (mundo + brasil) | tabelas | capitulo "por que este livro existe" (lacuna editorial) |
| P8 | mercado editorial (top 20 global/brasil, infantil, autores) | tabelas | mesmo capitulo, framing diadico (crianca -> ml engineer) |
| P9 | seo + json-ld + @graph + nlweb + llms.txt + faqpage + robots | doc longo | capitulo "seo, wasm e desafios" + ja casa com o @graph do blog |
| P10 | lottie / swf / flash / motion ui / design system universal | notes.md + trechos | experimento mon, ver `10-experimento-mon.md` |
| P11 | complemento (repeticao de P1..P3 + listas de ui libs) | redundante | dedupe contra P1..P3, extrair so as ui libs novas |

## regras de tratamento (fase 3)

- deduplicar: P11 repete P1..P3. um agente consolida e marca o que e novo (as
  listas grandes de ui libs: shadcn, radix, mui, swiftui, compose, flutter, e o
  bloco rust ui: egui, slint, iced, dioxus, floem, freya, makepad, xilem, vizia,
  zed, vello, taffy, cosmic-text, parley, accesskit, tauri, gpui-component).
- validar links antes de citar. link quebrado ou homonimo nao entra. quando nao
  der para confirmar, marcar como "nao confirmado", nunca inventar.
- nao inventar numero de benchmark nem de stars. so usar dado claramente
  apresentado por fonte confiavel, com link. divergencia entre fontes fica
  explicita (cenarios diferentes), nao consolidada como se fosse comparavel.
- cada doc de refs/ ganha yaml header com data, tags, fonte, status de validacao.
- aplicar brennerwritter na prosa de analise (P5, P6, P9 tem muito tell de llm).

## onde isso vira capitulo

- panorama do ecossistema rust (P4, P6): contexto, nao enchimento. o livro usa
  isso para situar o plev no mapa, com honestidade sobre onde rust ganha de fato
  (cli rapida, runtimes, infra) e onde o ganho e marginal vs c/go (so seguranca).
- viz e wasm (P2, P3): o cluster de visualizacao em wasm, e a lacuna (nao ha
  equivalente maduro a d3-force/networkx em rust+wasm). conecta com o que o plev
  faz e nao faz.
- por que este livro existe (P7, P8): a lacuna editorial. so um livro brasileiro
  especifico de wasm+rust (desmistificando webassembly, raphael amorim). quase
  nada para adolescente de 13-17. o caranguejo vermelho mira exatamente esse vao.
- seo, wasm e desafios (P9): o capitulo tecnico de descoberta por ai. casa com o
  fato de o blog ja ter @graph json-ld de entidade implementado no tema. mostra,
  na pratica, o problema de crawlability de wasm client-side e a solucao
  (ssr/pre-render, o caso rust servidor com serde).

## o que falta pesquisar (o brenner pediu)

- aprofundar devs brasileiros de rust/wasm (alem de japaric), com enfase em viz.
- runtimes de motion/lottie/rive (dotlottie, rive-rs, velato) para o experimento mon.
- design tokens cross-platform (style-dictionary, tokens-studio, open-props) e o
  aria apg como grafo de comportamento neutro (ja apontado no notes.md).
- confirmar metricas (stars, downloads) na data de publicacao, nunca chutar.

## material adicional recebido (best sellers fracos, para contraste)

o brenner apontou livros tecnicos best seller (O'Reilly e afins) que vendem bem
mas sao fracos, como contraste para o capitulo P7/P8 ("por que este livro
existe"). a fase 3 resolve titulo, autor, editora e metricas de cada um, e
escreve por que vende e onde e fraco, sem chutar.

- https://www.amazon.com.br/dp/8575225634
- https://www.amazon.com.br/dp/132854639X
- https://www.amazon.com.br/dp/B0DG37XVR6
- https://www.amazon.com.br/dp/B0CM8TRWK3
- https://www.amazon.com.br/dp/B0FG7NW67J
- https://www.amazon.com.br/dp/8550815624
