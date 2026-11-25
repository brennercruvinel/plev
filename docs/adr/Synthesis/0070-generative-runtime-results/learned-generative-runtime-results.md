---
id: learning-cycle-0070
title: "what the autonomous cycle taught: generative runtime results"
project: plev
type: learning
status: reference
date: 2026-09-25
date_basis: text
recorded: 2026-09-23
topics: [agents, generated-surfaces, semantics, recording, research]
relations: [adr-0070, task-97, task-80, task-96]
source: docs/log/learnings/general.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# what the autonomous cycle taught: generative runtime results

Records what the experiments and implementations of the autonomous cycle of September 2026 taught about [ADR 0070](0070-generative-runtime-results.md), in the words written at the time (in Portuguese), newest first, each with its source.

## 2026-09-25, task-97

- **medir de novo pelo mesmo critério mostrou o próximo teto:** com o
  componente de diff, a demo 5 passou de 5 para 14 linhas à vista, e o
  que sobrou de derrota é o layout (um só token de altura), não o
  componente. cada rodada de medida move o gargalo; o critério fixo é o
  que deixa ver para onde. fonte: a demo 5 no arquivo da task-80.
- **olhar a imagem pega o que o teste não pega:** os testes da demo
  contam linhas e atos, e passaram com o título do cartão sendo o id cru
  do commit e a coluna vazia apertando o ide. a validação do orquestrador
  abre as imagens, sempre. fonte: a demo 5 da task-97.

## 2026-09-25, task-96 e a demo 5

- **a superfície gerada ganha onde junta, perde onde precisa de um
  componente que o vocabulário não tem:** juntar os arquivos de três
  commits num lugar foi melhor que o painel fixo; mostrar um diff numa
  tabela foi pior. o vocabulário fechado é o teto da geração: quando a
  demo perde por falta de componente, a resposta é o componente, medido
  de novo pelo mesmo critério, não um critério mais fácil. fonte: a demo
  5 no arquivo da task-80.
- **o Claude vê um snapshot feito para ele:** o prompt leva todo nó do
  snapshot que recebe, então o ide monta um só com os commits que a
  revisão pede. regra: um canal novo para o modelo recebe uma projeção
  própria, nunca a do app inteiro. fonte: `apps/idestudio/src/generative/scope.rs`.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0070: generative runtime results](0070-generative-runtime-results.md)
- [task-80: proof of qualitatively different software](../../../Tasks/task-80-proof-qualitatively-different-software/task-80-proof-qualitatively-different-software.md)
- [task-96: a generated surface in the ide](../../../Tasks/task-96-ide-generated-surface/task-96-ide-generated-surface.md)
- [task-97: code and diff in the IR vocabulary](../../../Tasks/task-97-code-and-diff-in-the-ir/task-97-code-and-diff-in-the-ir.md)
<!-- /related -->
