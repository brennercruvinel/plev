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

### 2026-09-23, task-85

- **o receipt e a projeção podem se contradizer sem ninguém mentir:** o
  receipt dizia "commitado" e a projeção dizia "ainda staged, sem commit"
  até o refresh. cada um estava certo no seu tempo; o agente que
  confere logo depois via um mundo que contradizia o receipt que acabou
  de receber. regra que ficou: o app mostra o efeito no passo em que o
  receipt conclui.
- **duas fontes para o mesmo fato divergem:** os arquivos do commit
  vinham de `git show --name-only` no receipt e do diff de árvores no
  log; concordavam até o refresh e discordavam depois, numa renomeação.
  uma definição só, lida por uma função só.
- **o custo de uma observação inteira do ide é o log:** neste repositório
  a observação inteira tem uns 43 KB, 97% dela o `stacks` com 100 commits.
  pôr `files` em todos triplicaria. um agente que observa por escopo não
  paga isso; o limite (`files` nos 10 mais recentes) mantém o aumento em
  3,8%. quantos commits a projeção carrega é uma pergunta que ainda não
  foi feita.

### 2026-09-23, task-83

- **confirmado no laboratório:** a regra exata do `acted` (ações de
  outros atores checadas na mesma revisão que o autor viu e que rodaram)
  nomeia no cenário da intervenção exatamente a pessoa que tirou o
  arquivo do index, e ninguém mais. a atribuição útil saiu barata porque
  o dispatcher já guardava `validated_against` em cada receipt.
- **bytes, pela primeira vez a favor:** o commit nascendo em `stacks`
  custava ao agente 172 B de delta mais 402 B de uma segunda observação;
  com o nó no delta, 427 B numa ida. o delta de um item novo no todo
  subiu de 423 para 647 B, ainda 28% da observação inteira com 3 itens.
- **limite que continua:** uma mudança de fora do app e uma digitação
  (sem request) nunca aparecem em `acted`; a recusa diz o que mudou e
  fica calada sobre quem.

### 2026-09-23, task-84 e auditoria de verdade

fonte: `steps/checked/task-84-ide-refresh-never-goes-back-done.md`,
`state/truth.md`.

- **pressuposto errado:** "a view é otimista e reconcilia com um refresh"
  era tratado como inofensivo. não era: um refresh lido antes de uma
  mutação pousava por cima dela, e a projeção mostrava por um instante um
  estado que não era nem o do git nem o que o ide aceitou. o agente
  roteirizado sem a espera que o contornava deu errado nas dez corridas
  com intervenção na base (nove recusas à toa ou decisões erradas, e um
  commit de dois arquivos em vez de três).
- **o que isso ensina sobre o contrato:** a checagem de stale protege
  contra o mundo que mudou, não contra o app que mostra um mundo errado.
  o contrato ficou coerente o tempo todo; quem mentiu foi a projeção.
  projeção verdadeira é uma invariante de cada `Operable`, que o `sem`
  não tem como garantir: cada app precisa de teste que prove que a
  projeção nunca fica mais velha que uma mutação aceita e converge para
  a verdade.
- **o checkpoint a teve sorte que não mediu:** os dois agentes LLM
  escaparam da corrida por tempo. H1 e H2 continuam de pé, mas a
  confiabilidade do resultado dependia de algo que o experimento não
  controlava. a nova rodada do experimento, depois das tasks 83 a 85, é
  necessária também por isso.
- **o cenário da tese ainda não rodou num app de verdade:** o binário
  `ide` não abre a porta do agente; o checkpoint a rodou no `ide-lab`,
  com o workbench sem janela e uma pessoa roteirizada. "uma pessoa usa o
  app enquanto um agente o opera" nunca aconteceu num app que alguém usa.
- **digitação fora do contrato:** no todo e no ide, digitar muda o texto
  projetado sem request nem receipt (o ADR de ligação isenta de
  propósito). consequência: nem o `acted` da task-83 nem o recorder sabem
  dizer quem digitou. fica como limite registrado até a task-64
  (inspector) precisar.
- **cobertura semântica fina:** dos 19 widgets com `project`, só
  `TextField` e `Toast` têm chamador de produção; só o todo e o ide
  projetam. para superfícies geradas (69) isso vai importar.

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

- **dois workspaces num target dir:** com a perna urnaui rodando nos
  worktrees, o workspace raiz e o do urnaui passaram a gravar no mesmo
  `CARGO_TARGET_DIR`; o engine, com crate-type cdylib e staticlib, grava
  `libengine.rlib` sem hash no nome, e um workspace pegava o arquivo do
  outro ("multiple different versions of crate `sem`"). a classe:
  artefato sem hash compartilhado entre workspaces. o gate agora dá à
  perna urnaui um subdiretório próprio (`1bf6606`). achado pelo agente da
  task-83, que o contornou com `cargo clean -p engine` e reportou em vez
  de mexer no gate.

- **premissa de compatibilidade errada no briefing:** o briefing da
  task-83 afirmou que nenhum arquivo gravado continha `node_added`,
  conferido só no `Kind::Changes` do recorder. o `Kind::Snapshot` guarda o
  diff do passo, com `node_added`, e as sessões do checkpoint a deixariam
  de abrir. o agente achou abrindo as sessões de verdade. regra: toda
  afirmação de compatibilidade num briefing se confere contra os
  artefatos gravados de verdade (abrir os arquivos), não só lendo o
  código do escritor.

- **contorno de harness escondeu defeito de produto:** o agente
  roteirizado esperava o index "parar" (até 200 observações), e isso
  escondia a corrida do `Refresh`. regra: um harness ou agente que
  espera o app "assentar" antes de agir é sintoma; a espera vira bug do
  app, registrado, nunca contorno silencioso.
- **a perna urnaui do gate pulava em todo worktree:** o gate procura
  `../urna` a partir do worktree, que não existe. o symlink
  `.claude/worktrees/urna` fez a perna rodar (a primeira vez foi no gate
  da task-84).
- **32 divergências doc x código acumuladas:** os fechamentos escreviam
  o doc novo e não corrigiam os docs que a mudança contradizia (a 58b
  trouxe `decline` e o ADR commit-checks continuou dizendo que não
  existia). regra nova do fechamento: procurar em `docs/` e no README os
  nomes do contrato que mudou e corrigir o que ficou falso
  (`docs/how-to/orchestrate-coding-agents-on-this-repo.md`).

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
