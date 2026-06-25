---
title: blog zola (estrutura e plano)
status: aguardando revisao
tags: [blog, zola, building-plev, seo]
destino: kdb/caranguejovermelho/blog
---

# blog zola

## o que ja existe (confirmado no repo)

- blog zola completo, tema base welpo/tabi, customizado como tema "Brenner".
- `config.toml`: `base_url = https://brennercruvinel.com`, author Brenner
  Cruvinel, taxonomia unica `tags`, busca fuse, pwa, comments mastodon
  (vmst.io), goatcounter, paletas wcag aa, katex per-page.
- seo de entidade ja embutido no tema: json-ld @graph, `organization_knows_about`,
  `sameas` derivado das socials. isso casa direto com o capitulo P9 do livro.
- pastas por ano ja existem: 2009..2026, cada uma com `_index.md`. ha um adr do
  proprio blog para isso: `blog/kdb/adr/0006-blog-year-folders-path-override.md`.
- `ignored_content = ["blog/*/old/**"]`: rascunho vai em `blog/<ano>/old/`.
- ja existem posts reais (whisper cru, titulos truncados) que sao fragmentos da
  historia perdida e precisam de reconstrucao brennerwritter:
  - 2022: `designparatdah.md`, `tdahnobrasilmundo.md`, `coreseed.md`,
    `coreseess.md`, `pocprompts.md`, `aplicazmachine.md`,
    `triagraemparanerudiovegrnecias.md`
  - 2023: `findings.md`, `mrc-paper-scale-compression-fractal-dimension.md`,
    `relatoriohoff.md`

## decisao de idioma (ver 12-decisoes-pendentes)

`default_language = "en"` no config, mas voce escreve em pt-br e o tema e
multilingue. precisa decidir: blog primario em pt-br (e mudar default), ou pt-br
como lingua adicional com en de capa. isso muda o frontmatter de todo post.

## o plano de conteudo (2021 a 2026)

todo post de jornada leva a tag `building plev`, mais tags de segmentacao por
assunto (rust, wasm, gpu, wgpu, animation, lottie, seo, neurodiversidade,
benchmark, etc). distribuir os posts pelos anos conforme a linha do tempo real
reconstruida pelo historiador (fase 2), nao por ano arbitrario.

### o primeiro post

a origem: escrever um livro tecnico quando a aurora nasceu. legado, ensinar rust
a ela, por que rust sobrevive as ai/llms, e o inicio da jornada de construir uma
engine tao robusta quanto um flutter, cross-device. humilde, creditando quem
inspirou. esse post e o portao de entrada da serie building plev e o eco do
capitulo 0 do livro.

### a serie building plev

cada marco real do plev vira post, construido a partir de diff + adr +
conversa (claude2026) + benchmark. exemplos de fios:

- o primeiro quad com alpha blending na gpu.
- atlas de glifos com etagere + lru, e por que shaping e caro.
- dirty tracking por hash: como uma layer limpa vira zero trabalho.
- o signal system inspirado em leptos, e o bug do observer que o RAII guard matou.
- o spring solver analitico que consertou o jitter dependente de frame-rate.
- lottie virando .monster: descobrir deltas, dedupe de payload, o codec binario.
- a11y que some por padrao no gpu-first, e por que accesskit entrou na fundacao.
- seo de wasm: por que o crawler nao ve o que so existe depois do js rodar.

### reconstrucao dos posts perdidos

os posts cru de 2022/2023 sao ouro de historia, mas estao truncados e com tells
de whisper. parte do material de origem chega caotico ou fragmentado. um agente
reconstroi cada post com brennerwritter, preservando o sinal original, deixando
legivel, sem inventar e sem expor dado pessoal. o foco e a tecnica e a jornada,
nao a vida privada.

## seo e descoberta (casando com P9)

o tema ja entrega @graph json-ld por pagina. o plano:

- preencher `organization_knows_about` e `author_sameas` reais (github, mastodon,
  e wikidata se houver).
- cada post de serie linka o diff/commit/adr correspondente (rastreabilidade que
  tambem e sinal de qualidade para citacao por ai, ver P9).
- manter gptbot, oai-searchbot, perplexitybot, claudebot, google-extended, ccbot
  liberados no robots (a menos que voce queira o contrario).
- o capitulo P9 do livro usa este blog como o estudo de caso pratico.

## como os posts sao produzidos (fase 6)

mesmo fluxo do livro: leitor levanta a ancora (diff/adr/conversa), escritor faz o
sumario do passo 0 para mim, eu aprovo o recorte, ele escreve com brennerwritter
(perfil blog), verificador roda os taboos e confere links, eu valido. build do
zola tem que passar (`zola build`) antes de considerar um lote pronto.
