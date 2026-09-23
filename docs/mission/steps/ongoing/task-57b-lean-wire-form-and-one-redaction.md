---
project: plev
audience: [ai-agents, contributors]
status: in-progress
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

## revisão do orquestrador (2026-09-23, antes do briefing)

diagnóstico, pelo mecanismo:

- `crates/sem/src/node.rs:24-39`: `Node` deriva serde sem atributo, então
  todo campo é escrito, inclusive `enabled: true`, `inert: false`,
  `focusable: false`, `description: null`, `parent: null`, `state: {}`,
  `actions: []`, um `constraints` inteiro com sete campos no valor padrão
  e o `source` com arquivo, linha e coluna. `crates/sem/src/decl.rs:146-153`:
  o mesmo em `ActionDecl` (`label: null`, `reads: []`, `param: null`) e
  em `Policy` (quatro atores `true`, `confirm: "never"`,
  `destructive: false`).
- `crates/sem/src/snapshot.rs:100`: a revisão é calculada da árvore em
  memória e ignora o `source`; nenhuma forma JSON muda revisão. um teste
  prova.
- `crates/wire/src/server.rs:270-283`: a resposta `snapshot` clona cada
  nó e filtra `state` pelas `keys`; o `source` vai junto. um `Change`
  (`crates/sem/src/diff.rs`) só carrega `SemId`, então o delta nunca
  carregou `source`: a retirada é só na resposta `snapshot`.
- `crates/rec/src/persist.rs:58`: a trilha `S` é JSON lines do `Event`
  pela forma serde; o gravador serializa o snapshot que recebe, com
  `source`, e `crates/rec/src/weigh.rs:58` conta esses bytes no
  orçamento do anel. o `rec` continua gravando o `source` (a task-65
  precisa dele). por isso o `source` não sai da forma serde: sai da
  resposta do `wire`, a não ser que o agente peça.
- `crates/wire/src/redact.rs` e `crates/rec/src/redact.rs`: a mesma regra
  duas vezes, chamada em `crates/wire/src/server.rs:21` (e o `REDACTED`
  reexportado em `lib.rs:55`) e em `crates/rec/src/ring.rs:238,243`. as
  duas divergem num ponto: o `wire` redige um delta stale só quando o nó
  está entre os nós do request (alvo e observados) que eram secretos ou
  desconhecidos no dispatch; o `rec` redige quando o nó é secreto ou
  desconhecido no último snapshot dele. em todo caso testado coincidem,
  porque o delta stale é sobre os `reads` da ação, que são os nós
  observados.

decisões do orquestrador:

1. a forma enxuta é do `sem`: `skip_serializing_if` no valor padrão e
   `#[serde(default = ..)]` na leitura, campo a campo; a forma antiga
   (padrões explícitos, `source: null`) continua lendo e vira fixture de
   teste; a forma nova é o golden. a versão do arquivo `.monster` de
   sessão (`MONS`) não muda: a trilha `S` é forma serde e uma linha da
   forma antiga lê pelo `default`.
2. `source` fica na forma serde quando existe (`skip` só em `None`). a
   observação do agente não o recebe: `Observe` ganha `source: bool`
   (padrão `false`), o servidor zera `node.source` nos nós da resposta a
   não ser que pedido, e o `plev observe` ganha `--source`.
3. a regra de redação mora em `sem::redact` (nome aprovado), com `REDACTED`,
   `is_secret(state, id)`, `length(v)` e um `Secrets` capturado de um
   request e um snapshot num instante. `Secrets::holds(id)` é a união das
   duas regras de hoje: os nós do request secretos ou desconhecidos no
   snapshot consultado, mais todo nó que esse snapshot marca `secret`. o
   `wire` e o `rec` passam a chamar o `sem` e as duas cópias saem, com os
   testes de segredo dos dois inalterados no que afirmam.
4. medição de bytes só (a task-62 fase c mede token): observação completa,
   por escopo e delta, no todo com 3 e com 103 itens, forma antiga e
   nova, na tabela deste arquivo.
