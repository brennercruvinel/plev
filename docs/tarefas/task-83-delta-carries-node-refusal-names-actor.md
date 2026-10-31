---
id: task-83
title: "task-83: o delta traz o nó que nasceu, e a recusa diz quem agiu"
project: plev
type: tarefa
status: concluido
date: 2026-09-23
date_basis: cabecalho
topics: [semantic-contract]
relations: [task-57b, task-61, task-62]
evidence: [78f463c]
source: docs/log/steps/task-83-delta-carries-node-refusal-names-actor.md
source_revision: ec0a224c
revised: 2026-10-30
review: pendente
---

# task-83: o delta traz o nó que nasceu, e a recusa diz quem agiu

## por que

os dois agentes LLM do checkpoint a operaram o ide só pelo contrato e
apontaram, sem que isso impedisse a tarefa (`task-62`, seção "o que o
experimento mostrou"):

- `node_added` num delta traz só o id: o agente que vê o commit nascer em
  `stacks` precisa de uma segunda observação para saber o que nasceu;
- a recusa por estado velho diz o que mudou, mas não quem mudou: o agente
  não sabe se foi uma pessoa, outro agente ou algo de fora do app;
- a regra "stale é por `reads`, e a revisão só rastreia" foi deduzida pelo
  agente, não lida em nenhum lugar que ele alcança.

o brenner escolheu corrigir isso antes da onda 5.

## estado no código (main @ c5fdf36)

- `rust/accord/src/mutation.rs:23`: `Change::NodeAdded(SemId)`.
- `rust/record/src/ring.rs:47-53`: o recorder trata `NodeAdded`,
  `ParentChanged` e `Reordered` como estruturais e grava um snapshot
  inteiro; `rust/record/src/state.rs:83` recusa `node_added` dentro de
  `Kind::Changes`.
- `rust/socket/src/server.rs:329-339`: o delta passa o `Change` como veio
  do `diff`, filtrando só `StateChanged` por `keys`; o snapshot tira
  `source` quando não pedido (`:349`).
- `rust/accord/src/dispatch.rs`: o dispatcher guarda os receipts abertos e
  os últimos 256 terminados, cada um com `validated_against` (a revisão
  viva em que foi checado); ninguém usa isso para dizer quem agiu.
- `apps/idestudio/src/capability/notice.rs:84-87`: a recusa stale vira "Refused
  agent's submit: <o que mudou>", sem autor da mudança.

## escopo

1. `Change::NodeAdded` carrega o nó inteiro; o wire aplica ao nó o mesmo
   filtro de `keys` e a mesma retirada de `source` do snapshot.
2. `Rejection::Stale` ganha a lista das ações de outros atores que rodaram
   sobre o estado que o autor da ação recusada viu (regra exata no
   briefing); o dispatcher preenche num lugar só.
3. a recusa do ide nomeia quem agiu.
4. a regra de stale fica escrita onde o agente lê (`plev --help`, doc do
   protocolo) e no ADR.

## fora do escopo

aplicar `NodeAdded` no recorder sem snapshot inteiro (a posição entre
irmãos não vem no `Change`), `files` no nó do commit (task-85), a corrida
do `Refresh` (task-84), eco do `set_value`.

## aceite

testes novos no sem, no wire e no ide; o delta de um item novo no todo traz
o nó; a recusa do laboratório (`ide-lab --arm scripted --human
unstage-after-observe:src/session.rs`) nomeia a pessoa que tirou o arquivo
do index; gate verde; ADR novo.

## medição

bytes da linha que o servidor escreve (sem o `\n`), build de debug, por
um socket de verdade. o "antes" rodou na base (`dbc4a3f` mais o teste de
medição, que entrou primeiro justamente para isso), o "depois" na ponta
da branch.

o todo do showcase: uma pessoa adiciona um item pela página e o agente
observa `since` a revisão de antes (`cargo test -p uigallery
agent_port_new_row -- --ignored --nocapture`).

| itens antes | delta antes | delta depois | observação inteira depois |
| --- | --- | --- | --- |
| 3 | 423 B | 647 B | 2286 B |
| 103 | 378 B | 604 B | 26479 B |

o delta leva o nó novo, o rótulo e o `active` do contador e, com 3
itens, o foco indo para o campo. o nó custa uns 225 B. o delta fica em
28% da observação inteira com 3 itens e em 2,3% com 103, longe da
condição de parada (delta maior que o snapshot inteiro).

o laboratório, quando o commit nasce em `stacks`: `ide-lab --arm plev
--human none --grace 20 --app ide-t83m`, a tarefa feita com `plev` por um
script (fora do repositório), depois `plev observe --since <a revisão em
que o submit se apoiou> --scope stacks`.

| | antes | depois |
| --- | --- | --- |
| delta de `stacks` | 172 B | 427 B |
| segunda observação para saber o commit (`--scope stacks.commit[sha]`) | 402 B | não precisa |
| o mesmo delta sem escopo | 1748 B | 2003 B |
| observação inteira de `stacks` | 818 B | 818 B |
| observação inteira | 4205 B | 4205 B |

574 B em duas idas e voltas viraram 427 B em uma. na sessão gravada do
mesmo roteiro a seção semântica foi de 53732 para 54066 B, 51 eventos
nos dois: o evento de snapshot que adiciona o commit repete o nó no seu
`changes`.

o aceite do laboratório: `cargo run -p agenteval -- --arm
scripted --human unstage-after-observe:src/session.rs --out
tmp/lab-t83` (no `tmp/` do worktree, ignorado pelo git) saiu 0, MATCH.
`report.json` traz em `stale[0].acted` um só nome, `human unstage
changes.file[src/session.rs]`, request 6, o da intervenção; o agente
roteirizado leu o mesmo `acted` pelo wire; o `session.monster` grava o
estágio com `acted` e o toast "Refused scripted's submit: src/session.rs
left the staged files (you unstaged src/session.rs)". a regra não nomeou
ninguém além da pessoa.

## handoff

```text
Goal: task-83, o delta traz o nó que nasceu, e a recusa diz quem agiu
Status: pronto para validação; a decisão pedida foi tomada (B) e está feita
```

commits, na ordem:

- `41fcb2d` teste de medição ignorado do delta de uma linha nova no todo
  (entrou antes para medir a base).
- `0cbe4b0` `Change::NodeAdded(Box<Node>)`, forma serde pinada, rec
  (peso do nó, comentário de `structural`) e testes.
- `6e06c44` wire: `serve`, um filtro só (keys e `source`) para o nó do
  snapshot e o nó do `node_added`; teste.
- `7d1f80c` `Acted`, `Rejection::Stale { changes, acted }`,
  `Rejection::stale`, preenchimento no `Dispatcher::push`, redação,
  peso no rec; os outros crates só ganham `..` no match.
- `f3038ad` ide: o toast diz quem agiu; o laboratório escreve `acted` no
  relatório e o agente roteirizado o registra; teste de notice e do lab.
- `5eefba5` a regra de stale no `plev --help` e no doc do `Act`; teste.
- `d7f7c60` o ADR novo e a linha no ADR dos agentes.
- `dc8396f` medição e handoff neste arquivo.
- `def57cf` rec: o leitor da sessão converte o `node_added` da forma
  antiga num evento de snapshot; doc de `Kind::Changes`; teste da leitura
  e teste ignorado que abre sessões por caminho (`REC_OPEN`).
- `3687980` o ADR diz o que é verdade depois da conversão; comentário do
  teste pinado.
- o commit desta atualização do handoff.

o que mudou em cada contrato:

- `sem::Change::NodeAdded(Box<Node>)` (era `SemId`): o nó como o
  snapshot `after` o tem, com `source`; forma
  `{"node_added": {<nó na forma enxuta>}}`. `Snapshot::diff` lista as
  mesmas mudanças na mesma ordem, e nada exatamente quando os snapshots
  são iguais. o `Change` não lê a forma antiga (string): sem o snapshot
  não há nó.
- rec, leitura de sessão (`read_event` em `rust/record/src/persist.rs`):
  uma linha que não lê como `Event` tem mais uma chance. num evento de
  snapshot, cada `node_added` escrito como id vira o nó que o snapshot do
  próprio evento tem para aquele id, como foi gravado. id que o snapshot
  não tem é `FileError::Event` naquela linha; nenhum nó inventado. a
  escrita só escreve a forma nova; o `MONS` continua na versão 0; nenhum
  tipo público do rec mudou.
- wire: o nó de um `node_added` passa pelo mesmo `serve` do snapshot
  (`state` cortado por `keys`, `source` só com `observe.source`).
  nenhum tipo público do wire mudou; docs de `Observe`, `DeltaReply` e
  `Act` atualizados. protocolo continua v1.
- `sem::Rejection::Stale { changes, acted: Vec<Acted> }`, `acted` com
  `serde(default, skip_serializing_if = "Vec::is_empty")`;
  `pub struct Acted { id, actor, target, action }` exportado;
  `Rejection::stale(changes)` para quem constrói fora do dispatcher
  (`check`, `truth.rs` do ide). o `Dispatcher` preenche em `push`, o
  único lugar por onde todo estágio passa: dispatch, segunda checagem
  do `confirm`, `Execution::Rejected` e `reject_at_truth`; o que o
  chamador pôs em `acted` é substituído. a busca lê só `open` e
  `finished` (já limitados). a revisão continua não decidindo nada.
- `sem::redact`: `Acted` não carrega segredo e passa como está; o delta
  de nó secreto continua reduzido a comprimento, alvo claro ou secreto
  (teste em `redact.rs`).
- ide: o toast de stale acrescenta "(you unstaged src/session.rs)"
  quando `acted` não está vazio; vazio, o texto é o de hoje.
- `plev` USAGE e doc do `Act`: a frase da regra.

decisões minhas, locais:

- "chegaram a rodar" = estágio atual executing, completed ou failed. um
  pedido que passou por executing e foi recusado no ponto de verdade
  fica de fora: ele não rodou efeito nenhum, e a regra do briefing tira
  "recusados". um que falhou entra (rodou e pode ter mexido em algo).
- o teste de medição do todo é um teste ignorado novo em
  `apps/uigallery/src/views/dispatcher/mod.rs` (arquivo permitido), ao lado do
  `agent_port_cost` da 57b, para o antes e o depois rodarem o mesmo
  código. a medição do laboratório usou um script de `plev` fora do
  repositório (não precisou mudar o lab).
- os verbos do toast (`staged`, `unstaged`, `committed`, `edited the
  commit message`...) ficam em `notice.rs`, com um genérico `ran <ação>
  on <alvo>`.

validação executada (com o ambiente do briefing):

- `cargo test -p record && cargo test -p accord && cargo test -p socket &&
  cargo test -p idestudio`: verde, 323 passed, 0 failed, 2 ignored.
- `./tool/tasks/fullcheck.sh` na ponta da branch: verde. workspace 1996 passed, 0
  failed, 8 ignored (a main tinha 1985, 0, 6: onze testes novos e dois
  ignorados novos, o de medição do todo e o que abre sessões por
  caminho). a perna do urnaui rodou também, porque existe um checkout em
  `../urna` relativo ao worktree: 80 passed, 0 failed, 1 ignored.
- uma rodada do gate falhou antes dessa, na perna do urnaui, sem relação
  com o código: 25 erros "multiple different versions of crate `sem`"
  compilando `comps`. o motivo: o `engine` tem `crate-type = ["cdylib",
  "staticlib", "rlib"]`, e com cdylib o cargo escreve `libengine.rlib` sem
  hash no nome. o workspace da raiz e o do urnaui compartilham esse
  arquivo no mesmo target dir: quando a raiz recompila o `engine` depois
  de uma rodada do urnaui, o urnaui acha o seu `engine` em dia, mas o
  arquivo é o da raiz, ligado a outro `sem`. a mesma perna falhava
  sozinha, de novo, até um `cargo clean -p engine` no mesmo target dir;
  depois dele o gate inteiro passou. acontece desde que o `engine` passou
  a depender do `sem` (task-59), num target dir usado pelas duas pernas.
  fica para o orquestrador: não mexi em `tool/tasks/fullcheck.sh` nem no
  `rust/engine/Cargo.toml`.
- o comando do laboratório de aceite: exit 0, com a recusa nomeando a
  pessoa (seção medição).

testes novos ou estendidos:

- sem: `a_node_added_is_the_node_as_after_holds_it` (diff),
  `a_node_added_has_a_pinned_json_form_and_roundtrips` e
  `a_stale_refusal_roundtrips_with_and_without_who_acted` (wire),
  `a_stale_refusal_names_who_ran_something_on_the_state_the_agent_saw`,
  `a_refusal_at_the_point_of_truth_names_who_acted_too`,
  `who_acted_is_read_from_the_receipts_the_dispatcher_holds` (dispatch),
  o de redação do delta secreto estendido com `acted` e alvo secreto.
- wire: `a_node_added_in_a_delta_is_served_as_a_snapshot_serves_it`,
  `a_stale_refusal_names_the_person_who_acted_on_what_the_agent_saw`,
  `the_help_says_what_makes_an_act_stale`.
- ide: `a_stale_agent_request_names_the_person_who_acted`; o
  `human_agent` com o texto novo do toast; o `lab` confere que `acted`
  nomeia o unstage da pessoa, request da intervenção.
- showcase: `agent_port_new_row` (ignorado, medição).
- rec: `a_node_added_by_id_alone_reads_as_the_node_its_snapshot_holds`
  e `open_the_named_sessions` (ignorado, lê `REC_OPEN`), em
  `rust/record/tests/session/file.rs`.

decisão tomada pelo orquestrador: B. as sessões gravadas antes continuam
abrindo. o `Kind::Snapshot { snapshot, changes }` guardava o diff do
passo com `node_added` como id; o leitor da sessão, que tem o snapshot,
põe no lugar o nó que o próprio snapshot tem (sem perda). a conversão não
foi para o `sem`: a forma antiga não carrega o nó e o `sem` não tem o
snapshot em mãos.

aceite manual, `Session::open` pelo teste ignorado
(`REC_OPEN=<caminhos separados por :> cargo test -p record --test session
open_the_named_sessions -- --ignored --nocapture`), nas sessões de
`/Users/nn/Dev/hoff/plevdev/target/agents/`. todas abrem; antes da
conversão as quatro primeiras davam `FileError::Event` (conferido em a1,
a2 e numa do lab rodada na base):

| sessão | eventos | `node_added` nos eventos de snapshot | receipts de `commit.submit` no fim |
| --- | --- | --- | --- |
| `exp/a1` | 61 (15 snapshot, 11 changes, 7 observed, 28 stage) | `stacks.commit[6eb55b7]` | request 6 de `agent:claude-a1`, completed, commit `6eb55b7`, os três arquivos |
| `exp/a2` | 67 (17 snapshot, 8 changes, 7 observed, 34 stage, 1 input) | `notices[0]` (o toast da recusa), `stacks.commit[7c21633]` | request 7 de `agent:claude-a2` rejected stale (`src/session.rs` saiu de `changes:staged`, `acted` vazio: gravada antes); request 9 completed, commit `7c21633`, os três arquivos |
| `lab-scripted` | 71 | `notices[0]`, `stacks.commit[117a78b]` | request 7 rejected stale; request 9 completed, `117a78b` |
| `lab-plev` | 58 | `notices[0]`, `stacks.commit[60685b2]` | request 7 rejected stale; request 9 completed, `60685b2` |
| `lab-control` | 46 | `stacks.commit[0222ec0]` | request 6 completed, `0222ec0` |
| `lab-control2` | 46 | `stacks.commit[9f33318]` | request 6 completed, `9f33318` |

o teste `a_node_added_by_id_alone_reads_as_the_node_its_snapshot_holds`
(`rust/record/tests/session/file.rs`) escreve a seção S de uma sessão
gravada na forma antiga, com o índice e os sha256 refeitos, e confere:
abre, e os eventos são iguais aos da janela em memória (o nó do
snapshot, com `source`); gravada de novo, só a forma nova sai; o mesmo
arquivo com um id que o snapshot não tem dá `FileError::Event` na linha
2, com o id na mensagem.

limites conhecidos (também no ADR e no doc de `Acted`):

- mudança de fora do app (um `git` no terminal) não tem receipt: aparece
  em `changes`, nunca em `acted`.
- de uma cadeia de ações de outra pessoa, só as checadas na revisão exata
  que o autor viu aparecem; a seguinte foi checada no estado que a
  primeira deixou.
- `acted` diz quem rodou algo sobre aquele estado, não que a ação mexeu
  no que a recusada lê: quem só moveu um cursor naquele estado aparece
  ao lado de quem tirou o arquivo do index. o `changes` diz o que mudou.
- receipt que saiu da retenção (256 terminados por padrão) não aparece.
- o `NodeAdded` continua obrigando o recorder a gravar o snapshot
  inteiro (a posição entre irmãos não vem no `Change`), e o evento de
  snapshot agora repete o nó adicionado no seu `changes`.
- o leitor da sessão carrega uma conversão da forma antiga, tentada só
  numa linha que não lê como está. `node_added` como id fora de um evento
  de snapshot (num `Kind::Changes`, que o recorder nunca escreveu)
  continua sendo o erro de hoje.

o que ficou de fora, e por quê:

- [`0037-session-recording-replay-not-reexecution.md`](../decisoes/gravacao/0037-session-recording-replay-not-reexecution.md) diz que "a
  `Change` carries neither the new node nor the new order" e
  [`0040-lean-semantic-form-and-one-redaction.md`](../decisoes/semantica/0040-lean-semantic-form-and-one-redaction.md) diz que "a `delta`
  never carried `source`". o ADR novo corrige os dois; a nota em cima
  de cada um fica com o orquestrador no fechamento.
- dívida registrada, fora deste escopo: `apps/agenteval/src/arms/plev.rs`
  já tinha 728 linhas na main (acima das 639) e foi a 733; a formatação
  de `acted` foi para `story.rs` para não crescer mais.
- `docs/ATLAS.toml` (papel do `sem`): cabe dizer que um `node_added`
  carrega o nó e que o dispatcher nomeia em `acted` quem agiu sobre o
  estado visto. [`code-against-the-plev-engine.md`](../referencia/operacao/code-against-the-plev-engine.md) (seção do
  contrato semântico) pode ganhar a mesma frase. os dois ficam com o
  orquestrador no fechamento.
- fora do escopo pelo arquivo da task: aplicar `NodeAdded` no recorder
  sem snapshot inteiro, `files` no nó do commit (task-85), a corrida do
  `Refresh` (task-84), eco do `set_value`.

riscos de integração: o gate num target dir compartilhado pelas duas
pernas pode falhar na do urnaui depois de uma recompilação do `engine`
pela raiz (acima); `cargo clean -p engine` resolve. quem casar
`Rejection::Stale { changes }` numa
branch aberta precisa de `..` (a task-84 mexe no ide; os arquivos dela,
`session.rs`, `views/` e `rust/gitdrv`, não foram tocados aqui). a task-85
põe `files` no nó do commit, que passa a viajar dentro do `node_added`
sem mais nada.

linha proposta para [`indice-historico.md`](../historia/indice-historico.md):

```text
| 2026-09-23 | [a-delta-carries-the-node-a-refusal-names-who-acted](a-delta-carries-the-node-a-refusal-names-who-acted.md) | accepted | `Change::NodeAdded` carrega o nó como o snapshot `after` o tem, e o agent port o serve com o mesmo filtro de `keys` e `source` do snapshot (o delta que mostra o commit nascendo em `stacks`: 172 B mais uma observação de 402 B viram 427 B numa ida só); `Rejection::Stale` ganha `acted`, as ações de outros atores checadas na revisão que o autor viu e que rodaram, preenchido só pelo dispatcher; a regra de stale fica no `plev --help` e no doc do `Act`; protocolo continua v1, e a sessão gravada antes continua abrindo: o leitor põe no lugar do id o nó que o snapshot do próprio evento tem, sem perda (task-83) |
```

## fechamento (orquestrador, 2026-09-23)

integrada na `main` em `78f463c` (onze commits, rebaseados sobre a `main`
sem conflito). validação do orquestrador:

- diff inteiro lido, commit por commit, nas duas rodadas. `acted` é
  preenchido num lugar só (`Dispatcher::push`), para toda recusa stale
  que o dispatcher registra; o receipt recusado nunca entra na própria
  lista (no dispatch ele ainda não foi guardado, no `reject_at_truth` ele
  sai de `open` antes do `push`). o wire serve o nó de um `node_added` e
  o de um snapshot pela mesma função (`serve`). a redação mantém `acted`
  como está, que só tem ids, ator e nome de ação.
- devolvida uma vez: a forma nova do `node_added` fazia as sessões
  gravadas antes não abrirem (a premissa errada estava no briefing do
  orquestrador). decisão: o leitor converte a forma antiga (opção B).
  aceite: as seis sessões em disco, as do checkpoint a inclusive, abrem.
- gate verde no diretório da task sobre a base rebaseada, com a perna
  urnaui no subdiretório próprio: 2084 passaram, 0 falharam, 9 ignorados.
- reproduzido pelo orquestrador: `ide-lab --arm scripted --human
  unstage-after-observe:src/session.rs` saiu 0, MATCH, e a recusa traz
  `acted = [human, unstage, changes.file[src/session.rs], request 6]`,
  ninguém mais.
- fechamento: linha no índice de ADRs, notas datadas nos ADRs
  session-recording e lean-semantic, manual do engine, changelog.

dívida registrada: `apps/agenteval/src/arms/plev.rs` passou de 728 para 733
linhas, acima do limite de tamanho de arquivo; dividir o laboratório fica
para a próxima task que mexer nele.
