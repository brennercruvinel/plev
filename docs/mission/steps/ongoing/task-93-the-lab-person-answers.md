---
project: plev
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2026-09-24
domain: experiment
task: task-93
depends: [task-88, task-92]
branch: task/93-lab-person-answers
origin: handoff da task-92 (o laboratório não tem pessoa que responde ao modal)
severity: P2 (habilita o experimento do handoff; nada que existe fica inválido)
---

# task-93: a pessoa do laboratório responde

## por que

a task-92 deu ao agente um jeito de perguntar à pessoa antes de agir
(`plev act --ask`), pela confirmação que o ide já mostra num modal. o
laboratório (`ide-lab`) tem uma pessoa roteirizada que tira um arquivo do
index. ela não responde a modal nenhum, então um agente que pergunta
fica esperando até o timeout. o próximo experimento com LLM (o agente
pergunta em vez de passar por cima ou parar?) precisa dessa pessoa.

## escopo

- um cenário novo da pessoa, que responde a toda confirmação pedida por
  um agente: permitir, recusar, ou não responder (o padrão de hoje),
  depois de um atraso configurável que o relatório diz;
- o relatório conta as perguntas (requests com `ask`), as respostas, o
  tempo até a resposta e o que o agente fez depois de cada uma;
- a gravação da sessão tem a resposta como input da pessoa;
- o roteirizado ganha um modo que pergunta antes de recolocar o arquivo,
  para o teste do gate.

## aceite

testes do laboratório: o roteirizado que pergunta, com a pessoa
permitindo (commit com os três arquivos) e recusando (sem commit, o
agente para); o relatório com as perguntas e as respostas; gate verde.

## sonda (2026-09-24, antes de implementar a pessoa)

na base (`e42e596`), com o modo novo do roteirizado feito primeiro (o item
3 do briefing, só o lado do agente): `--scripted ask` observa como o
padrão e, quando um `submit` volta stale com uma pessoa em `acted`,
pergunta antes de recolocar o arquivo (`Act::with_ask`) e espera a
resposta como o `plev act --ask` espera. ninguém responde.

```sh
ide-lab --arm scripted --scripted ask \
    --human unstage-before-submit:src/session.rs --timeout 5
```

exit 2 (sem commit), `ended by timeout` em 5001 ms, 13 mensagens in e 30
out. o `submit` 7 foi recusado stale nomeando a pessoa (request 6,
`human unstage changes.file[src/session.rs]`), o agente observou, e o
`stage` de `src/session.rs` com a pergunta (request 8) ficou assim até o
fim:

```text
act changes.file[src/session.rs] stage  awaiting_confirmation at 196 ms, request 8, ...
8 changes.file[src/session.rs] stage ['requested', 'awaiting_confirmation'] awaiting_confirmation
notes: the scripted agent: the app closed the connection
```

o agente só soube do fim porque a porta fechou no fim da corrida; sem o
`--timeout` curto, ele esperaria os 300 s do `ASK_TIMEOUT_MS` (o mesmo do
`plev act --ask`).

onde o `LabHost` enxerga o request esperando, sem mudar o `workbench`
(conferido com uma impressão descartável, fora do commit):

- em `LabHost::dispatch`, logo depois de `bench.dispatch(req)`:
  `bench.receipt(id).stage()` é `awaiting_confirmation`, e
  `receipt.request.ask` traz a pergunta. saída da sonda: `request 8:
  stage Some("awaiting_confirmation"), ask Some("you took src/session.rs
  out of the index, and the task commits it: stage it again?"),
  modal_open true`;
- o modal na tela: `bench.workspace().modal_open()` e o overlay
  `OverlayKind::Modal` em `workspace().overlay_mgr.stack`, com título
  "Question from scripted", corpo `scripted asks the person: "you took
  src/session.rs out of the index, and the task commits it: stage it
  again?" If allowed, it will stage src/session.rs.` e botão "Allow";
- os botões como desenhados no último frame:
  `workspace().modal_buttons()`, `([723, 437.4, 85, 40], [621, 437.4, 94,
  40])` (Allow e Cancel). `bench.click` no centro de um deles responde
  pelo mesmo caminho do modal: `UiRequest::Confirm` ou `Decline`,
  `Workbench::confirmed_by_person` ou `declined_by_person`, o dispatcher
  como `Actor::Human`. é o que os testes do ide já fazem
  (`tests/human_agent.rs`, `tests/agent_port.rs`);
- o sink `Log` do laboratório também ouve o estágio
  `awaiting_confirmation`, como ouve todos.

um achado para a gravação: `Compositor::hit` no centro dos dois botões do
modal devolve `stacks`, o controle embaixo dele, porque o modal não é
ligado a nenhum `SemId`. a janela do ide grava o clique de uma pessoa com
esse `hit` (`crates/ide/src/record.rs`, fora da lista), então um clique no
modal fica gravado como um clique em `stacks`. o laboratório não repete
isso: grava a resposta com o ponto do botão e sem alvo.
