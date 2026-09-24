---
project: plev
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2026-09-23
domain: semantic-contract
task: task-83
depends: [task-57b, task-61, task-62]
branch: task/83-delta-and-refusal
origin: checkpoint a (task-62 fase c), decisão do brenner "correções primeiro"
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

- `crates/sem/src/diff.rs:23`: `Change::NodeAdded(SemId)`.
- `crates/rec/src/ring.rs:47-53`: o recorder trata `NodeAdded`,
  `ParentChanged` e `Reordered` como estruturais e grava um snapshot
  inteiro; `crates/rec/src/state.rs:83` recusa `node_added` dentro de
  `Kind::Changes`.
- `crates/wire/src/server.rs:329-339`: o delta passa o `Change` como veio
  do `diff`, filtrando só `StateChanged` por `keys`; o snapshot tira
  `source` quando não pedido (`:349`).
- `crates/sem/src/dispatch.rs`: o dispatcher guarda os receipts abertos e
  os últimos 256 terminados, cada um com `validated_against` (a revisão
  viva em que foi checado); ninguém usa isso para dizer quem agiu.
- `crates/ide/src/workbench/notice.rs:84-87`: a recusa stale vira "Refused
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
observa `since` a revisão de antes (`cargo test -p showcase
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

o aceite do laboratório: `cargo run -p ide --bin ide-lab -- --arm
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
Status: pronto para validação; uma decisão pedida (sessões gravadas antes, abaixo)
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
- o commit deste handoff (medição e handoff neste arquivo).

o que mudou em cada contrato:

- `sem::Change::NodeAdded(Box<Node>)` (era `SemId`): o nó como o
  snapshot `after` o tem, com `source`; forma
  `{"node_added": {<nó na forma enxuta>}}`. `Snapshot::diff` lista as
  mesmas mudanças na mesma ordem, e nada exatamente quando os snapshots
  são iguais. a forma antiga (string) não lê mais.
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
  `crates/showcase/src/view/dispatch.rs` (arquivo permitido), ao lado do
  `agent_port_cost` da 57b, para o antes e o depois rodarem o mesmo
  código. a medição do laboratório usou um script de `plev` fora do
  repositório (não precisou mudar o lab).
- os verbos do toast (`staged`, `unstaged`, `committed`, `edited the
  commit message`...) ficam em `notice.rs`, com um genérico `ran <ação>
  on <alvo>`.

validação executada (com o ambiente do briefing):

- `cargo test -p sem && cargo test -p wire && cargo test -p rec &&
  cargo test -p ide`: verde.
- `./script/gate`: verde. workspace 1995 passed, 0 failed, 7 ignored (a
  main tinha 1985, 0, 6: dez testes novos e um ignorado novo, o de
  medição). a perna do urnaui rodou também, porque existe um checkout
  em `../urna` relativo ao worktree: 80 passed, 0 failed, 1 ignored.
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

decisão pedida (formato de ambiguidade):

```text
problema: o briefing supôs que nenhum arquivo persistido guardava a forma
  antiga do node_added. o Kind::Changes nunca guarda, mas o
  Kind::Snapshot { snapshot, changes } guarda o diff que explica o passo,
  com node_added quando um nó nasceu. sessão gravada antes desta task com
  um nó adicionado na janela não abre mais: Session::open responde
  FileError::Event naquela linha, tipado, sem panic. conferido nas
  sessões do checkpoint a (target/agents/exp/a1 e a2, fora do git) e numa
  do lab rodada na base.
opções:
  A. aceitar, como está (o briefing pede "não há leitura da forma
     antiga"); o ADR diz o custo.
  B. ler a forma antiga no rec (crates/rec/src/persist.rs, fora da minha
     lista): ao ler um Kind::Snapshot, trocar {"node_added": "<id>"} pelo
     nó que o snapshot do próprio evento tem. sem perda, porque o nó
     adicionado num evento de snapshot é exatamente o nó do seu snapshot.
     umas 30 linhas e um teste com uma linha antiga fixada.
  C. subir a versão do MONS para 1, para a sessão antiga ser recusada
     pela versão, com mensagem clara, sem leitura.
impacto: A deixa as sessões do checkpoint a sem abrir no Session::open e
  no inspector (task-64); os report.json continuam legíveis. B mantém a
  regra da 57b ("a forma antiga continua lendo") e custa um shim no
  leitor. C troca um erro de linha por um de versão e muda o formato.
recomendação: B, se as sessões do checkpoint a servem de material para a
  task-64; senão A. não fiz B porque o arquivo não é meu e o briefing
  decidiu por A com a premissa errada.
decisão necessária: A, B ou C.
```

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

o que ficou de fora, e por quê:

- `crates/rec/src/event.rs`: o doc de `Kind::Changes` ainda diz que um
  `Change` "does not carry the new node or the new order"; a primeira
  metade não vale mais. uma linha, fora dos meus arquivos.
- `docs/ADR/session-recording-replay-not-reexecution.md` diz o mesmo
  ("a `Change` carries neither the new node nor the new order") e
  `docs/ADR/lean-semantic-form-and-one-redaction.md` diz que "a `delta`
  never carried `source`". o ADR novo corrige os dois; uma nota em cima
  de cada um fica para o close-out.
- `crates/ide/src/bin/lab/lab.rs` já tinha 728 linhas na main (acima das
  639) e foi a 733; a formatação de `acted` foi para `story.rs` para não
  crescer mais.
- `docs/arc/arc.toml` (papel do `sem`): cabe dizer que um `node_added`
  carrega o nó e que o dispatcher nomeia em `acted` quem agiu sobre o
  estado visto. `docs/how-to/code-against-the-plev-engine.md` (seção do
  contrato semântico) pode ganhar a mesma frase. os dois são do
  close-out.
- fora do escopo pelo arquivo da task: aplicar `NodeAdded` no recorder
  sem snapshot inteiro, `files` no nó do commit (task-85), a corrida do
  `Refresh` (task-84), eco do `set_value`.

riscos de integração: quem casar `Rejection::Stale { changes }` numa
branch aberta precisa de `..` (a task-84 mexe no ide; os arquivos dela,
`session.rs`, `views/` e `crates/git`, não foram tocados aqui). a task-85
põe `files` no nó do commit, que passa a viajar dentro do `node_added`
sem mais nada.

linha proposta para `docs/ADR/README.md`:

```text
| 2026-09-23 | [a-delta-carries-the-node-a-refusal-names-who-acted](a-delta-carries-the-node-a-refusal-names-who-acted.md) | accepted | `Change::NodeAdded` carrega o nó como o snapshot `after` o tem, e o agent port o serve com o mesmo filtro de `keys` e `source` do snapshot (o delta que mostra o commit nascendo em `stacks`: 172 B mais uma observação de 402 B viram 427 B numa ida só); `Rejection::Stale` ganha `acted`, as ações de outros atores checadas na revisão que o autor viu e que rodaram, preenchido só pelo dispatcher; a regra de stale fica no `plev --help` e no doc do `Act`; protocolo continua v1, e sessão gravada antes com nó adicionado não abre mais (task-83) |
```
