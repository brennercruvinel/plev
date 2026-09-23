---
project: plev
audience: [ai-agents, contributors]
status: pending
last-updated: 2026-09-23
domain: generative-runtime
task: task-57b
depends: [task-61, task-63]
branch: task/57b-lean-wire
---

# task-57b: forma JSON enxuta e uma redação só

## objetivo

duas coisas que a task-61 mostrou, antes do experimento da task-62 fase c,
que mede quanto dado e quanto token o agente recebe:

1. a forma JSON de um `Snapshot` é verbosa. no todo do showcase, uma
   observação completa tem 6419 bytes; o `source` (arquivo, linha e coluna
   de cada nó) é cerca de 28 % disso, e os campos com valor padrão cerca de
   40 % a mais. a estimativa enxuta é 2532 bytes. para um agente, isso é
   custo em tokens que não carrega significado.
2. a regra de redação de segredo (o valor de um campo `secret` sai só como
   tamanho; texto livre sobre um alvo secreto é trocado por um texto fixo;
   nó desconhecido conta como secreto) existe duas vezes, escrita
   independente em `crates/rec/src/redact.rs` e em `crates/wire/src/redact.rs`.
   uma regra de segurança com duas cópias diverge.

## escopo

- `sem`: os campos com valor padrão deixam de ser escritos na forma serde
  (`skip_serializing_if`, com `#[serde(default)]` na leitura, então JSON
  antigo continua lendo). o `source` sai da forma padrão e vira opt-in: a
  observação do agente não o carrega, a não ser que peça (`observe
  {source: true}` no `wire`); o `rec` continua gravando o `source` (a
  task-65 precisa dele para ir do replay ao código).
- `sem::redact` (ou nome aprovado): a regra única, com as funções que
  `rec` e `wire` usam hoje; os dois passam a chamar a do `sem` e as cópias
  saem.
- goldens do `sem`, do `wire` e do `rec` atualizados; um teste prova que
  um JSON da forma antiga ainda lê.
- a tabela de bytes da task-61 medida de novo (observação completa, uma
  linha, delta), no todo com 3 e com 103 itens.

## aceite

observação completa do todo abaixo de 3 KB com 3 itens; nenhuma
redação fora do `sem` (grep); os testes de segredo do `rec` e do `wire`
verdes sem mudança de comportamento; forma antiga lendo; gate verde.
