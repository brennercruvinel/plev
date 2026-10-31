---
id: task-57b
title: "task-57b: forma JSON enxuta e uma redação só"
project: plev
type: tarefa
status: concluido
date: 2026-09-23
date_basis: cabecalho
topics: [semantica, agentes, formatos]
relations: [task-61, task-62, task-63, task-65, adr-0039, adr-0040, changelog]
evidence: [a7a6735]
source: docs/log/steps/task-57b-lean-wire-form-and-one-redaction.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# task-57b: forma JSON enxuta e uma redação só

Enxuga a forma JSON do `Snapshot` do crate `sem` (hoje `accord`) e reúne numa função só, `sem::redact`, a regra de redação de segredo que estava copiada em `wire` (hoje `socket`) e `rec` (hoje `record`). Uma observação completa do todo com 3 itens caiu de 6419 para 2039 bytes, e o `source` passou a sair da resposta do agent port, a não ser que o agente peça, sem deixar de ser gravado na sessão para a task-65. Foi integrada em `a7a6735` com o aceite de menos de 3 KB atendido e a regra de redação encontrada só em `rust/accord/src/redactor.rs`; importa porque uma regra de segurança em duas cópias já tinha divergido, e bytes sem significado custam tokens ao agente.

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
   independente em `rust/record/src/redact.rs` e em `rust/socket/src/redact.rs`.
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

## fechamento (2026-09-23)

integrada na `main` em `a7a6735` (seis commits, rebaseados sobre
`1debd01` sem conflito), validada pelo orquestrador: diff inteiro lido
commit a commit, `./tool/tasks/fullcheck.sh` verde no diretório de build da task
(1941 passed, 0 failed, 6 ignored; a `main` tinha 1930), e o grep do
aceite achando a regra de redação só em `rust/accord/src/redactor.rs`. o
aceite de 3 KB passou com 2039 bytes (2939 com `source` pedido). ADR:
[`0040-lean-semantic-form-and-one-redaction.md`](../decisoes/semantica/0040-lean-semantic-form-and-one-redaction.md); o ADR da task-61
aponta para ele nas duas consequências que esta task fecha.

aceitos: a união do `holds` (diverge da cópia do rec só num caso que o
`check` não produz, escrito no ADR); `SnapshotReply.focused` do proto
continua `null` (15 bytes, fora do contrato do sem, candidato a um v2); a
fixture da forma antiga do rec é uma linha construída no teste, porque
não existe `.monster` gravado antes no repositório. a `main` andou só em
docs entre o briefing e a integração, então o gate da branch rebaseada
é o gate do merge. o gate repetido no diretório da `main` foi
interrompido pelo orquestrador com o disco em 1,5 GB livres (o build
dela cresceu 10 GB); o código gateado e o integrado são o mesmo, e a
próxima validação em código roda em diretório isolado.

o que segue: a task-62 fase c mede quanto token esta forma custa a um
agente de verdade; um v2 do protocolo pode tirar o `focused` nulo.

## revisão do orquestrador (2026-09-23, antes do briefing)

diagnóstico, pelo mecanismo:

- `rust/accord/src/controls.rs:24-39`: `Node` deriva serde sem atributo, então
  todo campo é escrito, inclusive `enabled: true`, `inert: false`,
  `focusable: false`, `description: null`, `parent: null`, `state: {}`,
  `actions: []`, um `constraints` inteiro com sete campos no valor padrão
  e o `source` com arquivo, linha e coluna. `rust/accord/src/manifest.rs:146-153`:
  o mesmo em `ActionDecl` (`label: null`, `reads: []`, `param: null`) e
  em `Policy` (quatro atores `true`, `confirm: "never"`,
  `destructive: false`).
- `rust/accord/src/snapshot.rs:100`: a revisão é calculada da árvore em
  memória e ignora o `source`; nenhuma forma JSON muda revisão. um teste
  prova.
- `rust/socket/src/server.rs:270-283`: a resposta `snapshot` clona cada
  nó e filtra `state` pelas `keys`; o `source` vai junto. um `Change`
  (`rust/accord/src/mutation.rs`) só carrega `SemId`, então o delta nunca
  carregou `source`: a retirada é só na resposta `snapshot`.
- `rust/record/src/persist.rs:58`: a trilha `S` é JSON lines do `Event`
  pela forma serde; o gravador serializa o snapshot que recebe, com
  `source`, e `rust/record/src/weigh.rs:58` conta esses bytes no
  orçamento do anel. o `rec` continua gravando o `source` (a task-65
  precisa dele). por isso o `source` não sai da forma serde: sai da
  resposta do `wire`, a não ser que o agente peça.
- `rust/socket/src/redact.rs` e `rust/record/src/redact.rs`: a mesma regra
  duas vezes, chamada em `rust/socket/src/server.rs:21` (e o `REDACTED`
  reexportado em `lib.rs:55`) e em `rust/record/src/ring.rs:238,243`. as
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

## medição (2026-09-23)

bytes da linha JSON que o servidor escreve (`encode_server` da `Reply`,
envelope incluído), medidos pelo harness `agent_port_cost` de
`apps/uigallery/src/views/dispatcher/mod.rs`: a página todo do showcase servida
por uma porta real em diretório temporário, sem janela, build de debug
(bytes não dependem do perfil). a forma antiga é a `main` em `22f885e`, a
nova é esta branch. token é da task-62 fase c.

| o que | 3 itens, antiga | 3 itens, nova | 103 itens, antiga | 103 itens, nova |
| --- | --- | --- | --- | --- |
| observação completa (`observe` sem campo) | 6419 B | 2039 B | 87417 B | 26237 B |
| a mesma com `source: true` | (o padrão antigo) | 2939 B | (o padrão antigo) | 36137 B |
| uma linha (`--scope todo.item[1]`) | 929 B | 361 B | 929 B | 361 B |
| completa, `keys` = `completed` | 6281 B | 1868 B | 87277 B | 26064 B |
| delta de um toggle (`since`) | 624 B | 624 B | 632 B | 632 B |

o aceite (observação completa do todo com 3 itens abaixo de 3 KB) passa
com folga: 2039 B, e ainda 2939 B quando o agente pede o `source`. a
estimativa da task-61 (2532 B, um filtro grosseiro sobre o JSON) errava
para cima porque não tirava o `constraints` inteiro nem o `focused`
nulo. o delta não muda porque um `Change` carrega ids e valores, nenhum
dos campos que saíram da forma. a linha da forma nova é 68 % menor no
todo com 3 itens, 70 % menor com 103, 61 % menor por escopo de um item.

## estado (2026-09-23, entrega do agente)

branch `task/57b-lean-wire`, base `main @ 22f885e`. a `main` andou um
commit de docs depois do briefing (`1debd01`, desenho da task-62 fase c;
tabelas do programa e o arquivo da task-62), sem tocar em código; o
rebase é do orquestrador na integração.

commits, cada um compilando; gate verde no último:

1. `7d2cbcb` sem: padrões omitidos na forma serde e `default` na leitura
   (`rust/accord/src/{decl,node,snapshot,request}.rs`), fixture da forma
   antiga (`OLD_SNAPSHOT`) e golden novo em `rust/accord/tests/encoding.rs`.
2. `df44464` wire: `Observe.source` (padrão `false`), o servidor zera
   `node.source` na resposta `snapshot` a não ser que pedido, `plev
   observe --source` (`rust/socket/src/{proto,server}.rs`,
   `rust/socket/src/bin/plev.rs`, `rust/socket/tests/{codec,server,cli}.rs`).
3. `e95253a` `sem::redact` (novo, `rust/accord/src/redactor.rs`, `pub mod` em
   `lib.rs`), o wire sobre ele (`rust/socket/src/{lib,server}.rs`), a
   cópia `rust/socket/src/redact.rs` apagada, `wire::REDACTED` removido
   (único consumidor: `rust/socket/tests/server.rs`, que lê
   `sem::redact::REDACTED`).
4. `1c72d96` rec sobre `sem::redact` (`rust/record/src/{lib,ring}.rs`), a
   cópia `rust/record/src/redact.rs` apagada, teste da linha de `Event` da
   forma antiga em `rust/record/tests/session/file.rs`.
5. `b10cd36` medição: harness do showcase mede a forma servida e a mesma
   com `source: true` (`apps/uigallery/src/views/dispatcher/mod.rs`, só o módulo
   de testes do `agent`); a tabela acima.
6. este: estado e handoff.

testes (`cargo test --workspace`, linhas de resumo somadas): `main`
1930 passed, 0 failed, 6 ignored, 69 linhas; branch 1941 passed, 0
failed, 6 ignored, 69 linhas. os 11 a mais: 3 em `sem/tests/wire.rs`
(forma antiga lê, campo ausente lê no padrão, `source` roundtrip), 6
unitários em `sem::redact`, 1 em `wire/tests/server.rs` (`source` só
quando pedido), 1 em `rec/tests/session/file.rs` (linha antiga do
`Event`). o teste de parse do `plev` e o roundtrip do codec ganharam o
campo sem contar a mais.

validação executada, com `CARGO_TARGET_DIR=target/agents/task-57b`,
`CARGO_INCREMENTAL=0` e debug `line-tables-only`: `cargo test -p accord`,
`-p socket`, `-p record`, `-p uigallery`, `cargo check -p accord
--no-default-features`, `./tool/tasks/fullcheck.sh` (test, clippy `-D warnings`,
fmt, wasm check do showcase; a perna urnaui pulou por falta de `../urna`),
`cargo test -p uigallery agent_port_cost -- --ignored --nocapture` antes e
depois. `grep -rn 'REDACTED\|fn length\|fn is_secret\|struct Secrets'
crates --include='*.rs'` acha a regra só em `rust/accord/src/redactor.rs`; a
outra ocorrência é o teste do wire lendo `sem::redact::REDACTED`.

decisões locais:

- os predicados da forma enxuta (`is_default<T: Default + PartialEq>`,
  `is_false`, `is_true` e `yes()` para o padrão `true` de `enabled`) são
  `pub(crate)` em `rust/accord/src/manifest.rs`, sob a feature `serde`; o
  `node.rs` os importa. `Constraints` sai inteira quando é o padrão e,
  quando não é, cada campo dela no padrão sai também. `Actors` é escrito
  inteiro quando algum ator está de fora.
- o `skip` de `source` em `None` entrou no commit 1 com os demais campos
  do `Node` (é um atributo); o commit 2 ficou só com o wire.
- no rec o `Secrets::of` é construído a cada estágio contra o último
  snapshot dele, como a cópia fazia; no wire é capturado uma vez no
  dispatch, como a cópia fazia. o comportamento dos dois não mudou.
- as duas cópias não tinham teste unitário; os seis de `sem::redact` são
  novos e os de comportamento (`rec/tests/session/secret.rs`,
  `wire/tests/server.rs`) ficaram com as mesmas asserções.
- `SnapshotReply.focused` (proto do wire, não `SnapshotWire` do sem)
  continua escrito como `null`: não estava no contrato; são 15 bytes.
- a função `lean()` do harness (estimativa por filtro de JSON) saiu: a
  forma servida é a enxuta.

ambiguidades e dívidas, para o orquestrador:

- `Secrets::holds` é a união decidida. ela diverge da cópia do rec num
  caso só: um `Change` stale sobre um nó que não é do request, não é
  secreto no snapshot do rec e o rec não conhece; o rec reduzia o valor à
  length e a união não. o caso é inalcançável hoje: `check::stale`
  compara os `reads` da ação como `now` os declara, o `before` de um nó
  fora do `observed` é `None` e o `after` vem do snapshot do dispatcher,
  onde um secreto já é length. o valor cru de um segredo só entra por
  `observed.values` de um request, e esses nós estão no request.
- a forma antiga no rec é uma linha construída no teste (padrões
  explícitos, `source: null`, um nó com `source`), lida pelo mesmo
  desserializador de `Event` que `read_events` usa. um arquivo `.monster`
  gravado antes da mudança não existe no repositório para servir de
  fixture.
- docs fora do escopo que citam o usage antigo ou a cópia dupla:
  [`0039-agents-act-through-the-semantic-contract.md`](../decisoes/semantica/0039-agents-act-through-the-semantic-contract.md) (linha 157, o
  usage sem `--source`; linhas 244 a 250, as duas consequências que esta
  task fecha), [`changelog.md`](../historia/changelog.md) se a mudança for user-facing (o
  `--source` é). o how-to de orquestração não cita o usage.
- risco de integração: nenhum golden fora de sem, wire e rec pinava a
  forma antiga em bytes (grep em showcase, ide, engine, comps: só nomes
  de campo em código). o `ide` constrói `Policy` e `Constraints` por
  literal, o que não muda com atributos serde.

proposta de texto para o ADR (um parágrafo):

a forma serde de `sem` é enxuta: um campo no valor padrão não é escrito
e, ausente, lê como o padrão, então uma observação e uma linha de sessão
carregam só o que diz algo (2039 bytes o todo do showcase com 3 itens,
contra 6419). `source` fica na forma quando existe, porque o gravador
precisa dele para ir do replay ao código (task-65), mas sai da resposta
do agent port a não ser que o agente peça (`observe {source: true}`,
`plev observe --source`): um agente age por id, e o lugar do código que
projetou um controle não é parte do significado dele. a regra que impede
um segredo de sair do processo mora num lugar só, `sem::redact`, porque
`sem` é o dono do que `secret` significa e é o único crate que o wire e o
rec compartilham; uma regra de segurança escrita duas vezes tinha
divergido, e o `Secrets` capturado de um request e um snapshot num
instante é a união das duas regras (os nós do request secretos ou
desconhecidos, mais todo nó marcado `secret`), fechado por padrão.
