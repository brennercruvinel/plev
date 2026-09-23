---
project: plev
audience: [orchestrator, brenner]
type: journal
tags: [journal, autonomy, status]
---

# journal do ciclo autônomo

uma entrada por acontecimento que importa para o `status`: intervenção do
brenner, integração, decisão autônoma, bug relevante, agente disparado ou
devolvido, publicação. o mais novo em cima. o detalhe fica no arquivo da
task; aqui fica o que aconteceu e onde ler.

## 2026-09-23

### ciclo 1: CI do PR #45 pegou um defeito que o gate deixou passar

- o PR #45 (os 28 commits que não estavam no GitHub) falhou nos jobs wasm
  e urnaui: código morto no wasm32 vindo da 66b. corrigido na `main`
  (`c07a7f0`) junto com o gate, que agora roda as pernas wasm como o CI.
  detalhe em `learnings.md`, processo.

### ciclo 1 aberto

- **disparados:** task-83 (delta com o nó, recusa com autor, regra de stale
  escrita; `task/83-delta-and-refusal`), task-84 (corrida do `Refresh` no
  ide; `task/84-refresh-order`), auditoria de verdade (`task/truth-audit`,
  só leitura; escreve `state/truth.md`).
- **decisão autônoma:** a recusa com autor usa uma regra exata e limitada
  (ações de outros atores checadas sobre a mesma revisão que o autor viu e
  que chegaram a rodar), preenchida só pelo `Dispatcher`. motivo: a revisão
  é um hash, não uma ordem; atribuição completa de uma cadeia de mudanças
  pede o histórico que o recorder tem, e isso é pergunta da task-64
  (inspector). o limite vai no doc e no ADR.
- **decisão autônoma:** `files` no nó do commit sai da task-83 e vira a
  task-85 (git e projeção do ide), depois da task-84, porque as duas mexem
  na `GitSession` e no `crates/git`.
- **decisão autônoma:** a onda 5 espera o experimento do checkpoint a rodar
  de novo com as correções (é o "outro checkpoint" que o brenner pediu);
  se H1 a H5 se sustentarem e as lacunas fecharem, a onda 5 abre sem
  parar.

### intervenção do brenner

- respondeu ao checkpoint a: **correções primeiro** (as três lacunas do
  contrato e a corrida do `Refresh`), a onda 5 espera outro checkpoint.
- checkpoints futuros: parar e avisar. publicação: **a cada integração**
  (PR com merge commit). máquina: `caffeinate`, notificação do macOS,
  allowlist de comandos de leitura.
- mudou o modo de operação para **orquestrador autônomo**
  (`docs/mission/autonomy.md`): ciclo observe → choose → brief → implement
  → validate → learn → update → replan, parada só nas condições listadas.
  os checkpoints param quando caem nessas condições.
- estado da `main` nesse momento: `c5fdf36` (57b, 62c, 66b, 82a
  integradas), 26 commits à frente do GitHub.
