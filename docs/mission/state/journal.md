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

### ciclo 1: task-83 integrada

- **integrada:** task-83 (`78f463c`): delta com o nó, `acted` na recusa,
  regra de stale no `plev --help`, sessões antigas legíveis. gate verde
  (2084), laboratório reproduzido pelo orquestrador (a recusa nomeia só a
  pessoa).
- **corrigido:** o gate com a perna urnaui num subdiretório próprio do
  target dir compartilhado (`1bf6606`).
- **devolvida:** task-85, com duas correções: uma definição só dos
  arquivos de um commit (receipt e nó concordavam até o refresh e
  discordavam depois, em renomeação), e `files` só nos commits mais
  recentes (81 KB a mais na observação inteira deste repositório era
  inaceitável).

### ciclo 1: docs voltam a dizer o que o código faz

- **integrado:** as correções de docs (`2d4aa4b`, 8 commits: README,
  arc.toml, manual, nove ADRs, task-62). as afirmações falsas saíram; as
  promessas de design que o código ainda não cumpre ficaram, com a
  exceção de hoje e a task que a fecha. `arc_sync_guard` verde.
- **bug achado pelo agente de docs:** `cargo run -p ide` não rodava desde
  a fase c da task-62 (dois binários, sem `default-run`). corrigido no
  manifesto.

### ciclo 1: task-83 devolvida com uma correção

- a task-83 voltou pronta (delta com o nó, `acted` na recusa, regra no
  `plev --help`, gate verde com 1995 no workspace) e com uma decisão: a
  forma nova do `node_added` fazia as sessões gravadas antes (as do
  checkpoint a) não abrirem.
- **decisão autônoma:** ler a forma antiga convertendo (o nó adicionado de
  um evento de snapshot é o nó do próprio snapshot: sem perda), sem subir
  a versão do `MONS`. motivo: é a resposta conservadora e mantém a
  evidência do experimento legível para a task-64. devolvida ao mesmo
  agente com a correção específica.

### ciclo 1: replanejamento pela verdade

- **disparados:** agente de docs (`task/docs-follow-code`, sem build:
  corrige as divergências 1 a 25, 31 e 32 do `truth.md`) e task-82 fase b
  (`task/82b-wake-everywhere`).
- **decisão autônoma:** a fase b da task-82 sobe na fila. motivo: a
  auditoria mostrou que a promessa "no change, no frame" do README está
  quebrada no ide, o app da prova da tese (60 fps com o campo focado ou
  com toast), e na página Git do showcase; é invariante quebrada, a
  primeira na ordem do ciclo.
- **próximo, já escolhido:** o experimento do checkpoint a de novo quando
  83 e 85 entrarem; depois, o binário `ide` abrindo a porta do agente
  (opt-in, como o showcase), para o cenário da tese rodar num app que
  alguém usa.

### ciclo 1: task-84 integrada, verdade reconstruída

- **integrada:** task-84 (`4896451`), a corrida do `Refresh`. gate verde
  (2073 passaram), laboratório reproduzido pelo orquestrador (seis
  corridas certas).
- **integrada:** a auditoria de verdade (`7621403`),
  `docs/mission/state/truth.md`. achados que mudam o plano: o binário
  `ide` não abre a porta do agente (o checkpoint a rodou só no
  `ide-lab`); digitar não passa pelo dispatcher; 32 divergências entre
  docs e código.
- **publicado:** PR #45 (os 28 commits atrasados, com a correção do
  wasm), merge commit `1ecc84c`.
- **disparada:** task-85 (o commit que o receipt anuncia já está na
  projeção, com `files`), no lugar que a 84 liberou.
- **decisão autônoma:** as 32 divergências se corrigem antes de planejar
  a partir dos docs; um agente de docs, sem build, com `truth.md` como
  fonte; o orquestrador corrige o programa.

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
