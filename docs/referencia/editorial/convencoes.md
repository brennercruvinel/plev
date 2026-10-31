---
id: convencoes
title: Convenções do acervo
project: plev
type: manual
status: ativo
date: 2026-10-30
date_basis: registro-editorial
topics: [editorial, documentacao]
relations: [modelo-decisao]
source: docs/referencia/editorial/convencoes.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# Convenções do acervo

O acervo guarda a trajetória do plev desde 2021: o que foi decidido, pesquisado,
medido e aprendido, e onde isso está no código. Esta página é o contrato de
forma de toda página em `docs/`. O `tool/tasks/docbase.py check` confere a
parte mecânica dele no gate.

## Onde cada coisa mora

| Pasta | O que guarda | Identidade |
|---|---|---|
| `historia/` | linha do tempo, programas, diário, changelog, retratos de um período | nome do assunto |
| `tarefas/` | uma página por task: plano, execução, entrega e pendências | `task-NN` |
| `decisoes/<tema>/` | ADRs, numerados numa sequência só para todos os temas | `adr-NNNN` |
| `pesquisas/` | estudos de referência, comparações, a tese, o artigo | nome do assunto |
| `experimentos/` | método, ambiente, medições e resultados | nome do assunto |
| `aprendizados/` | o que um problema ou uma entrega ensinou | nome do assunto |
| `referencia/` | `arquitetura/` de cada subsistema, `operacao/` do repositório, `plataformas/`, `editorial/` | nome do assunto |
| `imagens/` | imagens que as páginas mostram | caminho |

Os temas das decisões são `geracao`, `importacao`, `interface`, `gravacao`,
`renderizacao`, `semantica`, `estrutura` e `processos`. A numeração de task e
de ADR é identidade, não cronologia: a ordem no tempo vem do campo `date`.

Uma task cujo registro passa de umas 600 linhas vira uma pasta com o mesmo
nome da página: a página de entrada (`tarefas/task-69-….md`) diz o objetivo, o
resultado e lista as fases; cada fase é uma página em `tarefas/task-69-…/`, com
`type: secao` e `parent: task-69`.

## O cabeçalho

YAML com um `chave: valor` por linha. Um valor é uma palavra, uma string entre
aspas duplas, `null` ou uma lista na mesma linha (`[a, "b c"]`).

| Campo | O que diz |
|---|---|
| `id` | identidade estável, única no acervo; as relações usam ela |
| `title` | o assunto, dito como o leitor procura |
| `project` | sempre `plev` |
| `type` | `tarefa`, `decisao`, `pesquisa`, `experimento`, `aprendizado`, `referencia`, `manual`, `registro`, `programa`, `tese`, `rascunho`, `modelo`, `secao`, `indice` |
| `parent` | só numa `secao`: o `id` da página de entrada |
| `status` | tarefa: `pendente`, `em-andamento`, `concluido`, `substituido`; decisão: `proposto`, `aceito`, `substituido`; os demais: `referencia`, `ativo`, `rascunho` |
| `date` | quando aconteceu o que a página registra, com a precisão que se tem: `2021`, `2021-03` ou `2021-03-06`; `null` quando não se sabe |
| `date_basis` | de onde veio a data: `cabecalho` (o cabeçalho de origem), `texto` (uma data escrita no corpo), `git` (um commit), `reconstruida` (inferida de várias fontes), `registro-editorial`, `sem-data` |
| `topics` | assuntos em kebab-case, poucos e reaproveitados entre páginas |
| `relations` | `id`s de páginas ligadas: dependências, a decisão que uma task tomou, a pesquisa que a motivou |
| `evidence` | commits (sha curto) que sustentam o que a página afirma |
| `source` | caminho de origem (ou lista) na revisão `source_revision`; `reconstrucao` quando a página foi escrita a partir de código e commits |
| `source_revision` | o commit em que a origem foi lida |
| `revised` | a data da última revisão editorial |
| `review` | `pendente` (migrada, ainda não lida inteira), `revisado` (lida inteira e conferida), `gerado` (índice) |

Um cabeçalho de origem com data revista depois do fato (o commit `704412f1`
reescreveu datas de 2026 para 2021) fica com essa data e `date_basis:
cabecalho`; [cronologia e fontes](../../historia/cronologia-e-fontes.md)
explica o caso. Uma data só muda quando outra fonte a contradiz, e então
`date_basis` diz qual.

## O corpo

- Abre com o título (`# …`) e um parágrafo **em português** que diz o que a
  página registra, o resultado e por que importa. O resto do corpo fica no
  idioma em que foi escrito: o texto técnico acompanha o código e os commits.
- O nome do projeto é plev. Nomes de crate, módulo, formato e comando são os
  do código: `TERMS.toml` mapeia os nomes antigos (`sem`, `comps`,
  `showcase`…) para os de hoje, e um registro antigo conserva os da sua época.
- Uma página fala do que aconteceu, não da organização do acervo: nada de
  "movido de", "reorganizado em" ou notas sobre a migração. A origem fica no
  cabeçalho.
- Instruções de processo que só valiam para o fluxo de arquivos da época
  (mover o arquivo para `ongoing/`, renomear para `-DONE`) saem; o que
  ensina algo técnico fica.
- Uma afirmação reconstruída diz em que se apoia (código num commit, um
  teste, uma medição registrada). O que não tem fonte não se afirma.
- Links são relativos e apontam para páginas do acervo ou para o código do
  repositório; código citado como prova histórica aponta para o commit.

## Índices

Os `README.md` de cada pasta, a [linha do tempo](../../historia/linha-do-tempo.md)
e o `CATALOG.json` são gerados por `tool/tasks/docbase.py index`; não se editam
à mão.
