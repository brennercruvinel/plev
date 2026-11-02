---
id: task-87
title: "task-87: a terminal stage carries the revision the app was left in"
project: plev
type: task
status: done
date: 2026-09-23
date_basis: header
recorded: 2026-09-23
topics: [agents, semantics, performance]
relations: [adr-0039, adr-0044, task-61, task-83, task-84, task-85, task-88]
evidence: [1cfb982, 06b35ed, 0c01560, 5fc730b, 365d8fd]
source: docs/log/steps/task-87-a-terminal-stage-carries-the-revision.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-87: a terminal stage carries the revision the app was left in

Makes the terminal stage of an act (`completed`, `rejected` or `failed`) carry `revision`, the revision the app was in when the stage ended, so that the agent can chain the next act without an observation just to learn the revision. The field is additive in protocol v1 and sits in the same ring that `observe` serves. In the scripted lab, the client went from 16 to 12 messages without intervention and from 24 to 17 when the person acted; the projection costs a median of 158 µs in the none scenario (172 µs in unstage), in debug. Integrated on `main` at `365d8fd`.

## por que

na segunda rodada do checkpoint a, os três agentes LLM disseram a mesma
coisa: o `completed` de um ato não traz a revisão nova, então para
encadear o próximo ato o agente precisa de uma observação só para saber
a revisão. é o maior custo de idas e voltas que sobrou, e idas e voltas
são onde o plev perde para o `git` direto (H4). um agente que age
`--from-revision` precisa de uma revisão servida; hoje a única fonte é o
`observe`.

## estado no código (main @ 5be83c5)

- `rust/socket/src/protocol.rs:197-203`: `StageReply { request, stage, at,
  asked }`, sem revisão.
- `rust/socket/src/server.rs:225-261`: `Server::forward` manda os
  estágios que o sink ouviu, sem acesso ao `Host`; chamado em `:219` (fim
  do `handle`) e em `rust/socket/src/port.rs:296` (fim do `drain`).
- `rust/socket/src/ring.rs:52`: `Ring::serve(snapshot) -> Revision`
  guarda um snapshot servido; `since` e `from_revision` só aceitam
  revisão que esteja no anel.

## escopo

um estágio terminal (`completed`, `rejected`, `failed`) servido a um
agente leva `revision`: a revisão da projeção inteira do app no instante
em que o estágio sai, guardada no anel como toda revisão servida. aditivo
no protocolo v1. mais a frase do `--keys` no `plev --help` (ele corta só
mudanças de estado; outras mudanças passam), que o controle da segunda
rodada estranhou.

## fora do escopo

por que um controle está desabilitado; o que `constrained` restringe;
conexão persistente do `plev`.

## aceite

teste no wire: o `revision` de um `completed` é aceito por `observe
--since` e por `act --from-revision`; o laboratório roteirizado usa a
revisão do `completed` onde hoje observa só para saber a revisão, com as
mensagens antes e depois no arquivo da task; gate verde.

## medição (laboratório roteirizado)

`ide-lab --arm scripted`, binário de debug, cinco corridas por cenário
antes (base, `main @ f741246`) e depois (esta branch), todas com exit 0 e
verdict MATCH. mensagens e bytes são os do `Traffic` da porta (a linha
com o `\n`); as mensagens do cliente contam o `hello`.

| | none, antes | none, depois | unstage, antes | unstage, depois |
| --- | --- | --- | --- | --- |
| mensagens do cliente | 16 | 12 | 24 | 17 |
| mensagens do servidor | 34 | 30 | 46 | 39 |
| observações (snapshot / delta) | 9 / 0 | 5 / 0 | 14 / 1 | 7 / 1 |
| atos / estágios enviados | 6 / 24 | 6 / 24 | 8 / 30 | 8 / 30 |
| bytes do cliente | 1132 | 992 | 1601 | 1356 |
| bytes do servidor | 54453 a 54473 | 28782 a 28791 | 91502 a 91507 | 45874 a 45878 |
| recusa stale / medição válida / recuperou | 0 / não / não | 0 / não / não | 1 / sim / sim | 1 / sim / sim |

o cenário unstage é `unstage-after-observe:src/session.rs`. as cinco
corridas de cada linha deram os mesmos números, fora os bytes do
servidor, que variam na faixa escrita (os tempos dentro das mensagens).

o que saiu: a observação depois de cada ato de arquivo (três), a
observação depois do `open` e, no reparo, a observação depois de cada
arquivo (três). o que ficou, porque decide: a primeira observação (quais
arquivos entram e saem), a que lê se o formulário está aberto, a decisão
antes de cada submit (é nela que a pessoa do roteiro age), o delta depois
da recusa, a observação do índice que decide o reparo, a observação de
`stacks` que confirma o commit, e a do workspace inteiro depois dele (só
pelo peso, como antes). a linha de um estágio terminal ficou 30 bytes
maior (`,"revision":"<16 hex>"`); cada observação inteira que saiu pesava
uns 6,5 KB.

custo: a projeção do `forward` (projetar e servir no anel), medida com um
build instrumentado que não foi commitado, deu mediana de 158 us no
cenário none e 172 us no unstage, máximo de 306 us, em 70 chamadas (uma
por estágio terminal; nenhum `forward` mandou dois terminais juntos), em
debug. o limite da task era 2 ms. as chamadas a `project` do laboratório
foram de 15 para 17 (none) e de 31 para 28 (unstage), a uns 120 us cada.

a revisão engana? nas 25 vezes em que um ato foi seguido direto por uma
observação (o último ato de arquivo e a leitura do formulário, o
`set_value` e a decisão, o reparo e a nova decisão), a observação serviu
exatamente a revisão que o estágio terminal daquele ato trouxe: o ide
mostra o efeito de um ato quando ele termina (task-84), e a projeção do
`forward` vê isso. o que a revisão não traz, e uma observação no mesmo
instante também não traria, está no ADR
([`0044-a-terminal-stage-carries-the-revision.md`](../../adr/Semantics/0044-a-terminal-stage-carries-the-revision/0044-a-terminal-stage-carries-the-revision.md), consequências): o
`show_diff` termina antes das linhas chegarem, e um stage que falha no
git deixa o palpite da view até o refresh.

o relatório do laboratório rotula como `observed` o `form` dos atos que
se apoiam na revisão de um estágio (`FooFoF` no none, `FooFoFFF` no
unstage, onde F é `from_revision`): `story::acts` só conhece as
observações servidas. o teste do laboratório deixou de ler o `form` e
confere que cada ato se apoia numa revisão servida, por observação ou
por estágio (o script escreve "the app at <rev>" depois de cada ato).

## handoff

```text
Goal: task-87, um estágio terminal traz a revisão em que o app ficou.
Status: pronto para integrar; gate verde na branch task/87-stage-revision.

Implementado:
- StageReply.revision (Option<Revision>, default + skip_serializing_if):
  presente só num estágio terminal (completed, rejected, failed), ausente
  nos outros e quando o app não projeta nada naquele instante.
- Server::forward(&mut self, host: &mut dyn Host): projeta pelo host e
  serve no anel pelo caminho do observe (host.project(), ring.serve),
  uma vez por chamada e só quando sai um terminal de um pedido desta
  porta. os dois pontos de chamada passam o host que já tinham (fim do
  handle, fim do AgentPort::drain).
- o significado no doc do StageReply, no plev --help (e plev act --help)
  e no ADR: a revisão em que o app ficou quando o estágio terminal saiu;
  inclui o que outros atores mudaram até ali; serve para --since e
  --from-revision. não decide nada (a checagem continua pelos reads).
- plev --help, na parte do observe: --keys corta só estado (valores e
  mudanças das outras chaves); nó novo ou removido, label, ações, flags,
  parent, ordem e foco passam sempre.
- o agente roteirizado encadeia na revisão do terminal onde observava só
  para saber a revisão; mantém as observações que decidem.

Arquivos principais:
- rust/socket/src/{proto.rs,server.rs,port.rs,lib.rs,bin/plev.rs}
- rust/socket/tests/{server.rs,codec.rs,traffic.rs,cli.rs}
- apps/agenteval/src/arms/scripted.rs, apps/agenteval/tests/experiment.rs
- docs/adr/semantics/0044-a-terminal-stage-carries-the-revision.md (novo),
  docs/adr/semantics/0039-agents-act-through-the-semantic-contract.md (consequências)

Interfaces adicionadas/alteradas:
- wire::StageReply ganha o campo pub revision (quem monta a struct à mão
  precisa dele; no repositório, só os testes do wire montavam).
- wire::Server::forward passa a receber &mut dyn Host. AgentPort::drain
  não mudou, então o showcase e o ide-lab não mudaram na chamada.
- Host não mudou de forma; o doc de Host::observed diz que a revisão de
  um estágio não é reportada ali.
- protocolo v1, aditivo: cliente antigo ignora o campo; estágio antigo
  (sem o campo) lê como sem revisão.

Testes:
- server.rs: completed traz a revisão (igual à projeção depois do ato,
  guardada no anel); --since com ela dá delta; --from-revision com ela é
  aceito, duas vezes em cadeia; o rejected stale traz a revisão com a
  mudança da pessoa, e um ato nela roda; requested, accepted, executing
  e awaiting_confirmation não trazem; o completed tardio de um sync traz
  o app como está quando sai; uma projeção serve todos os terminais de
  um forward, e nenhuma quando não sai terminal; app sem projeção: campo
  ausente; o anel continua limitado e as revisões de estágio empurram a
  observada para fora; o pin secreto continua comprimento (a revisão não
  muda com os quatro dígitos).
- codec.rs: a forma com e sem o campo, um StageReply antigo lê, revisão
  inválida é recusada. traffic.rs: observes_* e Host::observed não contam
  a revisão de um estágio. cli.rs: plev imprime, plev observe --since e
  plev act --from-revision aceitam. plev.rs: as frases do --help.
- lab.rs: cada ato se apoia numa revisão servida (por observação ou por
  estágio) e o agente encadeou pelo menos um ato, nos dois cenários.

Validação executada:
- ./tool/tasks/fullcheck.sh verde (test, clippy -D warnings, fmt, wasm -D warnings,
  perna urnaui no subdiretório do target dir): 2102 passados, 0
  falhas, 12 ignorados. a main tinha 2096 no gate da task-85; +6 testes
  novos (cinco em wire/tests/server.rs, um em plev.rs).
- cargo test -p socket, -p idestudio e -p uigallery verdes (dentro do gate).
- ide-lab --arm scripted, cinco vezes em none e cinco em
  unstage-after-observe:src/session.rs, antes (main @ f741246) e depois:
  todas exit 0 e MATCH; a tabela está na seção "medição" acima.

Decisões:
- Traffic: a revisão de um estágio não é observação que o agente pediu;
  observes_snapshot e observes_delta não mudam e não há contador novo
  (stages_out já conta o estágio).
- Host::observed não é chamado para ela, pela mesma razão: o agente
  recebe o nome de um estado, não o estado; o gravador (rec::Kind::
  Observed, "an actor observed the app") continua gravando só
  observações.
- ADR curto novo em vez de só uma entrada nas consequências: a decisão
  tem onde, como, o que não é, o anel e o caso do app que ainda assenta.
- o agente roteirizado mantém a observação de decisão antes de cada
  submit: é onde ele decide, e é nela que a pessoa do roteiro age.

Limitações:
- o relatório do ide-lab (story::acts, apps/agenteval/src/story.rs,
  fora dos arquivos permitidos) só conhece as observações servidas: o
  form de um ato que se apoia na revisão de um estágio sai "observed", e
  served_index, observations_between e fresh_before não o veem. o teste
  deixou de ler o form.
- a pessoa do cenário unstage-after-observe só age depois de uma
  observação servida (host.rs, Host::observed). um agente que encadeia
  tudo na revisão dos estágios e não observa antes do submit nunca a faz
  agir, e a medição do stale sai inválida.
- a revisão é o que uma observação no mesmo instante serviria. no ide, o
  show_diff termina antes das linhas chegarem, e um stage que falha no
  git deixa o palpite da view até o refresh (ver o ADR).

Dependências pendentes:
- nenhuma para integrar. para a onda 5 com agentes LLM, as duas
  limitações do laboratório acima (story e pessoa) precisam de decisão.

Riscos de integração:
- o anel é da porta, dividido entre as conexões, com 16 revisões por
  padrão: agora cada ato que termina ocupa um lugar (ou renova um que já
  está lá). um agente que guarda uma revisão observada enquanto 16 outras
  são servidas recebe unknown_revision e observa de novo.
- a task 82b mexe no shell do showcase e no ide. esta branch não mudou
  o showcase, e no ide mudou só o laboratório (bin/lab/scripted.rs e
  tests/lab.rs).
- recomendação ao orquestrador: numa task do laboratório, fazer o ide-lab
  ouvir os estágios que carregam revisão (um gancho no Host, ou o lab
  lendo o Traffic por estágio) para o story contar a revisão de um
  estágio como servida, e decidir se a pessoa também age quando um
  estágio serve o estado pronto.

Commits:
- 22d32e9 task-87: a terminal stage carries the revision the app is at
- aa9a0e8 task-87: the wire tests of the revision a terminal stage carries
- 43f25eb task-87: plev --help and the ADR say what the revision of a last stage is
- 55a3fdd task-87: the scripted agent chains its acts on the revision a last stage carries
- (este) task-87: handoff
```

linha proposta para [`adr/`](../../adr/README.md), depois da do task-83:

```text
| 2026-09-23 | [a-terminal-stage-carries-the-revision](a-terminal-stage-carries-the-revision.md) | accepted | um estágio terminal (`completed`, `rejected`, `failed`) traz `revision`, a revisão em que o app ficou quando ele saiu, com o que outros atores mudaram até ali, servida no anel como a do `observe` e aceita por `since` e `from_revision`; uma projeção por `forward`, só quando sai um terminal (mediana de 158 us sem ninguém intervindo e 172 us com a pessoa, no laboratório, em debug); não é observação (`Traffic` e `Host::observed` não contam) e não decide nada; protocolo continua v1; `plev --help` diz também o que `--keys` corta; o agente roteirizado foi de 16 para 12 mensagens sem intervenção e de 24 para 17 com a pessoa, com metade dos bytes (task-87) |
```

## fechamento (orquestrador, 2026-09-23)

integrada na `main` em `365d8fd` (cinco commits, rebaseados sobre a
`main` sem conflito). validação: diff inteiro lido; o `forward` projeta
uma vez por chamada e só quando sai um terminal, e é chamado no fim do
`handle` (ato síncrono, todo) e no fim do `drain` (ato assíncrono, ide);
gate verde no diretório da task, com a perna urnaui: 2102 passaram, 0
falharam, 12 ignorados; laboratório reproduzido pelo orquestrador (12
mensagens e 28,8 KB sem intervenção, 17 e 45,9 KB com, recusa válida,
MATCH). o risco do anel (16 revisões, cada ato concluído ocupa uma) fica
como está: um agente que perde a revisão recebe `unknown_revision` e
observa de novo, falha segura. os dois problemas do laboratório viraram a
task-88.
