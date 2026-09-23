---
project: plev
audience: [orchestrator, ai-agents]
type: learnings
tags: [learnings, hypotheses, process]
---

# learnings

o que experimentos e implementações ensinaram, e o que mudou no programa por
causa disso. só aprendizado concreto, com a fonte. o mais novo em cima.

## produto e tese

### 2026-09-23, checkpoint a (task-62 fase c)

fonte: `steps/ongoing/task-62-ide-human-agent-proof.md`, "resultado da fase
c"; saídas em `target/agents/exp/`.

- **confirmado (H1, H2, H3, H5):** um agente LLM sem contexto do repositório
  opera o ide só pelo contrato (`plev observe`/`act`), sem screenshot e sem
  `git`; a ação sobre estado velho é recusada antes do efeito, com o delta,
  e o HEAD não se move; o agente relê, repara e conclui; o receipt gravado
  na sessão bate com o git real.
- **onde o plev ganha (H4):** a recusa vem antes do efeito e nomeia o que
  mudou. no braço `git` com a mesma intervenção, o commit errado existiu, o
  git não avisou (exit 0), e o conserto foi `commit --amend`, que só
  funciona em commit local.
- **onde não ganha (H4):** volume e idas e voltas. 22 KB em 30 mensagens
  (plev) contra 1,5 KB em 20 invocações (`git`). tokens comparáveis. uma
  conexão por comando do `plev` pesa nas idas e voltas.
- **o que mudou no programa:** as três lacunas de contrato que os agentes
  apontaram viraram a task-83 (delta com o nó, recusa com autor, regra de
  stale escrita) e a corrida do `Refresh` virou a task-84, antes da onda 5
  (decisão do brenner: "correções primeiro"). a onda 5 espera o
  experimento rodar de novo com as correções.
- **a medir, não a supor:** se a conexão persistente (um processo `plev`
  por sessão de agente) muda o placar de idas e voltas. é ferramenta, não
  contrato; entra só se o re-run mostrar que o custo importa.

### 2026-09-23, forma enxuta (task-57b)

- a observação inteira do todo caiu de 6419 para 2039 B (3 itens) e de 87
  para 26 KB (103 itens) só tirando `source` e campos no default. o custo de
  token do contrato era forma, não conteúdo.

## processo

### 2026-09-23

- **o gate local divergia do CI:** a 66b deixou `ShapedGlyph` e
  `shaped_glyphs` como código morto no wasm32 (o único leitor, o export
  svg, é só nativo). o CI (`RUSTFLAGS=-D warnings`) reprovou os jobs wasm e
  urnaui do PR #45; o gate local passou porque a perna wasm rodava sem `-D
  warnings` e nunca checava o urnaui no wasm32. a classe: uma perna do
  gate mais frouxa que a do CI deixa passar o que o CI reprova. o gate
  agora roda as duas pernas wasm como o CI (`c07a7f0`); a falha reproduziu
  localmente antes da correção e sumiu depois. regra: toda perna do CI tem
  a mesma perna, com as mesmas flags, em `script/gate`.

- **graça do laboratório:** o laboratório fechava 1 s depois do commit e
  derrubou a verificação final de dois agentes (A2, B2). a classe: um
  harness que encerra no primeiro sinal de sucesso mede o sucesso e apaga a
  verificação. o `--grace` passou a ser parte do protocolo (180 s nas
  corridas refeitas). regra para os próximos experimentos: o harness só
  encerra depois que o agente disse que terminou, ou no timeout.
- **agentes parados:** o brenner viu agentes 16 minutos parados numa task.
  o ciclo autônomo agora volta a cada 10 min com agente rodando e confere
  commits e arquivos alterados; parado, pergunta; parado de novo,
  rebriefing (`autonomy.md`, "máquina").
- **disco:** três worktrees compilando em paralelo encheram o disco. um
  target dir por agente, incremental desligado, debug só com line tables;
  no máximo dois ou três agentes compilando ao mesmo tempo com 30 GB
  livres.
