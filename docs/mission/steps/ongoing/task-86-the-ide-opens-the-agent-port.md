---
project: plev
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2026-09-23
domain: agent-surface
task: task-86
depends: [task-61, task-62, task-82]
branch: task/86-ide-port
origin: auditoria de verdade (truth.md, seção 3, linha 21) e checkpoint a
---

# task-86: o ide que alguém usa abre a porta do agente

## por que

o cenário da tese é "uma pessoa usa o app enquanto um agente o opera, os
dois pelo mesmo contrato". os dois checkpoints a rodaram no `ide-lab`: o
workbench sem janela e uma pessoa roteirizada. o binário `ide`, o que
alguém abre de verdade, não usa `wire` e não abre porta nenhuma
(`crates/ide/src/main.rs`). enquanto isso não existir, a afirmação
principal do programa só está provada em laboratório, e o teste que só o
brenner pode fazer (ele usando o ide enquanto um agente age) não tem como
acontecer.

## estado no código (main @ 91a7319)

- `crates/ide/src/main.rs`: o shell do ide (winit), com o `Workbench`,
  o worker do git, o watcher, e desde a task-82 o `pace` e o tick de
  alcance. 655 linhas, acima do limite de 639: precisa dividir para
  crescer.
- `crates/ide/src/bin/lab/host.rs:232-263`: o `Host` do laboratório
  sobre o `Workbench`, com gravação, custos e a pessoa roteirizada.
- `crates/showcase/src/view/dispatch.rs:322-371` e
  `crates/showcase/src/app.rs:178-185`, `:295`, `:486-494`: como o
  showcase abre a porta (`AgentPort::open`, `PLEV_AGENT=1` em debug),
  acorda o loop (`UserEvent::Agent`), serve no thread do app e desenha só
  se um ato rodou; o `bounds` com o retângulo visível de cada ligação.
- o laboratório drena a porta também depois de cada evento do worker do
  git, porque os estágios tardios (execução que termina no worker) só
  saem assim.

## escopo

1. um `Host` do `Workbench` na lib do ide (projeção, `bounds` das
   ligações do frame, dispatch, sinks), usado pelo shell; o laboratório
   pode delegar a ele a parte comum.
2. o shell do ide abre a porta como o showcase (opt-in, `PLEV_AGENT=1`
   em debug), acorda por evento, drena no thread do app, drena de novo
   depois de cada evento do worker, desenha só quando um ato rodou.
3. prova numa janela de verdade: o ide aberto num repositório, um agente
   pelo `plev` observa, faz stage e commita; e uma pessoa (teclado ou
   clique real) mexendo ao mesmo tempo, com uma recusa stale vista pelos
   dois lados.

## fora do escopo

a ponte de acessibilidade no ide (task-75); a página Git do showcase
alcançável por agente; o laboratório (task-88).

## aceite

teste do `Host` da lib por uma porta real sem janela; o ide real servindo
`plev observe`/`act` com saída colada; a cena com a pessoa e o agente e a
recusa; gate verde.
