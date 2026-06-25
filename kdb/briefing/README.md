---
title: briefing mestre, caranguejo vermelho
project: plev / caranguejovermelho
audience: [claude-orquestrador, brenner, subagentes]
status: aguardando revisao do brenner
created: 2026-06-25
domain: orchestration-briefing
tags: [briefing, livro, blog, cocriacao, benchmarks, workflows, brennerwritter]
---

# briefing mestre

este diretorio e o hook de orquestracao. ele existe por uma razao: a conversa
que originou tudo isso e gigante (multiplas pesquisas coladas, transcricao
whisper, notas soltas, mudanca de assunto no meio). quando o contexto ficar
longo e comecar a degradar, eu (e qualquer agente) leio este briefing primeiro
e recupero o norte sem reler a conversa inteira.

regra de ouro: este briefing nao e codigo de aplicacao nem conteudo publicado.
e material interno de planejamento. segue a casa (sem emoji, sem em dash,
minuscula-tendente, portugues), mas nao precisa passar pelo benchmark completo
do brennerwritter. o conteudo que VAI ser publicado (livro, blog, tutoriais) sim.

## a sequencia (nao pular)

1. agora: eu construo este briefing completo, ancorado no repo real.
2. voce revisa. corrige rumo, corta, adiciona.
3. so depois da sua aprovacao eu ativo o exercito de subagentes nos workflows.

nada de disparar a frota antes do item 2. o briefing existe justamente para
voce poder revisar barato antes de gastar muito.

## mapa do briefing

| arquivo | o que cobre |
|---------|-------------|
| `00-visao.md` | o porque, a historia (aurora, legado, humildade, creditos), os nomes phi -> plev -> caranguejovermelho |
| `01-sequencia-fases.md` | as fases macro, o gate de aprovacao, o que nao disparar ainda |
| `02-guardrails.md` | guardrails universais para mim e para todo agente (no-commit, trash, read-only, historia git, cloud, hooks) |
| `03-brennerwritter.md` | o contrato de escrita para embutir em TODO agente escritor (mantras, 24 taboos, perfis, pipeline) |
| `04-corpus-pesquisa.md` | inventario da pesquisa que voce colou + onde cada bloco entra + o que falta pesquisar |
| `05-livro.md` | estrutura do livro caranguejo vermelho (569-963 pag), capitulos ancorados em adr/crate/diff, as 30 paginas abertas |
| `06-blog.md` | o blog zola real, anos, tags, tag building plev, primeiro post (aurora), reconstrucao dos posts perdidos, seo @graph |
| `07-tutoriais.md` | os tutoriais (build-against-plev, web, mobile, lottie/monster, parser) |
| `08-cocriacaoclaudinho.md` | plano dos 8 agentes para o volume claude2026, sanitizacao, yaml header, dataset ml |
| `09-benchmarks.md` | o capitulo de benchmark, os jupyter notebooks, a regra de testar o notebook antes de entregar |
| `10-experimento-mon.md` | as 30 paginas abertas: lot/monster, swf/flash, lottie, motion ui, design system universal |
| `11-frota-agentes.md` | desenho dos workflows pos-aprovacao: fases, papeis, poc, hooks, checklists, execucao em cloud |
| `12-decisoes-pendentes.md` | o que eu preciso que voce decida antes de eu disparar |
| `hooks/` | templates de hook/checklist por tipo de agente (escritor, extrator, benchmark, historiador) |

## estado do recon (o que ja foi confirmado no repo, 2026-06-25)

- `kdb/caranguejovermelho/blog`: blog zola completo (tema base welpo/tabi),
  `base_url = brennercruvinel.com`, taxonomia `tags`, json-ld @graph de entidade
  ja implementado no tema, pwa, comments mastodon, goatcounter. anos 2009..2026
  ja existem como pastas com `_index.md`. ja ha posts reais (whisper cru) em
  2022 e 2023 que precisam de reconstrucao.
- `kdb/caranguejovermelho/{livro,tutoriais}`: vazios, a preencher.
- `kdb/cocriacaoclaudinho`: vazio, destino da copia + extracao do claude2026.
- `/Volumes/500G-SSD/claude2026`: montado, 17gb, 68.029 arquivos
  (19.873 js, 6.703 json, 5.966 jsonl, 2.664 md, ...). historico phi presente.
  READ-ONLY: so copiar, nunca editar.
- engine plev: v0.3, 34+ tasks, 404 testes, ~15k loc core, 50 adrs indexados em
  `kdb/adr/index.md`, benchmarks criterion (m4), paper arxiv (outline+draft).
- crates do experimento mon: `lot` (lottie importer), `monster` (codec binario
  .monster), `parser` (transpiler poc). `notes.md` na raiz e o braindump cru do
  estudo motion-ui / swf / design tokens / aria apg / areweguiyet.
- skill `brennerwritter`: real, em `~/.claude/skills/brennerwritter/`
  (symlink para `templates/.../brennerwritter`). refs em `refs/voice.md`,
  `refs/taboo.md`, `refs/guide.md`.
