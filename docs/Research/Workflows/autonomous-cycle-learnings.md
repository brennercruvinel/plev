---
id: learning-general
title: "what the autonomous cycle taught about the way of working"
project: plev
type: learning
status: reference
date: 2026-09
date_basis: text
recorded: 2026-09-23
topics: [agents, process]
relations: [operations-autonomy, operations-orchestrate-coding-agents-on-this-repo, research-autonomous-cycle-journal, task-70, task-75, task-90, task-86, task-83, task-84]
evidence: [c07a7f0, 1bf6606]
source: docs/log/learnings/general.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# what the autonomous cycle taught about the way of working

Records what the autonomous cycle of September 2026 taught about the way of working: briefings, gates, merges, worktrees, disk and the machine, in the words written at the time (in Portuguese), newest first, each with its source. Most rules here became part of [autonomy](autonomy.md) and of [how to orchestrate coding agents on this repo](orchestrate-coding-agents-on-this-repo.md); the [journal of the cycle](autonomous-cycle-journal.md) keeps the days. What the cycle taught about a single decision lives beside that ADR.

## 2026-09-25, o prazo do Claude

- **um prazo curto não economiza, desperdiça:** a chamada morta em 180 s
  já tinha gastado o que gastou. o custo é limitado pelo teto por chamada
  (`--max-budget-usd`); o prazo só decide se o gasto vira resposta. regra:
  quando o prompt pede mais saída, o prazo acompanha a mais lenta medida.
  para achar a causa sem gastar, um stub no `PLEV_CLAUDE` que só grava a
  stdin mostra o prompt exato. fonte: `rust/claude/src/lib.rs`,
  `TIMEOUT`.

## 2026-09-24, task-70 fase 2

- **"nunca chame o claude" no briefing não basta:** o agente não chamou,
  mas o regress que ele rodou chamou, porque o proponente escolhe o
  Claude sozinho quando ele existe no PATH. regra: todo comando de agente
  e do orquestrador fora da prova real roda com `PLEV_CLAUDE` apontando
  para um stub que sai com erro (o `ClaudeCli::exists` só olha se o
  arquivo existe, `rust/claude/src/lib.rs:159`, então a tela e as
  baselines ficam iguais). fonte: handoff da fase 2, incidente.

## 2026-09-25, o handoff virava a próxima fase

- **o ciclo gerava trabalho em vez de fechar:** o template de handoff
  pedia "o que fica para depois", e o orquestrador disparava essa lista
  como a fase seguinte da mesma task, porque estava à mão. as tasks
  abertas da lista ficaram paradas e cada agente custou de 500 a 660 mil
  tokens. regra: uma task fecha pelo critério de aceite do arquivo dela;
  todo briefing diz qual item do critério fecha; o que sobra vira
  "pendências", nunca fase ou task sem o ok do brenner; tudo esperando o
  brenner, o ciclo para ([`autonomy.md`](autonomy.md), "contra gasto de tokens").

## 2026-09-25, antes de dizer onde a correção mora, ver quem lê

- **o briefing da task-75 fase 2a mandou a página projetar as ações da
  linha como controles:** a projeção da página parada é golden do `uir`,
  é o registro contra o qual o IR se verifica e o prompt se monta, e
  entra no `scene.json` do regress. o agente viu na sonda e pôs os
  controles só na árvore do leitor. regra para o briefing: antes de
  prescrever onde uma correção mora, listar quem lê aquele dado (goldens,
  registro, prompt, port, regress) e deixar a sonda escolher o lugar.
- **"só o `semantics.json` muda" não existe:** o `scene.json` de uma
  baseline carrega a revisão do `semantics.json`, então toda mudança de
  significado mexe numa linha dele. a regra certa é "o `scene.json` só
  na linha `semantics.revision`; frame, camadas e bindings iguais".

## 2026-09-24, dois checkouts num target dir só

- **o cargo rodou um `regress` velho:** um script de captura com o
  `REPO_ROOT` errado compilou o exemplo do checkout principal em
  `target/agents/one`; o `cargo run` seguinte no worktree executou esse
  binário, porque o cargo não recopia o executável quando o build local
  está em dia. regra: um target dir serve a um checkout só; quem precisar
  compilar o checkout principal usa outro dir, e antes de um gate que
  valida um exemplo o orquestrador toca a fonte do exemplo para forçar o
  rebuild.

## 2026-09-24, `trash` não libera disco

- **o orquestrador mandou um build dir de 3 GB para a Lixeira:** o
  `/usr/bin/trash` move para a Lixeira do mesmo volume, então o espaço
  não volta, e esta sessão não tem permissão de ler a Lixeira para
  desfazer. esvaziar a Lixeira é do brenner (apaga as coisas dele
  também). regra: cache de build se limpa com `cargo clean` (com
  `--target-dir` ou `--profile`, e antes um `--dry-run` que mostra o que
  sai); `trash` fica para arquivo que alguém pode querer de volta.

## 2026-09-24, documentação de biblioteca não é fonte

- **o briefing do item 6 citou a doc do winit como fato:** a doc dizia
  que o winit não registra delegado de aplicação no macOS; a fonte da
  versão travada no lock registra, e trocar o delegado derruba o laço. o
  agente leu a fonte na sonda e parou. regra para o briefing: afirmação
  sobre biblioteca vem da fonte da versão do lock, com arquivo e linha,
  não da doc; e uma frase de handoff anterior ("o engine já tem X")
  se confere no manifesto antes de virar premissa.

## 2026-09-24, briefing com premissa velha (fase 4a)

- **"congelado" tinha deixado de valer:** o briefing da fase 4a chamou o
  `view/dispatch.rs` de congelado e proibiu tocá-lo; ele saiu da lista
  `FROZEN` na fase 1 e tem 338 linhas. o agente parou certo e perdeu uma
  rodada. quarto briefing do ciclo com premissa que o código já não
  sustentava (83, 86, 90, 4a). regra: toda proibição de arquivo num
  briefing vem com o motivo conferido na hora (`grep` no guard, `wc -l`),
  não copiado do briefing anterior.

## 2026-09-24, o auditor de transcrição

- **o auditor contava errado um `;` entre aspas:** os dois `--ask` da
  quinta rodada tinham `;` no texto, e o auditor partia a linha ali e
  contava uma palavra da pergunta como programa. o erro só soma (nunca
  esconde um programa), então as contagens de zero das rodadas
  anteriores valem. o partidor agora respeita aspas, e um `--ask` com
  `;` e `|` virou caso de teste dele.

## 2026-09-23
- **briefing que proíbe o que o gate exige:** o briefing da 68 proibiu
  `README.md` e `docs/ATLAS.toml`, e o `arc_sync_guard` exige de todo
  crate novo uma entrada nos dois. o agente parou com o gate vermelho
  num teste só. regra: crate novo sempre leva, nos arquivos permitidos, a
  própria linha no README e a entrada no `arc.toml`, como a seção 8 do
  programa já dizia. o agente achou também que o guard passava por acaso
  para o `uir` no `arc.toml` (a palavra "acquire" contém "uir"): o guard
  procura texto, não entrada.

## 2026-09-23
- **não roubar o foco dá mais trabalho do que parece no macOS:** só
  não ativar o app não segurou o foco quando o app da frente era o que
  lançou o processo (o Zed). o macOS 14 deixa esse app ceder a ativação
  por cooperação. o que segurou foi a política de ativação proibida com
  a janela sempre por baixo (`PLEV_BACKGROUND=1`). a prova em janela real
  com o brenner na máquina usa isso, e a janela sobe sem que ninguém
  possa usá-la.

## 2026-09-23
- **a sonda antes do briefing pagou na primeira vez:** o briefing da
  88 marcou o diagnóstico como hipótese e pediu a sonda primeiro. ela
  mostrou que o gatilho proposto para a pessoa quebraria o teste do gate
  e que a medição proposta dava falso válido. nenhuma linha de correção
  foi escrita em cima da premissa errada.

## 2026-09-23
- **limitação "lida no código, não rodada" não vira cenário de task sem
  sonda:** o handoff da 86 descreveu a limitação do `acted` lendo o
  código, e eu escrevi a task-90 com esse cenário. a sonda do agente, pelo
  clique real, mostrou outro mecanismo (a seleção muda a revisão fora do
  dispatcher) e o `acted` vazio, não nomeando a seleção. terceiro briefing
  do ciclo com premissa errada (83, 86, 90). em todos, o agente conferiu
  antes de implementar, porque o briefing mandava confirmar e parar. regra
  a partir daqui: o cenário que motiva uma task nova é reproduzido por
  teste ou sonda antes do briefing, ou o briefing o marca como hipótese e
  pede a sonda como primeiro passo.

## 2026-09-23
- **prova em janela real na máquina do brenner:** a primeira tentativa
  da prova da task-86 postou um clique sintético que caiu no navegador
  dele, que estava na frente; na reprodução do orquestrador, a janela
  nova do ide tomou o foco e recebeu o que ele fazia. regra: o harness
  confere a janela no ponto antes de cada clique (a 86 passou a fazer);
  com o brenner na máquina, a reprodução do orquestrador usa o teste de
  porta sem janela e deixa a janela para a prova do agente; um processo
  morto pelo pid deixa o socket, que sai com `trash`.

## 2026-09-23
- **premissa do briefing conferida no código, de novo:** o briefing da
  86 dizia que `AgentPort::open` respeita `PLEV_AGENT=1` e debug; quem
  respeita é `from_env`. o agente seguiu o contrato ("release fechado")
  e apontou a divergência. é o segundo briefing do ciclo com premissa
  errada (a primeira foi a da 83); ao citar o comportamento de uma
  função no briefing, ler o doc dela antes.

## 2026-09-23
- **o auditor de transcrição dava falso positivo:** um agente guardou o
  caminho do `plev` numa variável (`P=...; $P observe`) e o auditor, que
  olhava o começo da linha, marcou 12 comandos como proibidos. conferido
  à mão antes de concluir; o script agora resolve a variável. regra:
  número de auditoria que acusa o agente se confere na transcrição antes
  de entrar no resultado.

## 2026-09-23
- **dois workspaces num target dir:** com a perna urnaui rodando nos
  worktrees, o workspace raiz e o do urnaui passaram a gravar no mesmo
  `CARGO_TARGET_DIR`; o engine, com crate-type cdylib e staticlib, grava
  `libengine.rlib` sem hash no nome, e um workspace pegava o arquivo do
  outro ("multiple different versions of crate `sem`"). a classe:
  artefato sem hash compartilhado entre workspaces. o gate agora dá à
  perna urnaui um subdiretório próprio (`1bf6606`). achado pelo agente da
  task-83, que o contornou com `cargo clean -p engine` e reportou em vez
  de mexer no gate.

## 2026-09-23
- **contorno de harness escondeu defeito de produto:** o agente
  roteirizado esperava o index "parar" (até 200 observações), e isso
  escondia a corrida do `Refresh`. regra: um harness ou agente que
  espera o app "assentar" antes de agir é sintoma; a espera vira bug do
  app, registrado, nunca contorno silencioso.
## 2026-09-23
- **a perna urnaui do gate pulava em todo worktree:** o gate procura
  `../urna` a partir do worktree, que não existe. o symlink
  `.claude/worktrees/urna` fez a perna rodar (a primeira vez foi no gate
  da task-84).
## 2026-09-23
- **32 divergências doc x código acumuladas:** os fechamentos escreviam
  o doc novo e não corrigiam os docs que a mudança contradizia (a 58b
  trouxe `decline` e o [ADR commit-checks](../../adr/Semantics/0036-commit-checks-the-index-it-observed/0036-commit-checks-the-index-it-observed.md) continuou dizendo que não
  existia). regra nova do fechamento: procurar em `docs/` e no README os
  nomes do contrato que mudou e corrigir o que ficou falso
  ([`orchestrate-coding-agents-on-this-repo.md`](orchestrate-coding-agents-on-this-repo.md)).

## 2026-09-23
- **o gate local divergia do CI:** a 66b deixou `ShapedGlyph` e
  `shaped_glyphs` como código morto no wasm32 (o único leitor, o export
  svg, é só nativo). o CI (`RUSTFLAGS=-D warnings`) reprovou os jobs wasm e
  urnaui do PR #45; o gate local passou porque a perna wasm rodava sem `-D
  warnings` e nunca checava o urnaui no wasm32. a classe: uma perna do
  gate mais frouxa que a do CI deixa passar o que o CI reprova. o gate
  agora roda as duas pernas wasm como o CI (`c07a7f0`); a falha reproduziu
  localmente antes da correção e sumiu depois. regra: toda perna do CI tem
  a mesma perna, com as mesmas flags, em `tool/tasks/fullcheck.sh`.

## 2026-09-23
- **graça do laboratório:** o laboratório fechava 1 s depois do commit e
  derrubou a verificação final de dois agentes (A2, B2). a classe: um
  harness que encerra no primeiro sinal de sucesso mede o sucesso e apaga a
  verificação. o `--grace` passou a ser parte do protocolo (180 s nas
  corridas refeitas). regra para os próximos experimentos: o harness só
  encerra depois que o agente disse que terminou, ou no timeout.
## 2026-09-23
- **agentes parados:** o brenner viu agentes 16 minutos parados numa task.
  o ciclo autônomo agora volta a cada 10 min com agente rodando e confere
  commits e arquivos alterados; parado, pergunta; parado de novo,
  rebriefing ([`autonomy.md`](autonomy.md), "máquina").
## 2026-09-23
- **disco:** três worktrees compilando em paralelo encheram o disco. um
  target dir por agente, incremental desligado, debug só com line tables;
  no máximo dois ou três agentes compilando ao mesmo tempo com 30 GB
  livres.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [task-70: adaptive transformations](../../Tasks/task-70-adaptive-transformations/task-70-adaptive-transformations.md)
- [task-75: production-ready platform accessibility](../../Tasks/task-75-platform-accessibility/task-75-platform-accessibility.md)
- [task-83: the delta carries the node that was just added, and the refusal names who acted](../../Tasks/task-83-delta-carries-node-refusal-names-actor/task-83-delta-carries-node-refusal-names-actor.md)
- [task-84: an old refresh does not undo what the ide has already accepted](../../Tasks/task-84-ide-refresh-never-goes-back/task-84-ide-refresh-never-goes-back.md)
- [task-86: the ide people use opens the agent port](../../Tasks/task-86-the-ide-opens-the-agent-port/task-86-the-ide-opens-the-agent-port.md)
- [task-90: a refusal names who acted since the agent looked](../../Tasks/task-90-a-refusal-names-who-acted-since-the-look/task-90-a-refusal-names-who-acted-since-the-look.md)
- [operating mode: autonomous orchestrator](autonomy.md)
- [how to orchestrate coding agents on this repo](orchestrate-coding-agents-on-this-repo.md)
- [autonomous cycle journal](autonomous-cycle-journal.md)
<!-- /related -->
