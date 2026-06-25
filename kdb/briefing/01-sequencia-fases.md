---
title: sequencia e fases macro
status: aguardando revisao
tags: [fases, gate, sequencia]
---

# sequencia e fases

## o gate de aprovacao (onde estamos)

```
[fase 0] recon + briefing  <- AGORA (eu escrevo, sem disparar frota)
   |
   v
[gate]   brenner revisa o briefing  <- voce. corrige, corta, aprova.
   |
   v
[fase 1+] frota de agentes nos workflows  <- so depois do aprovado
```

estou na fase 0. nao avanco para a frota sem o seu ok explicito. quando voce
aprovar, eu disparo as fases abaixo, cada uma como um ou mais workflows, com eu
validando entre elas (nao tudo no final).

## as fases macro (pos-aprovacao)

ordem proposta, com dependencia real. detalhe operacional de cada uma esta no
arquivo citado.

| fase | nome | entrega | depende de | detalhe |
|------|------|---------|------------|---------|
| 1 | cocriacao: ingest claude2026 | copia read-only + extracao estruturada + dataset ml hygienizado | nada | `08-cocriacaoclaudinho.md` |
| 2 | historiador git | linha do tempo reconstruida (phi -> plev), correlacao diff/adr/conversa, commits-de-historia (com seu consentimento sobre o metodo) | fase 1 (parcial) | `11-frota-agentes.md` |
| 3 | corpus de pesquisa | a pesquisa colada limpa, organizada em refs/, deduplicada, com yaml | nada | `04-corpus-pesquisa.md` |
| 4 | benchmarks | jupyter notebooks testados + resumos + graficos para o livro e possivel paper | engine ja tem criterion | `09-benchmarks.md` |
| 5 | livro | as partes/capitulos do caranguejo vermelho, ancorados em adr/crate/diff/bench | fases 2,3,4 | `05-livro.md` |
| 6 | blog zola | posts por ano, tag building plug, reconstrucao dos posts perdidos, primeiro post aurora | fases 2,3 | `06-blog.md` |
| 7 | tutoriais | tutoriais executaveis contra a engine | engine | `07-tutoriais.md` |
| 8 | experimento mon | as 30 paginas abertas: lottie/swf/flash/motion-ui/design-system | crates lot/monster/parser + notes.md | `10-experimento-mon.md` |

essas fases nao sao 100% sequenciais. 1, 3 e 4 podem rodar cedo e em paralelo.
2 depende de 1. 5 e 6 consomem o resultado das anteriores. 8 e o trabalho vivo,
fica aberto.

## principio de validacao entre fases

cada fase termina com um artefato que eu reviso antes de liberar a proxima.
nenhuma fase "entrega no final". se uma entrega vier rasa (agente fez de
qualquer jeito), volta com o hook apontando o que faltou, nao segue. isso e o
que voce pediu: orquestrador e validador, entrega por entrega.

## o que explicitamente NAO faco agora (fase 0)

- nao copio o claude2026 ainda (e 17gb, 68k arquivos; e fase 1).
- nao crio commits de historia reconstruida ainda (precisa do seu ok sobre o
  metodo, ver `12-decisoes-pendentes.md`).
- nao escrevo capitulo de livro nem post de blog ainda.
- nao disparo nenhum workflow de frota.
- nao toco em nada fora da arvore do projeto sem perguntar.
