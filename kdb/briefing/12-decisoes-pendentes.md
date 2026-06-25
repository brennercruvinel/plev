---
title: decisoes pendentes (preciso de voce antes de disparar)
status: aguardando revisao
tags: [decisoes, gate]
---

# decisoes pendentes

## resolvido (2026-06-25)

- historia git: reconstrucao honesta. mas SEM poluir commits e docs com
  meta-comentario de reconstrucao/inferido/pessoal; o tree fica limpo, a
  metodologia mora numa nota unica. e um livro. (item 1 fechado)
- idioma: pt-br primario no blog e no livro. (item 2 fechado)
- inicio: paralelo (cocriacao + corpus + benchmarks em background). (item 3 fechado)
- privacidade: nao citar dado pessoal sensivel em lugar nenhum. inputs caoticos
  sao reconstruidos com brennerwritter para ficarem legiveis, foco na tecnica e
  na jornada. moldura medica/pessoal removida dos arquivos do briefing. (item 4 fechado)

## ainda aberto (preciso antes de disparar)

estas travam a frota. respondo o que voce decidir.

## 1. historia git reconstruida (sensivel)

quase 4 anos de historico se perderam (forgejo deletado). para o periodo perdido,
qual metodo:

- (a) reconstrucao honesta: uma branch/arquivo de timeline documentada, commits
  claramente marcados como reconstrucao, datas documentadas como inferidas. nao
  finge ser historico real. (recomendado)
- (b) commits com datas retroativas que passam por reais. eu nao faco isso sem
  voce assumir explicitamente, porque e enganoso e pode quebrar confianca futura.
- (c) nao tocar em git history. a reconstrucao vive so como docs/timeline no kdb.

## 2. idioma do blog e do livro

o livro e pt-br (suas instrucoes). o blog tem `default_language = "en"` no config
mas voce escreve em pt-br. opcoes:

- (a) pt-br primario no blog (mudo o default para pt, en vira adicional). (recomendado)
- (b) pt-br como lingua adicional, en de capa (mais trabalho de frontmatter).
- (c) bilingue completo desde o inicio (caro, dobra o volume de escrita).

## 3. por onde comecar depois do aprovado

- (a) paralelo: cocriacao (fase 1) + corpus (fase 3) + benchmarks (fase 4) ao
  mesmo tempo, em background, e o esqueleto do livro em seguida. (recomendado)
- (b) serial: cocriacao primeiro (ela alimenta o historiador e o livro), depois o
  resto.
- (c) outra ordem que voce preferir.

## 4. privacidade (neurodiversidade e dado medico)

regra que proponho: conteudo de neurodiversidade que voce escreveu para publicar
(os posts de 2022) permanece, reconstruido com cuidado; detalhe medico sensivel
(meltdown, diagnostico) nao entra no dataset publico nem no material publicado
sem seu ok explicito, caso a caso. confirma essa linha?

## 5. clarificacoes

- "orley" = O'Reilly (a editora). resolvido. nao era um exit em negociacao; eram
  livros tecnicos best seller deles, fracos, usados como contraste no capitulo
  "por que este livro existe" (P7/P8). os links estao em `04-corpus-pesquisa.md`.
- marketplace e patches para download: ainda aberto. e escopo agora ou depois?
  por enquanto a ideia fica anexada a fase 1 (experimentos com erro viram patch),
  sem construir a plataforma.

## 6. execucao em cloud / remoto (resolvido)

sem execucao remota disponivel. fica local, multiplos agentes em paralelo, em
background, para nao travar a maquina. confirmado.

## 7. escala e orcamento

ultracode esta ligado (sem economizar token). quer que eu trate isso como teto
real (frota grande, loop ate secar), ou prefere um teto por fase para eu te
mostrar resultado antes de escalar? minha sugestao: comecar com a fase 1 e a 3
em escala media, voce ve a qualidade da primeira leva, e ai escalamos.
