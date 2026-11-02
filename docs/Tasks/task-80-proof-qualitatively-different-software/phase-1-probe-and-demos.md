---
id: task-80/phase-1-probe-and-demos
title: "task-80, phase 1: probe, five-demo method, demos 3 and 4 and handoff"
project: plev
type: section
parent: task-80
status: reference
date: 2026-09-25
date_basis: text
recorded: 2026-09-23
topics: [agents, recording, generated-surfaces, research]
relations: [adr-0050, adr-0070, task-60, task-62, task-63, task-64, task-68, task-69, task-70, task-84, task-86, task-87, task-89, task-92, task-93, task-94, task-96, generative-runtime-thesis]
evidence: [e44e4234, c8a6c3a9, f008c897, 46f272d3, b26dc988, ae5be30e, 677b1ad9]
source: docs/log/steps/task-80-proof-qualitatively-different-software.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# phase 1: probe, five-demo method, demos 3 and 4 and handoff

Phase 1 of task-80 was opened by the orchestrator on 2026-09-25, with tasks 62, 63, 64, 69 and 70 integrated. It gathers the code probe, the five-demo method written before running, what was already proven from the source, demo 3 (a real bug recorded without a window, read at the moment by plev-inspect and turned into a regression test) and demo 4 (the ephemeral interface measured through the recording), in addition to the handoff. Left out are demo 2 with Brenner, the core question, an outside person, and any real call to Claude.

## fase 1 (aberta em 2026-09-25)

aberta pelo orquestrador com as tasks 62, 63, 64, 69 e 70 integradas. o
método das cinco demos escrito antes de rodar, o que já está provado
reunido com a fonte, e as demos 3 (inspectability, teste no gate) e 4
(interface efêmera). ficam de fora: a demo 2 com o brenner, a pergunta do
núcleo, uma pessoa de fora e qualquer chamada ao Claude de verdade. a
demo 5 vira task nova se pedir superfície gerada num app que ainda não
tem.

## fase 1: sonda (2026-09-25, antes de qualquer demo nova)

base: `main` @ `d1b4938d`, branch `task/80a-proof` (o `1927390c` abriu
esta fase). toda afirmação sobre código tem arquivo e linha; toda
afirmação sobre um resultado de antes tem o arquivo que o registrou e a
seção. esta sonda é só leitura: nenhuma sessão nova foi gravada, nenhum
teste novo rodou.

### (a) as cinco demos contra o que existe

| demo | já medido, e onde | o que falta | quem faz o que falta | funcionalidade nova? |
| --- | --- | --- | --- | --- |
| 1, stale-state rejection | o experimento do ide em cinco rodadas, com braços e hipóteses escritos antes: A0 roteirizado, A LLM pelo contrato, B o `git` na linha de comando, C o volume de um screenshot (`checked/task-62-ide-human-agent-proof-done.md`, "os braços", "hipóteses (escritas antes de rodar)", "resultado da fase c" e os resultados da segunda à quinta rodada) | as duas alternativas de controle do escopo que ninguém rodou: um agente por screenshot (o braço C é "comparação de volume, não de capacidade" e "nenhum agente por screenshot foi rodado", mesma task, "as hipóteses", H4) e um agente por snapshot de accessibility de terceiros (programa, seção 10: "se não houver comparação justa, a limitação é registrada") | o orquestrador, com o Claude de verdade | sim, para o braço por screenshot: o `wire` só age por identidade (`Act.target: SemId`, `rust/socket/src/protocol.rs:135-146`), e clique por coordenada de tela é proibido aqui; um braço que receba "clique em x, y" do agente e resolva pelo `Compositor::hit` no frame headless é trabalho novo do laboratório (`apps/agenteval/src/**`). o de accessibility de terceiros pede a janela e a árvore do AccessKit lida por fora (task-60): orquestrador com janela |
| 2, handoff humano-agente | com uma pessoa roteirizada, não com o brenner: a quarta e a quinta rodada (a pessoa tira um arquivo antes do submit e responde ao modal; `task-62...`, "resultado da quarta rodada" e "resultado da quinta rodada"); a pessoa de verdade só como CGEvent sintético numa janela real (`checked/task-86-the-ide-opens-the-agent-port-done.md:148-163`) | o roteiro do escopo com o brenner: o agente prepara, ele assume, muda parte, devolve, o agente termina, e cada receipt diz quem fez | o brenner, com o orquestrador (janela, `PLEV_AGENT=1 PLEV_RECORD=1`, o Claude de verdade) | não: a porta na janela do ide (task-86), a gravação do ide (task-63d), a pergunta à pessoa (task-92) e a pessoa que responde no laboratório (task-93) existem |
| 3, inspectability | a fase a da task-64 achou um defeito real lendo a sessão do A2 da primeira rodada no instante do submit: o `changes:staged` observado "outside the scope observed", o que virou a task-89 (`checked/task-64-session-inspector-and-tests-done.md`, "saídas de aceite" e "fase a integrada"); a fase c transformou a sessão a2ppa em teste de regressão (`apps/idestudio/tests/rerun/eval.rs:165`) e provou à mão, fora do gate, que ele pega uma precondição quebrada de propósito (mesmo arquivo, "o teste gerado e a prova de que pega regressão") | um bug real do histórico, enviado como trecho, aberto no instante, com as perguntas da tese respondidas, e um teste no gate que pega o bug reintroduzido (a fase c provou com uma quebra provocada, e à mão) | este agente | não, com uma fixture nova só em teste (item c) |
| 4, interface efêmera | a demo do editor de vídeo gravada sem janela pela 69 fase 4c (`apps/uigallery/tests/fixtures/sessions/silences-demo.monster`, 186.837 bytes, lida por `apps/uigallery/tests/demo.rs`; `checked/task-69-ephemeral-generative-surfaces-done.md`, "a demo (arquivo, tamanho, o que tem dentro)"); o checkpoint b, duas rodadas, a revisão gerada contra a tela fixa na mesma tarefa (mesma task, "checkpoint b, primeira rodada" e "checkpoint b, segunda rodada") | a gravação com o código de hoje (a revisão das regras mudou na task-70 fase 2), a leitura pelo inspector mostrando que a superfície nunca foi tela, e a comparação com critério escrito antes para o passo da comparação, que o checkpoint b não mediu (ele comparou só a revisão) | este agente | não para o que as regras fazem. "não consigo comparar" pelas regras é recusado na Silences: o único `List` do app são os ids dos cortes e a duração é um `num` por corte (`checked/task-70-adaptive-transformations-done.md`, "(b) a gramática das regras" e "a demo"); a comparação que existe na Silences é a da gramática de superfície, "compare the original with the cuts" (`rust/offers/src/change.rs:5-8`). trocar as durações por gráfico pede o app declarar uma lista de durações (task nova, abaixo) |
| 5, UI generativa em app real | nada: as superfícies geradas vivem só nas páginas Surface e Silences do showcase | uma intenção no ide ("revisa esses três commits") ou no urnaui gerando uma superfície que a pessoa opera | uma task nova | sim: nem o ide nem o urnaui têm `SurfaceHost` (`SurfaceHost` aparece só em `rust/screen/src/host/mod.rs` e em `apps/uigallery/src/views/apps/**`; `apps/idestudio/Cargo.toml` não depende de `surface`, `propose` nem `uir`; `apps/urnadbapp/Cargo.toml` também não). o que já ajuda: o `uir` tem fixtures no domínio do ide, verificadas contra o registro que a projeção do ide dá (`rust/plevir/tests/fixtures/idestudio-review-commit.ir.json`, `ide-commit-form.ir.json`, `ide-workbench.snapshot.json`, escrito por `apps/idestudio/tests/plevir_fixture.rs`) |

a proposta para a demo 5, que não é implementada aqui:

- **task-96, uma superfície gerada no ide.** escopo: um `SurfaceHost` na
  janela do ide ao lado das vistas fixas; a linha de intenção (campo,
  proponente, propor, fechar) projetada como na Silences; o registro de
  capacidades tirado do `Workbench::snapshot`; os atos da superfície
  roteados ao dispatcher único do workbench (`Surface::route`), com os
  receipts e a gravação do ide como estão; as regras propondo "review
  these commits" sobre `stacks` (o `uir` já verifica
  `ide-review-commit.ir.json`); o Claude pelo `propose::Llm` fora da
  thread do app. dependências: 68 (uir), 69 (surface e propose), 70
  (transformação e política), 63d e 64 (gravação e inspector),
  integradas. superfície: `apps/idestudio/src/**` (a janela e a projeção),
  `apps/idestudio/Cargo.toml` (três dependências de workspace, nenhuma
  externa). o que a derruba: a superfície pior que o painel de stacks
  mais o diff para revisar três commits, pelo critério da demo 5 abaixo.
- **o gráfico das durações na Silences** (P3, já anotado pelo programa
  na linha da task-70): o app declarar a lista das durações como estado,
  para "não consigo comparar" trocar a lista por gráfico pelas regras.
  não bloqueia a demo 4.

### (b) gravar sem janela

sim, sem janela e sem GPU, pelos dois caminhos que já existem:

- o showcase: o exemplo `demo` roda a Silences servida pela porta num
  `Compositor` sem janela (`apps/uigallery/examples/videos/main.rs:12-16`,
  `shell.rs:50` `draw`, `:61` `turn`), e grava pelo mesmo gancho do shell,
  `showcase::app::record::Recording::frame`
  (`apps/uigallery/src/shell/window/record.rs:171`), que chama
  `rec::Recorder::frame` (`rust/record/src/recorder.rs:105`): a cena e as
  ligações de cada frame, sem pixel. o teste
  `apps/uigallery/tests/record.rs` faz o mesmo no gate, com a página
  semeada (nenhum ffmpeg) e uma porta num diretório temporário.
- o ide: o laboratório (`apps/agenteval/src/arms/plev.rs:286` `record`)
  grava estado e frame a cada volta em que algo mudou, com o renderer
  headless (`apps/agenteval/src/subject.rs:124`), sem janela.

então a demo 4 grava sem o orquestrador. o que muda na Silences sem o
ffmpeg: a detecção real e os frames do vídeo. um teste parte da semente
(os cortes 1 e 2) e põe frames sintéticos no cache pelo gancho público
`ShowcaseView::stage_silences_frames`
(`apps/uigallery/src/views/apps/silences/generative/page.rs:530`), como o
`regress` faz (`tool/bench/regress/src/state.rs:314`, `:431`).
nenhuma janela: nada para o orquestrador rodar com `PLEV_BACKGROUND=1`.

### (c) a demo 3: qual bug, como reproduzir, o inspector e o teste

as perguntas da tese ([`generative-runtime-thesis.md:885-901`](../../Research/Synthesis/generative-runtime-thesis.md), seção 12):
qual controle era esse, qual era seu estado, qual foi a sequência
anterior, qual ação ocorreu, quem iniciou, qual era a precondição, o que
mudou, qual foi o resultado. e a seção "sessão vira teste" (`:903`).

o caminho que transforma sessão em teste é o harness da task-64 fase c:
cada ato do trecho refeito na ordem gravada contra uma `Fixture` do app
(`apps/idestudio/tests/rerun/harness.rs:38-65`), e o veredito, o estágio
terminal e as marcas comparados com a gravação. a fixture do ide chama
`Workbench::dispatch` direto (`apps/idestudio/tests/rerun/eval.rs:102-104`), e
antes de cada ato o harness assenta o app (`harness.rs:329-336`), menos
onde a gravação tinha uma execução em voo (`harness.rs:242-246`); o
`settle` da fixture pede um refresh e aplica o worker até todo refresh
devido responder (`lab.rs:74-96`, `:117-121`).

os três candidatos:

- **o porto que não acordava (task-92).** a correção é do `wire` (o
  sink do porto acorda o loop, `rust/socket/src/port.rs:166-172` e
  `rust/socket/src/sink.rs:33`, `Waits`), e o harness nunca passa pela porta: o rerun despacha no
  workbench. um trecho não reproduz a espera; nem a gravação mostra
  quando um estágio chegou ao agente. não serve sem um harness novo, que
  passe pela porta.
- **o `changes:staged` apoiado no que o agente nunca viu (task-89),
  achado pelo próprio inspector.** também é do `wire` (`shown.rs`), e o
  rerun despacha o request com os valores já gravados em `observed`: a
  regra do porto não roda. não serve pelo mesmo motivo.
- **a corrida do `Refresh` do ide (task-84), a escolhida.** um bug real,
  achado pelo agente roteirizado no checkpoint a
  (`checked/task-62-...`, "o que o experimento mostrou além das
  hipóteses"), com a correção conhecida: `6500f2dc` ("task-84: a read
  taken before a mutation the view shows is dropped"), hoje em
  `apps/idestudio/src/repository/git.rs:262-269` (a leitura velha descartada em
  `apply`) sobre `Owed` e `Reads` (`:42-127`). ele mora no workbench, o
  mesmo caminho que o rerun roda, e aparece no estado projetado, que a
  gravação guarda e o inspector lê: um arquivo que o ide aceitou no index
  sai do `changes:staged` sem request nenhum. e o teste escrito à mão da
  época existe para comparar (`apps/idestudio/tests/human_agent.rs:491`,
  `:517`, `:553`, `:578`).

o que o harness de hoje não alcança, e como alcançar sem mudar interface:
o `settle` de hoje drena todo refresh devido antes de cada ato, então
nunca há um status em voo quando o próximo ato sai, e a corrida some. o
rerun do a2ppa, que é o mesmo cenário, passa com o bug de volta por
construção. a saída cabe numa fixture nova, só em teste, sobre o mesmo
`Fixture`: assentar só as execuções (aplicar as respostas do worker, uma
por vez e na ordem, até nenhum request despachado estar `executing`) e
deixar em voo as leituras que o app pediu, como uma janela viva tem
quando o próximo ato chega. o worker responde em fila
(`rust/gitdrv/src/client.rs:125-130`, pela task-84), então a ordem
aplicada é determinística. o harness, o `Fixture` e as marcas não mudam;
na fixture do ide entra um método que aplica uma resposta por vez com a
mesma conta de refreshes devidos.

como reproduzir numa sessão gravada: o laboratório (`ide-lab --arm
scripted`, sem janela e sem modelo) sobre um build com o bug de volta. o
roteirizado encadeia um ato no estágio terminal do anterior (task-87),
então o próximo `stage` sai logo depois de a resposta do anterior pedir
o refresh, dentro da janela da corrida. o trecho exportado por
`plev-inspect --export` é o bug report; `--timeline`, `--control changes
--at <t>` e `--why` respondem no instante.

um limite do harness que isso expõe: ele compara o veredito com o
gravado, então um trecho gravado com o bug só vira teste se o bug não
mudou veredito nenhum (uma corrida de controle, sem pessoa, costuma ser
assim: task-84, "handoff", dez de dez com exit 0 no cenário `none`). se
mudou, o teste nasce do mesmo cenário gravado na `main`.

### (d) a prova de que o teste pega a quebra

- a branch descartável: `task/80a-bug84`, a partir de `task/80a-proof`,
  nunca no merge. o que volta: o descarte da leitura velha em
  `GitSession::apply` (`apps/idestudio/src/repository/git.rs:266-269`, a essência do
  `6500f2dc`): a resposta é contada e aplicada mesmo quando é mais velha
  que a view.
- o teste novo roda nela (falha esperada, com a saída colada) e em
  `task/80a-proof`, que é a `main` com os commits desta fase (passa).
  o teste do a2ppa com o `settle` de sempre roda nas duas, como controle.
- antes do handoff, o worktree volta para `task/80a-proof`, limpo.

## fase 1: o método das cinco demos (escrito antes de rodar)

escrito e commitado antes de qualquer demo nova rodar: o sha deste commit
é a prova. vale para as cinco o critério de parada do programa (seção 9 e
"critérios de parada" acima): se a rejeição e os receipts não mudarem
como a delegação é feita, ou se as superfícies geradas forem piores que
as fixas nas mesmas tarefas, a direção é reavaliada com os dados. o
resultado de cada demo vai como sair, inclusive negativo.

### demo 1, stale-state rejection

- **roteiro.** o do laboratório da task-62 (repositório com seis
  alterações, a tarefa "commit only the login fix", a pessoa tirando
  `src/session.rs` do index depois de uma observação ou logo antes do
  submit), com os dois braços de controle que faltam: (i) um agente LLM
  por screenshot, que recebe só o PNG do frame do ide e age por "clique
  em x, y", resolvido pelo `Compositor::hit` do frame headless; (ii) um
  agente LLM pelo snapshot de accessibility de terceiros, a árvore do
  AccessKit da janela lida por fora, como o Playwright MCP lê um
  navegador.
- **alternativa de controle.** o `git` na linha de comando (braço B, já
  rodado), o volume de um screenshot (braço C, já medido) e os dois
  braços acima.
- **critério.** o das hipóteses H1 a H5 da task-62, braço a braço lado a
  lado: o commit é exatamente o da tarefa; um commit errado nunca existe
  (a recusa vem antes do efeito, com o delta nomeando o arquivo); o
  agente se recupera; o receipt bate com o git; bytes e tokens enviados
  ao agente, turnos, tempo até o commit, custo da gravação. o plev pode
  perder em bytes e idas e voltas, e isso é escrito como está.
- **o que a derruba.** um commit errado no braço do plev; uma recusa sem
  o delta; um agente que não se recupera; receipt diferente do git; ou um
  braço de controle com a mesma segurança (nada commitado sobre um index
  que mudou, sem ninguém avisar) por menos bytes e tokens: aí a diferença
  é de contrato, não de modelo.

### demo 2, handoff humano-agente

- **roteiro.** a janela do ide com `PLEV_AGENT=1 PLEV_RECORD=1` num
  repositório de rascunho; um agente Claude de verdade prepara o commit
  pelo `plev` (stage, mensagem); o brenner assume na janela e muda parte
  (tira um arquivo ou reescreve a mensagem); devolve dizendo ao agente
  que siga; o agente termina. a sessão é salva e aberta pelo
  `plev-inspect`.
- **alternativa de controle.** o mesmo handoff com o agente e o brenner
  no mesmo repositório pelo `git` e um terminal, sem receipt nem sessão.
- **critério.** cada passo com o ator certo no receipt (passos certos
  sobre o total); nenhum ato do agente sobre o que o brenner mudou sem
  uma recusa que o nomeie ou uma pergunta a ele; o commit final é o que
  ele decidiu; depois, o que ele reconstrói de quem fez o quê pela sessão
  contra o que reconstrói pelo `git` (reflog, log), com as palavras dele.
- **o que a derruba.** um passo sem ator ou com o ator errado; o agente
  passando por cima do brenner sem recusa nem pergunta; o brenner não
  conseguir dizer pela sessão quem fez o quê.

### demo 3, inspectability (feita nesta fase)

- **roteiro.** (1) na branch descartável `task/80a-bug84`, com a corrida
  da task-84 de volta, `ide-lab --arm scripted --human none` grava uma
  sessão sem janela e sem modelo (se a corrida não aparecer no estado
  gravado, a mesma com `--human unstage-after-observe:src/session.rs`);
  (2) o instante em que um arquivo que o ide aceitou sai do
  `changes:staged` é achado na sessão, e o trecho em volta é exportado
  com `plev-inspect --export`: o bug report; (3) o trecho é aberto pelo
  `plev-inspect` (resumo, `--timeline`, `--control changes --at
  <instante>`, `--why` do ato mais perto) e as oito perguntas da tese são
  respondidas só pelo arquivo; (4) o trecho vira teste em
  `apps/idestudio/tests/rerun/`, com marcas e a fixture que assenta só as
  execuções (sonda, item c); se o bug mudou algum veredito gravado, o
  teste nasce do mesmo cenário gravado em `task/80a-proof`; (5) o teste
  roda em `task/80a-bug84` e em `task/80a-proof`, o do a2ppa com o
  `settle` de sempre roda nas duas como controle, e o novo roda dez vezes
  seguidas em `task/80a-proof`.
- **alternativa de controle.** (i) o bug report como mídia: só os frames
  do mesmo trecho (a trilha visual, sem a semântica), o que um vídeo
  mostra; (ii) o rerun do harness como está (assenta tudo antes de cada
  ato); (iii) o teste escrito à mão da task-84
  (`apps/idestudio/tests/human_agent.rs:491-611`).
- **critério.** K1: das oito perguntas, quantas o `plev-inspect` responde
  pelo trecho sozinho, com a resposta conferida contra o código (o
  defeito em `apps/idestudio/src/repository/git.rs`), contra quantas os frames
  sozinhos respondem. K2: o teste do trecho falha em `task/80a-bug84` e
  passa em `task/80a-proof`, dez de dez sem oscilar; o controle (ii)
  passa nas duas. K3: o trecho abaixo de 1 MB, dentro do git ao lado do
  teste. K4: o que foi escrito à mão para ter o teste (linhas de marcas e
  de fixture) contra o teste da task-84.
- **o que a derruba.** o inspector nomeia um ator errado ou não localiza
  a mudança no tempo; a corrida não aparece em estado gravado nenhum (a
  sessão não serve de bug report, e isso é escrito); o teste passa com o
  bug; o teste oscila na `task/80a-proof`; o trecho passa de 1 MB.

### demo 4, interface efêmera (feita nesta fase)

- **roteiro.** um teste em `apps/uigallery/tests/` grava, sem janela, a
  Silences em 1200 por 800 (a janela do checkpoint b), servida por uma
  porta de verdade, a partir da semente (os cortes 1 e 2, sem ffmpeg) e
  com frames sintéticos no cache: um agente escolhe as regras, escreve
  "the list with the video and the counts" (a revisão como as regras a
  leem, task-69 fase 4c) e propõe; a revisão nasce ao lado da tela fixa;
  uma pessoa decide um corte com um clique na superfície gerada (input
  gravado); o agente escreve "I can't compare these" e propõe (pelas
  regras, a sonda espera a recusa na Silences); escreve "compare the
  original with the cuts" e propõe (a revisão vira a comparação no
  lugar, o mesmo id); fecha. a sessão é salva fora do git e lida pelo
  `plev-inspect`.
- **alternativa de controle.** a tela fixa da Silences, na mesma página e
  no mesmo instante, ao lado da gerada, para as mesmas duas tarefas:
  revisar os cortes (decidir cada um) e comparar o original com o
  resultado.
- **critério**, medido na gravação, não no olho:
  - K1, a tarefa chega ao app: a decisão tomada na superfície é um
    request sobre o corte do app (`silences.cut[n]`), pela pessoa, e a
    tela fixa mostra a mesma decisão depois (um estado só);
  - K2, à vista sem rolar em 1200 por 800: por corte, os controles da
    revisão (o frame do corte, a faixa, a decisão, aplicar, rejeitar)
    com a ligação dentro do frame, na gerada e na fixa, no mesmo
    instante, pelas ligações gravadas. a gerada não é pior se tem pelo
    menos tantos quanto a fixa;
  - K3, a comparação: depois da intenção de comparar, quantos frames do
    vídeo estão à vista para pôr o original ao lado do resultado, na
    gerada e na fixa;
  - K4, efêmera: os nós da superfície existem no estado gravado só entre
    o propose e o close; depois do close nenhum nó e nenhuma ligação
    dela; o `source` de todo nó dela é o materializador
    (`rust/screen/src/**`), nunca um arquivo da tela do app, enquanto
    os nós da fixa apontam para `apps/uigallery/src/views/apps/silences/**`;
  - K5, custo: tempo do propose à superfície pelas regras, nós, bytes e
    frames da gravação;
  - K6, a recusa: "I can't compare these" deixa a superfície como estava
    (os mesmos nós, as mesmas decisões) e a linha diz por quê.
- **o que a derruba.** a gerada pior que a fixa em K2 ou K3; qualquer
  coisa da superfície depois do close (nó, ligação, estado); uma decisão
  na superfície que não chega ao dispatcher do app; a gravação sem a vida
  da superfície; um vazamento de privacidade (as promessas do briefing:
  segredo só como tamanho, input só como tipo e `SemId`, nenhuma câmera,
  a preferência de accessibility fora da gravação).

### demo 5, UI generativa em aplicação real (task-96)

- **roteiro.** no ide, com a task-96: a pessoa escreve "review these
  three commits" na linha de intenção; as regras (e o Claude, pelo
  orquestrador) propõem; a superfície mostra os três commits com os
  arquivos e o diff de cada um; a pessoa abre cada diff pela superfície e
  fecha. gravado, sem janela pelo laboratório e com janela pelo brenner.
- **alternativa de controle.** o painel de stacks fixo mais o diff, para
  os mesmos três commits.
- **critério.** atos e tempo para ver os arquivos e o diff dos três; o que
  fica à vista sem rolar; cada ato da superfície no dispatcher do
  workbench, com o receipt da pessoa; nada da superfície depois de
  fechar.
- **o que a derruba.** a superfície pior que o painel fixo nessa tarefa;
  um ato fora do dispatcher; a superfície deixando rastro.

### a pergunta do núcleo e a pessoa de fora

com o brenner, ao abrir o primeiro `.monster` de uma delegação real (a
demo 2): o que ele consegue descobrir, e se isso basta para confiar nela,
com as palavras dele. uma pessoa de fora usa as demos 2 e 4 quando
possível, e o relato dela entra no resultado. nenhuma das duas é deste
agente.

## fase 1: o que já estava provado, com a fonte

nada aqui foi rodado de novo; cada linha vem do arquivo que registrou o
resultado.

### demo 1, stale-state rejection: provada pela task-62, com dois controles faltando

fonte: `checked/task-62-ide-human-agent-proof-done.md`, seções
"resultado da fase c" e os resultados da segunda à quinta rodada.

- **primeira rodada (checkpoint a, `main` @ `d26c9b1`).** H1 a H5
  sustentadas. no A2 (LLM pelo contrato, com a pessoa) o `submit` apoiado
  na revisão de antes da pessoa voltou `stale` com o delta exato
  (`changes:staged` perdeu `src/session.rs`), o HEAD não mudou, o agente
  observou, repôs e commitou os três arquivos, receipt igual ao git. no
  `git` (B2, B3) a intervenção equivalente, um hook no commit, produziu
  um commit de dois arquivos sem aviso; o B3 só consertou investigando
  depois e reescrevendo com `--amend`. em volume o `git` ganhou: 1.459 B
  em 20 invocações (B3) contra 22.040 B em 30 mensagens (A2). uma
  observação inteira é 9,9 vezes menor que o PNG do frame a escala 1
  (6.359 B contra 62.939 B), como volume, não como capacidade.
- **segunda rodada (depois das tasks 83, 84 e 85).** R1 a R3 sustentadas
  (o `acted` nomeou a pessoa e os dois agentes citaram isso); R4 mista:
  o controle 11% mais barato em bytes, as corridas com a pessoa 14 a 16%
  mais caras.
- **terceira rodada (a regra da task-89).** a segurança se sustentou nas
  quatro: nenhum commit errado. um agente (A2''b) parou sem commit por
  não ter como perguntar à pessoa, o que virou a task-92.
- **quarta e quinta rodada (o handoff).** a segurança se sustentou nas
  oito. sem dica ninguém perguntou; com a frase no briefing, ou com a
  linha do stderr da task-94, os quatro que puderam perguntar perguntaram,
  e a resposta da pessoa decidiu o commit.
- **o custo da gravação**, primeira rodada, debug: 138 us por
  `rec.snapshot`, 289 us por `rec.frame`; a sessão do A2 com 181.586 B,
  67 eventos e 15 frames ("os números").

o que ainda não cobre: um agente por screenshot (o braço C mediu só
volume) e um agente por snapshot de accessibility de terceiros; todas as
corridas com um modelo só (`claude-fable-5-1`) e duas a quatro corridas
por braço ("duas corridas por braço não medem taxa", quinta rodada); os
custos em build de debug; a pessoa sempre roteirizada.

### demo 2, handoff: provado com uma pessoa roteirizada, não com o brenner

fonte: a mesma task-62, quarta e quinta rodada; `checked/task-92-the-agent-asks-the-person-done.md`
e `checked/task-93-the-lab-person-answers-done.md`.

- cada recusa nomeou a pessoa no `acted` (request 6, `unstage` de
  `src/session.rs`) nas seis corridas da quarta rodada e nas duas da
  quinta; o `--ask` chegou à pessoa pelo modal e a resposta dela decidiu
  nas quatro corridas em que houve pergunta (A4c1, A4c2, A5a, A5b);
  nenhum ato depois de um `declined`.

o que ainda não cobre: o roteiro do escopo (o brenner assume, muda parte,
devolve) numa janela de verdade; a pessoa de fora; a pergunta do núcleo.

### demo 3, inspectability: provada em parte pela task-64

fonte: `checked/task-64-session-inspector-and-tests-done.md`, "saídas de
aceite", "fase a integrada", "o teste gerado e a prova de que pega
regressão" e "fechamento da fase c".

- o `plev-inspect` respondeu, numa linha rotulada por pergunta, sobre as
  nove sessões dos checkpoints; na do A2 da primeira rodada mostrou que o
  submit se apoiou num `changes:staged` "outside the scope observed", um
  valor que a conexão nunca viu. isso foi um defeito real achado lendo a
  sessão, e virou a task-89.
- a sessão a2ppa virou teste no gate
  (`apps/idestudio/tests/rerun/eval.rs:165`), e com a precondição do `submit`
  quebrada de propósito o teste falhou dizendo onde (à mão, repetido pelo
  orquestrador; no gate quem guarda o caso é o brinquedo).

o que ainda não cobre: um bug do histórico, com correção conhecida,
enviado como trecho e virando teste no gate que o pega quando volta; uma
corrida entre o worker e um ato, que o `settle` de hoje esconde (sonda,
item c).

### demo 4, interface efêmera: provada pela task-69 e pelo checkpoint b, no que eles mediram

fonte: `checked/task-69-ephemeral-generative-surfaces-done.md`, "fase 4c"
("a demo", "o que o plev-inspect disse"), "checkpoint b, primeira rodada"
e "checkpoint b, segunda rodada"; `checked/task-70-adaptive-transformations-done.md`,
"a demo" da fase 2.

- a demo do editor de vídeo foi gravada sem janela, pelas regras e pelo
  Claude (`apps/uigallery/tests/fixtures/sessions/silences-demo.monster`, 186.837
  B; `silences-demo-claude.monster`, 861.354 B), e
  `apps/uigallery/tests/demo.rs` confere no gate que a superfície está no
  estado e nas ligações no instante do apply e sumiu no fim.
- checkpoint b, primeira rodada: a gerada saiu pior (a linha do corte
  quebrada pelas regras; pelo Claude a tarefa por último, sliders sem
  rótulo, números crus). segunda rodada, com as correções: a gerada
  deixou de ser pior. a do Claude põe os dois cortes e as ações à vista
  sem rolar em 1200 por 800; a fixa mostra mais por corte (o frame e a
  duração). o estado é um só: decidir pela fixa e pela gerada deixou as
  duas iguais.
- task-70 fase 2: "too much information" enxuga a revisão aberta pelas
  regras; "I can't compare these" é recusado na Silences com o motivo.

o que ainda não cobre: o passo da comparação contra a tela fixa (o
checkpoint b comparou só a revisão); uma gravação com a revisão das regras
depois da task-70; uma medida pela gravação em vez do olho; a leitura pelo
inspector mostrando que a superfície nunca foi tela.

### demo 5, UI generativa em app real: nada provado

nenhuma superfície gerada num app que já existe (sonda, item a). a demo
espera a task-96.

## fase 1: demo 3, inspectability (feita)

o bug: a corrida do `Refresh` da task-84. a correção é o `6500f2dc`; na
branch descartável `task/80a-bug84` (nunca no merge) o commit `84161012`
põe a corrida de volta: em `GitSession::apply`
(`apps/idestudio/src/repository/git.rs:266-269`) a resposta continua contada, mas é
aplicada mesmo quando foi lida antes de uma mutação que a view já mostra.

### a sessão

gravada sem janela e sem modelo, no build da branch descartável:

```text
ide-lab --arm scripted --human none --out <scratch>/bug84-none --app ide80 --timeout 60
```

exit 0, sessão x git MATCH, os três arquivos commitados. mas com ninguém
além do agente, o primeiro submit (request 6) voltou `stale`: "changes:
added [tests/login_test.rs]", `acted` vazio. a sessão tem 81.862 bytes,
47 eventos, 10 frames (sha256 `bedd9e73...`, igual antes e depois de
cada leitura e export).

dois trechos exportados por `plev-inspect --export`:

- o bug report, `--export 0 120`: 71.580 bytes, 37 eventos, 9 frames, do
  começo até a recusa. fica fora do git (na scratchpad da sessão), é o que
  se manda.
- o trecho do teste, `--export 0 92`: 43.916 bytes, 25 eventos, 7
  frames, do começo até o form aberto, antes do submit. entra no git em
  `apps/idestudio/tests/fixtures/sessions/bug84.monster` (abaixo de 1 MB).

### a leitura no inspector, no instante

o resumo e o trecho da timeline que importa (`plev-inspect <sessão>`, e
`--timeline`):

```text
session   rec 0.1.0, built from 84161012
span      +0.000s to +0.173s, 47 events, 10 frames
actors    agent:scripted: 8 observations, 7 requests, 28 stages
requests  7
  +0.018s  request 1  agent:scripted  unstage on changes.file[README.md]          completed
  +0.036s  request 2  agent:scripted  stage on changes.file[src/session.rs]       completed
  +0.065s  request 3  agent:scripted  stage on changes.file[tests/login_test.rs]  completed
  +0.091s  request 4  agent:scripted  open on commit.form                         completed
  +0.093s  request 5  agent:scripted  set_value on commit.message                 completed
  +0.096s  request 6  agent:scripted  submit on commit.submit                     rejected: stale
  +0.128s  request 7  agent:scripted  submit on commit.submit                     completed: commit = "936ede1c...", files = [src/login.rs, src/session.rs, tests/login_test.rs]
refusals  1
  request 6, submit on commit.submit by agent:scripted, rejected at +0.113s: stale
    changes:staged: added tests/login_test.rs

+0.036s     stage     request 2 executing (agent:scripted, stage on changes.file[src/session.rs])
+0.036s     changes   changes:staged: added src/session.rs; changes.file[src/session.rs]:staged: false -> true; ...
+0.048s     changes   changes:staged: removed src/session.rs; changes.file[src/session.rs]:staged: true -> false; ...
+0.064s     stage     request 2 completed (agent:scripted, stage on changes.file[src/session.rs])
+0.065s     stage     request 3 executing (agent:scripted, stage on changes.file[tests/login_test.rs])
+0.065s     changes   changes:staged: added tests/login_test.rs; changes.file[tests/login_test.rs]:staged: false -> true; ...
+0.077s     changes   changes:staged: added src/session.rs; removed tests/login_test.rs; ...
+0.089s     stage     request 3 completed (agent:scripted, stage on changes.file[tests/login_test.rs])
+0.090s     observed  agent:scripted observed the whole app, revision 0267c905f7e90383
...
+0.096s     stage     request 6 accepted (agent:scripted, submit on commit.submit)
+0.096s     stage     request 6 executing (agent:scripted, submit on commit.submit)
+0.101s     snapshot  keyframe, 14 nodes: ... changes:staged: added tests/login_test.rs; changes.file[tests/login_test.rs]:status: "untracked" -> "added"; ...
+0.113s     stage     request 6 rejected: stale (agent:scripted, submit on commit.submit)
```

no bug report, os dois instantes (`--control`, cortado):

```text
$ plev-inspect bug84-report.monster --control 'changes.file[src/session.rs]' --at 50
control changes.file[src/session.rs] at +0.050s
  node          changes.file[src/session.rs]: list_item "src/session.rs", enabled
  state         status = "modified", staged = false, unstaged = true
  source        apps/idestudio/src/capability/project.rs:88:26, in build 84161012
  requests      1 touched it up to +0.050s (their target, or a state they read)
  request 2 (origin 0): stage on changes.file[src/session.rs] by agent:scripted, at +0.036s
    observed      not recorded at revision 6bb666e6c0b67ed8: ... the revision it acted on came another way, such as the last stage of an earlier act
                  the recorded state was at revision 6bb666e6c0b67ed8 last at +0.018s
    stages        +0.036s requested / accepted / executing
    result        not finished: still executing (since +0.036s)

$ plev-inspect bug84-report.monster --control 'changes.file[tests/login_test.rs]' --at 90
control changes.file[tests/login_test.rs] at +0.090s
  state         status = "untracked", staged = false, unstaged = true
  request 3 (origin 0): stage on changes.file[tests/login_test.rs] by agent:scripted, at +0.065s
    observed      not recorded at revision 6bb666e6c0b67ed8: ...
                  the recorded state was at revision 6bb666e6c0b67ed8 last at +0.048s
    stages        +0.065s requested, accepted, executing; +0.089s completed
    result        completed at +0.089s
```

e o submit que a corrida enganou (`--why 6`, inteiro menos a nota):

```text
request 6 (origin 0): submit on commit.submit by agent:scripted, at +0.096s
  control       commit.submit: button "Commit", enabled
  state         none of its own
  initiated by  agent:scripted
  action        submit, no parameter
  observed      +0.095s, the whole app, at revision 3ce76fd8e2065683
  precondition  what the action reads, as agent:scripted saw it:
                  changes:staged = {src/login.rs, src/session.rs}
                  commit.message:text = "fix login session timeout"
  before        from the observation (+0.095s) to the request (+0.096s):
                  changes:staged did not move
                  commit.message:text did not move
  stages        +0.096s requested
                +0.096s accepted
                +0.096s executing
                +0.113s rejected: stale
  verdict       accepted at +0.096s
                refused at the point of truth, after it started: stale
  what changed  changes:staged: added tests/login_test.rs
  acted         the refusal names nobody
  result        nothing ran: the point of truth refused it before the effect
```

as oito perguntas da tese ([`generative-runtime-thesis.md:893-900`](../../Research/Synthesis/generative-runtime-thesis.md)),
respondidas só pelo arquivo:

| pergunta | resposta do inspector |
| --- | --- |
| qual controle era esse | `changes` e as linhas `changes.file[src/session.rs]` e `changes.file[tests/login_test.rs]`, projetadas em `apps/idestudio/src/capability/project.rs:74` e `:88` |
| qual era seu estado | a +0.050 s, `src/session.rs` com `staged = false` enquanto o próprio stage executava; a +0.090 s, `tests/login_test.rs` com `staged = false`, 1 ms depois de o stage dele concluir |
| qual foi a sequência anterior | stage de `src/session.rs` executando a +0.036 s, o arquivo sai do `changes:staged` a +0.048 s, o stage conclui a +0.064 s; o mesmo com `tests/login_test.rs` a +0.065, +0.077 e +0.089 s |
| qual ação ocorreu | `stage` (requests 2 e 3), depois o `submit` (request 6) |
| quem iniciou | o agente, em todos os requests; as duas saídas do arquivo não têm request nenhum (a timeline não tem estágio de ninguém a +0.048 e +0.077 s), e o único ator da sessão é `agent:scripted` |
| qual era a precondição | o `submit` leu `changes:staged = {src/login.rs, src/session.rs}`, como o agente viu a +0.095 s: dois arquivos, depois de pôr três |
| o que mudou | no ponto de verdade, `changes:staged: added tests/login_test.rs`: o index do git tinha o arquivo o tempo todo |
| qual foi o resultado | nada rodou: o `CommitIf` recusou antes do efeito; o segundo submit commitou os três |

o que leva ao código: uma mudança do `changes:staged` sem request, com um
stage do mesmo arquivo em execução, e a revisão que o `completed` do
request 2 carregou (`6bb666e6...`, em que o request 3 se apoiou) é a
mesma que o app tinha antes do request 2 (a +0.018 s) e de novo a +0.048
s: a view voltou a um estado que já tinha deixado. o único lugar que
troca a lista inteira pelo que o git leu é `GitSession::apply`
(`apps/idestudio/src/repository/git.rs:271-275`, `set_files`), e a resposta que
chegou ali foi lida antes do stage.

### o controle (i): os frames sozinhos

a trilha visual guarda a cena e as ligações de cada frame, nada de
estágio, ator ou request (`rust/record/src/visuals/mod.rs:1-23`). pelos frames,
um vídeo mostraria a linha do arquivo mudando de grupo e voltando:
responde, no máximo, qual controle, o estado desenhado e que algo mudou
(3 das 8). não diz que um stage estava em execução, que ninguém agiu,
qual era a precondição do submit nem que o ponto de verdade recusou. e os
dois lapsos duraram 29 ms (+0.048 a +0.077 s) e 24 ms (+0.077 a +0.101
s), menos que um quadro de um vídeo a 30 fps (33 ms): uma gravação de
tela pode não ter nem o quadro.

### o teste

`apps/idestudio/tests/rerun/inflight.rs` (novo), no binário `rerun` do ide
(`cargo test -p idestudio --test rerun`):

- `InFlight`: a fixture do ide (`Lab`) assentada só até as execuções. o
  `settle` aplica as respostas do worker uma por vez, na ordem, enquanto
  algum request despachado no rerun está `executing`; os refreshes
  pedidos no caminho ficam em voo. `Lab` ganhou `answer_while` (e o
  `drain` passou a usar o mesmo `take`), e `fixture` ficou público no
  módulo; nada no `harness`, no `Fixture`, nas marcas nem no relatório
  mudou.
- `the_bug84_excerpt_keeps_every_staged_file_with_the_reads_in_flight`:
  o trecho `bug84.monster` com as marcas de `bug84.marks` (seis: o
  staged de partida e o que a view tem de guardar depois de cada stage,
  o form aberto no fim). as marcas dizem o que a view precisa ter, não o
  que a gravação teve; todo veredito gravado estava certo.

em `task/80a-proof` (`46f272d3`), verde, e dez de dez seguidas:

```text
rerun of an excerpt recorded by rec 0.1.0, built from 84161012: 4 requests, 6 marks
  same      request 1 (origin 0, rerun as 1): unstage on changes.file[README.md] by agent:scripted: completed
  same      request 2 (origin 0, rerun as 2): stage on changes.file[src/session.rs] by agent:scripted: completed
  same      request 3 (origin 0, rerun as 3): stage on changes.file[tests/login_test.rs] by agent:scripted: completed
  same      request 4 (origin 0, rerun as 4): open on commit.form by agent:scripted: completed
  held      line 18: at 0  changes:staged = {"README.md", "src/login.rs"}
  held      line 22: after request 1  changes:staged = {"src/login.rs"}
  held      line 23: after request 2  changes:staged = {"src/login.rs", "src/session.rs"}
  held      line 24: after request 3  changes:staged = {"src/login.rs", "src/session.rs", "tests/login_test.rs"}
  held      line 25: after request 3  changes.file[tests/login_test.rs]:staged = true
  held      line 26: after request 4  commit.form:open = true
nothing diverged
test inflight::the_bug84_excerpt_keeps_every_staged_file_with_the_reads_in_flight ... ok
```

### a prova de que ele pega a quebra

na branch descartável `task/80a-bug84` (`84161012`, a corrida de volta,
mais o commit do teste escolhido por cherry-pick como `034b951d`):

```text
thread 'inflight::the_bug84_excerpt_keeps_every_staged_file_with_the_reads_in_flight' panicked at apps/idestudio/tests/rerun/inflight.rs:103:5:
rerun of an excerpt recorded by rec 0.1.0, built from 84161012: 4 requests, 6 marks
  same      request 1 (origin 0, rerun as 1): unstage on changes.file[README.md] by agent:scripted: completed
  same      request 2 (origin 0, rerun as 2): stage on changes.file[src/session.rs] by agent:scripted: completed
  same      request 3 (origin 0, rerun as 3): stage on changes.file[tests/login_test.rs] by agent:scripted: completed
  same      request 4 (origin 0, rerun as 4): open on commit.form by agent:scripted: completed
  held      line 18: at 0  changes:staged = {"README.md", "src/login.rs"}
  held      line 22: after request 1  changes:staged = {"src/login.rs"}
  DIVERGED  line 23: after request 2  changes:staged = {"src/login.rs", "src/session.rs"}
            found     {"src/login.rs"}
  DIVERGED  line 24: after request 3  changes:staged = {"src/login.rs", "src/session.rs", "tests/login_test.rs"}
            found     {"src/login.rs", "src/session.rs"}
  DIVERGED  line 25: after request 3  changes.file[tests/login_test.rs]:staged = true
            found     false
  held      line 26: after request 4  commit.form:open = true
the rerun diverged: 0 request(s), 3 mark(s)
test inflight::the_bug84_excerpt_keeps_every_staged_file_with_the_reads_in_flight ... FAILED
test result: FAILED. 9 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s
```

os controles, na mesma branch com a corrida:

- (ii) o rerun em repouso não vê a corrida: o do a2ppa passa
  ("nothing diverged", os nove requests `same`), e o mesmo trecho
  `bug84` com a fixture de sempre também passou, as seis marcas `held`
  (um teste descartável, não commitado).
- (iii) os quatro testes à mão da task-84 falham como falhavam na base
  dela (`a_status_read_before_a_stage_never_takes_the_file_back_out`:
  `left: Set({"A"})`, `right: Set({"A", "B"})`, e os outros três).

### privacidade da gravação

a sessão do laboratório não tem input de pessoa (`--human none`: tipos
`observed` 3, `snapshot` 3, `changes` 7, `stage` 24 no bug report; um
ator só, `agent:scripted`, com `inputs: 0`), o ide não projeta campo
secreto, não há câmera, e a preferência de accessibility não é estado do
ide (nenhum nó ou chave com "secret", "redact", "camera", "contrast",
"text_scale" ou "density" no `--timeline --json` do bug report).

### resultado contra o critério

- **K1:** as oito perguntas respondidas pelo trecho sozinho, conferidas
  contra o código (`apps/idestudio/src/repository/git.rs:266-275`); os frames
  sozinhos respondem no máximo três, e os lapsos são mais curtos que um
  quadro de vídeo a 30 fps.
- **K2:** o teste falha na branch com a corrida (3 marcas divergem) e
  passa em `task/80a-proof`, dez de dez; o controle em repouso passa nas
  duas.
- **K3:** 43.916 bytes no git; o bug report inteiro, 71.580.
- **K4:** para ter o teste: o trecho exportado por um comando, 26 linhas
  de marcas (6 marcas), e a `InFlight`, que serve a qualquer sessão do
  laboratório. os testes à mão da task-84 são 146 linhas de Rust
  (`apps/idestudio/tests/human_agent.rs:466-611`) e cobrem quatro cenários
  (stage, unstage da pessoa, discard, watcher); o do trecho cobre o do
  stage, duas vezes.

nada derrubou a demo. o que ela achou, para o orquestrador:

- o `settle` do harness da task-64 esconde toda corrida entre uma
  resposta do worker e o próximo ato; a `InFlight` resolve para o ide sem
  mexer no contrato do harness. se o harness virar crate (task-64,
  "fechamento da fase c"), assentar só as execuções pode virar opção da
  fixture, com uma linha no ADR sessions-become-regression-tests. P3.
- um trecho gravado com o bug tem os vereditos do bug (aqui, o request 6
  aceito no check e recusado no ponto de verdade), e o harness sempre
  compara o veredito com o gravado: o trecho do teste foi cortado antes
  do submit. marcar um veredito esperado, como as marcas já marcam um
  valor, deixaria o bug report inteiro virar teste. P3.
- o ponto de verdade segurou: com a view mentindo, o `CommitIf` recusou o
  commit de dois arquivos. a segunda checagem é o que fez a corrida custar
  uma recusa, não um commit errado.

## fase 1: demo 4, interface efêmera (feita)

o teste `apps/uigallery/tests/ephemeral.rs` (novo, no gate:
`cargo test -p uigallery --test ephemeral`) roda o roteiro do método sem
janela, sem ffmpeg e sem Claude: a Silences em 1200 por 800 servida por
uma porta de verdade num diretório temporário, a partir da semente
(cortes 1 e 2), com frames sintéticos postos no cache pelo
`stage_silences_frames` antes de cada tick, gravada pelo mesmo
`Recording` do shell. o agente `demo80` age sempre sobre uma observação
da página inteira feita logo antes; a pessoa clica no meio de onde o
botão aparece no frame, e o clique é gravado como input. cada medida
espera a página parar (a entrada e a saída da superfície) e usa o frame
que foi gravado naquele instante; o teste confere que as ligações
gravadas são as que o frame vivo tinha.

### a gravação

guardada fora do git, em `$CARGO_TARGET_TMPDIR/showcase-ephemeral-test.monster`
(reescrita a cada corrida); a lida abaixo foi copiada para a scratchpad
da sessão (sha256 `c9d20453...`). 153.658 bytes (eventos 71.146, chunks
52.539, ligações 8.923), 59 eventos, 61 frames.

```text
$ plev-inspect showcase-ephemeral-test.monster
session   rec 0.1.0, built from b26dc988
span      +0.000s to +0.132s, 59 events, 61 frames
actors    agent:demo80: 11 observations, 8 requests, 32 stages
          human: 1 request, 4 stages, 2 inputs
requests  9
  +0.037s  request 1  agent:demo80  select on silences_surface.proposer   completed
  +0.039s  request 2  agent:demo80  set_value on silences_surface.intent  completed
  +0.041s  request 3  agent:demo80  propose on silences_surface.propose   completed: surface = "proposed", nodes = 39
  +0.052s  request 4  human         apply_cut on silences.cut[1]          completed: decision = "apply"
  +0.056s  request 5  agent:demo80  set_value on silences_surface.intent  completed
  +0.061s  request 6  agent:demo80  propose on silences_surface.propose   failed: nothing here can be swapped for a table or a chart to compare: `proposed.silences.cuts` repeats its items over `silences:cuts` (as a table or a chart its items' ids would leave the surface), and no list of numbers is shown: each item's numbers are values of their own
  +0.066s  request 7  agent:demo80  set_value on silences_surface.intent  completed
  +0.071s  request 8  agent:demo80  propose on silences_surface.propose   completed: surface = "proposed", nodes = 33
  +0.116s  request 9  agent:demo80  close on silences_surface.close       completed: closed = "proposed"
refusals  0
```

a timeline sem as linhas de estágio (`--timeline`, cortada): a revisão
nasce num keyframe, a pessoa decide nela, a comparação troca a forma no
lugar, o close tira tudo.

```text
+0.040s     changes   silences_surface.intent:text: "" -> "the list with the video and the counts"; ...
+0.043s     snapshot  keyframe, 57 nodes: proposed added; proposed.card added; proposed.silences.video.original added; and 40 more
+0.052s     input     pointer_down on proposed.silences.cut[1].apply_cut
+0.052s     input     pointer_up on proposed.silences.cut[1].apply_cut
+0.053s     changes   silences.cut[1]:decision: "pending" -> "apply"; ...; silences.counts label "2 silences, 0 applied, ..." -> "2 silences, 1 applied, 0 rejected, 2.0 s cut"; and 10 more
+0.058s     changes   silences_surface.intent:text: "the list with the video and the counts" -> "I can't compare these"
+0.063s     changes   silences_surface.status:message: "proposed by the rules for ..." -> "nothing here can be swapped for a table or a chart to compare: ..."
+0.067s     changes   silences_surface.intent:text: "I can't compare these" -> "compare the original with the cuts"
+0.074s     snapshot  keyframe, 51 nodes: proposed.silences.video.duration removed; proposed.silences.video.shown removed; proposed.silences.threshold.value removed; and 11 more
+0.117s     changes   proposed removed; proposed.card removed; proposed.silences.video_views removed; and 34 more
```

### a leitura pelo inspector: a tela nunca existiu como tela fixa

```text
$ plev-inspect showcase-ephemeral-test.monster --control proposed --at 50
control proposed at +0.050s
  node          proposed: region "the list with the video and the counts", enabled
  source        rust/screen/src/project.rs:96:11, in build b26dc988

$ plev-inspect showcase-ephemeral-test.monster --control proposed --at 130
control proposed at +0.130s
  node          not in the recorded state at that instant

$ plev-inspect showcase-ephemeral-test.monster --control 'proposed.silences.cut[1].apply_cut' --at 50
control proposed.silences.cut[1].apply_cut at +0.050s
  node          proposed.silences.cut[1].apply_cut: button "apply cut", enabled
  actions       activate (reads proposed.silences.cut[1].decision:text)
  source        rust/screen/src/project.rs:104:23, in build b26dc988
  requests      0 touched it up to +0.050s (their target, or a state they read)

$ plev-inspect showcase-ephemeral-test.monster --control 'silences.cut[1]' --at 130
control silences.cut[1] at +0.130s
  state         ... decision = "apply", frame = "silences.video@4.017052"
  source        apps/uigallery/src/views/apps/silences/fixed_page/semantics.rs:256:30, in build b26dc988
  requests      1 touched it up to +0.130s (their target, or a state they read)

$ plev-inspect showcase-ephemeral-test.monster --why 4
request 4 (origin 0): apply_cut on silences.cut[1] by human, at +0.052s
  control       silences.cut[1]: list_item "0:03.0 to 0:05.0", enabled
  initiated by  human
  action        apply_cut, no parameter
  observed      not recorded: no observation by human before the request
                (a person acts on the frame on screen; the recorder keeps only the observations the app reports)
  precondition  what the action reads, as human saw it:
                  silences.cut[1]:decision = "pending"
  stages        +0.052s requested, accepted, executing, completed: decision = "apply"
  result        completed at +0.052s: decision = "apply"
```

a superfície e cada controle dela vêm do materializador
(`rust/screen/src/project.rs`), nunca de um arquivo da tela do app; o
controle do app que a decisão mudou vem de
`apps/uigallery/src/views/apps/silences/fixed_page/semantics.rs`, e é ele que guarda
a decisão depois que a superfície some. o teste confere isso nó a nó: os
40 nós da superfície carregam `source`, todos em `rust/screen/src/`; os
9 da tela fixa, todos em `apps/uigallery/src/views/apps/silences`.

dois achados da task-69 fase 4c continuam como estavam, e o inspector os
mostra aqui: o ato pela superfície chega ao dispatcher como a capacidade
do app (`apply_cut on silences.cut[1]`; o `--control` do botão da
superfície diz "0 requests touched it"), e a observação da pessoa não é
colocada ("not recorded"). só o input dela nomeia o botão gerado.

### a comparação contra a tela fixa, pelo critério do método

| critério | gerada | fixa | fonte na gravação |
| --- | --- | --- | --- |
| K1, a tarefa chega ao app | o clique em `proposed.silences.cut[1].apply_cut` é o request 4 da pessoa, `apply_cut` em `silences.cut[1]`, concluído | a mesma decisão no nó da fixa, `silences.cut[1]:decision = "apply"`, no fim | receipts e estado no fim |
| K2, corte 1 à vista em 1200x800 | 5 de 5 (frame, faixa, decisão, aplicar, rejeitar) | ao lado da gerada: 0 de 5 (a linha em y = 968, abaixo de 800); sozinha, antes da superfície: 5 de 5 (a linha inteira em y = 684) | ligações do frame medido |
| K2, corte 2 à vista em 1200x800 | 5 de 5 | ao lado: 0 de 5 (y = 1080); sozinha: 0 de 5 (a linha em y = 752, 8 de 68 px à vista) | idem |
| K3, o original ao lado do resultado | 2 frames à vista, lado a lado (`proposed.silences.video.original` e `.result`, 180 por 140 cada) | 1 frame (`silences.video`, 380 por 214); o resultado pede outro ato | ligações depois da comparação |
| K4, efêmera | no estado só de +0.043 s (logo depois do propose, +0.041 s) a +0.117 s (o close, +0.116 s); nenhuma ligação dela fora disso; nada no último frame | a tela fixa, no estado do começo ao fim | estado evento a evento e ligações frame a frame |
| K5, custo | a revisão em 0 ms pelas regras (39 nós); a comparação em 0 ms (33 nós); viveu 75 ms nesta corrida | não se aplica | receipts |
| K6, a recusa | "I can't compare these" termina `failed` com o motivo; os nós e os estados da superfície iguais antes e depois | não se aplica | estado antes e depois do request 6 |

nada derrubou a demo. a gerada não foi pior em K2 nem em K3, e em K2 foi
melhor que a fixa sozinha na janela padrão (a fixa mostra o segundo
corte só 8 px). a revisão nasceu da intenção, virou a comparação no
lugar (o mesmo id, `proposed`), e sumiu no close sem deixar nó nem
ligação.

o que isso não cobre, dito: só as regras (o Claude fica com o
orquestrador, e é por ele que "I can't compare these" pode virar outra
coisa); a semente em vez da detecção real e frames sintéticos; "à vista"
conta presença no frame, não quão bem uma pessoa decide (o frame da
gerada é menor que o da fixa); e ao lado da superfície a fixa fica com
meia largura e desce, pelo layout da própria página, por isso a fixa
sozinha entra como segundo controle. "não consigo comparar" virando
comparação na Silences pede o app declarar uma lista de durações (a
proposta da sonda, P3).

### privacidade da gravação

conferido pelo teste, em todos os estados gravados (2.521 nós lidos
evento a evento) e em todos os eventos:

- a entrada da pessoa: dois inputs, `pointer_down` e `pointer_up`, com o
  controle sob eles (`proposed.silences.cut[1].apply_cut`) e a posição;
  o tipo do evento não tem campo para conteúdo
  (`rust/record/src/event.rs:57-63`, `:79-91`).
- segredo: nenhum nó com `constraints.secret` na Silences, então nada a
  redigir nesta gravação; a redação de valor secreto como tamanho não foi
  exercida aqui (ela é da `sem::redact`, com testes próprios).
- câmera: nenhum nó com "camera" no id ou no rótulo; nenhuma câmera é
  aberta pela página.
- accessibility: a faixa das preferências não abriu, então nenhum nó
  dela está na gravação; aberta, os botões dela não têm estado
  (`apps/uigallery/src/views/apps/silences/generative/access.rs:169-186`),
  e o teste confere que um nó de accessibility, se houver, não guarda
  estado.
- uma observação: o texto da intenção fica gravado como valor do campo
  (`silences_surface.intent:text`), porque é estado projetado e não
  secreto. aqui quem escreveu foi o agente; se uma pessoa digitar, o que
  ela digitou chega à gravação pelo estado do campo, não pelo input. é o
  desenho da gravação (a task-63 grava o estado), não um vazamento da
  promessa sobre o input, e fica anotado para quem decidir se um campo de
  intenção deve ser secreto.

## fase 1: handoff (2026-09-25)

```text
Goal: task-80 fase 1, o método das cinco demos antes de rodar, o que já
      estava provado com a fonte, e as demos 3 e 4 de ponta a ponta
Status: pronto para o orquestrador, na branch task/80a-proof
```

### commits

- `e44e4234` task-80: the probe, the five demos against what exists,
  recording with no window, the bug for demo 3 and how its test catches it
- `c8a6c3a9` task-80: the method of the five demos, written before any new
  demo runs (a prova de que o método veio antes: nenhuma sessão nova foi
  gravada antes deste commit)
- `f008c897` task-80: what was already proven, demo by demo, with its source
- `46f272d3` task-80: demo 3, the task-84 race as a recorded bug report,
  and the test it becomes
- `b26dc988` task-80: demo 3 result, the bug report read at the instant and
  the proof that its test catches the race
- `ae5be30e` task-80: demo 4, the review born from an intent, turned into
  the comparison and closed, recorded with no window and held to the
  method's criterion
- `677b1ad9` task-80: demo 4 result, the recording read by the inspector,
  the comparison against the fixed screen and the privacy check
- o commit deste texto, com o ADR

a branch descartável `task/80a-bug84` (`84161012`, a corrida de volta, e
`034b951d`, o teste por cherry-pick) fica local, fora do merge; pode ser
apagada depois da validação.

### resumo

- a sonda (seção "fase 1: sonda"): a demo 5 pede funcionalidade nova (nem
  o ide nem o urnaui têm `SurfaceHost`), proposta como task-96; gravar sem
  janela funciona pelo showcase e pelo laboratório; o bug da demo 3 é a
  corrida da task-84 (o porto da 92 e a regra da 89 são do `wire`, fora do
  caminho do rerun); "não consigo comparar" é recusado pelas regras na
  Silences.
- o método (seção "o método das cinco demos"), com roteiro, controle,
  critério e o que derruba, para as cinco.
- o que já estava provado (seção "o que já estava provado"): a 1 pela
  task-62, a 2 só com pessoa roteirizada, a 3 em parte pela task-64, a 4
  pela task-69 e pelo checkpoint b, a 5 nada.
- a demo 3: o bug report gravado sem janela na branch com a corrida,
  lido no instante (as oito perguntas pelo arquivo), o trecho de 43.916
  bytes virando teste no gate; falha com a corrida (3 marcas divergem),
  passa sem ela (dez de dez); o rerun em repouso não a vê.
- a demo 4: o teste grava a revisão nascendo da intenção, a pessoa
  decidindo nela, a recusa de "I can't compare these", a comparação no
  lugar e o close; pela gravação, a superfície só existe entre o propose e
  o close e vem toda do materializador; em 1200 por 800 a gerada mostra as
  cinco peças dos dois cortes e o original ao lado do resultado, onde a
  fixa ao lado mostra nenhum corte e a fixa sozinha um corte de dois.
- a privacidade das duas gravações: input só como tipo e controle,
  nenhum segredo (nada a redigir nestas páginas), nenhuma câmera, nenhuma
  preferência de accessibility como estado; o texto de um campo de
  intenção fica gravado como estado (anotado).

### arquivos

- [`task-80-proof-qualitatively-different-software.md`](task-80-proof-qualitatively-different-software.md):
  sonda, método, provado, demos 3 e 4, handoff.
- [`0070-generative-runtime-results.md`](../../adr/Synthesis/0070-generative-runtime-results/0070-generative-runtime-results.md) (novo, proposed).
- `apps/idestudio/tests/rerun/inflight.rs` (novo): a fixture `InFlight` e o
  teste do trecho.
- `apps/idestudio/tests/rerun/eval.rs`: `answer_while` e `take` (o `drain` usa
  o mesmo), `fixture` público no módulo. `apps/idestudio/tests/rerun/main.rs`:
  o módulo novo.
- `apps/idestudio/tests/fixtures/sessions/bug84.monster` (43.916 bytes) e
  `bug84.marks`.
- `apps/uigallery/tests/ephemeral.rs` (novo): a demo 4.

nenhum código de produção, nenhum `Cargo.toml`, nenhuma linha do
`Cargo.lock`, nenhum baseline.

### interfaces

nenhuma adicionada nem alterada fora dos testes. o `Fixture` do harness,
as marcas e o relatório estão como estavam.

### validação

com o ambiente do briefing (o `PLEV_CLAUDE` no stub) e
`CARGO_NET_OFFLINE=true` em todo comando cargo:

- `./tool/tasks/fullcheck.sh`: verde (exit 0, 03:17 a 03:19), com a perna urnaui e o
  check wasm com `-D warnings`. workspace 2927 passam, 0 falham, 26
  ignorados (157 suítes); urnaui 85 passam, 0 falham, 1 ignorado (6
  suítes). total 3012 passam e 27 ignorados, contra 3010 e 27 na base: os
  dois testes novos. `df -h /`: 16 GB livres antes e depois.
- `cargo run --offline -p regress -- check` (com o
  `main.rs` tocado): 176 baselines, 176 iguais, nenhum frame desenhado,
  nenhuma GPU aberta.
- `cargo test -p idestudio --test rerun inflight`: dez de dez em
  `task/80a-proof`; `cargo test -p uigallery --test ephemeral`: dez de dez.
- na `task/80a-bug84`: o teste novo falha, o do a2ppa passa, os quatro da
  task-84 falham (saídas na demo 3).
- nenhuma janela aberta, nenhum clique ou tecla sintética, nenhuma chamada
  ao `claude`, nenhum ffmpeg nos testes.

### linha para o índice de ADRs

```text
| 2026-09-25 | [generative-runtime-results](generative-runtime-results.md) | proposed | o método das cinco demos escrito antes de rodar (roteiro, controle, critério, o que derruba); a 1 provada pela task-62; a 3 feita: a corrida da task-84 gravada como bug report, lida no instante pelo plev-inspect e virando teste no gate que a pega, com uma fixture que assenta só as execuções; a 4 feita: a revisão nasce da intenção, vira comparação e some, medida pela gravação contra a tela fixa; faltam a 2 e a pergunta do núcleo (brenner), a 5 (task-96) e o Claude de verdade (orquestrador) (task-80 fase 1) |
```

### decisões

- a demo 3 pela corrida da task-84, não pelo porto da 92 nem pela regra
  da 89: as duas são do `wire`, e o rerun despacha no workbench.
- o trecho do teste cortado antes do submit: o bug mudou o veredito dele,
  e o harness compara o veredito com o gravado. as marcas dizem o que a
  view precisa ter.
- a `InFlight` em teste, ao lado da fixture do ide, e não uma opção do
  harness: o contrato do harness fica como estava.
- a demo 4 como teste no gate, não como exemplo: mede pela gravação e
  guarda a propriedade efêmera. a comparação é "compare the original with
  the cuts" (a que existe na Silences); "I can't compare these" entra
  como a recusa medida.
- K2 com dois controles: a fixa ao lado da gerada (o que o método
  escreveu) e a fixa sozinha, antes da superfície, porque ao lado dela a
  fixa fica com meia largura.

### limitações

- a demo 4 só pelas regras, da semente, com frames sintéticos; "à vista"
  conta presença no frame, não quão bem uma pessoa decide.
- a demo 3 cobre o cenário do stage da corrida; os testes à mão da
  task-84 cobrem quatro.
- a recusa de "I can't compare these" na Silences continua (o app não
  declara uma lista de números).
- os dois achados da task-69 fase 4c continuam: o ato pela superfície
  gravado como a capacidade do app, e a observação da pessoa não colocada.

### o que falta, e com quem

- a demo 1, os dois braços de controle (agente por screenshot, que pede
  um caminho de clique por pixel no laboratório; agente por snapshot de
  accessibility de terceiros): o orquestrador, com o Claude de verdade.
- a demo 2, o handoff com o brenner na janela do ide: o brenner, com o
  orquestrador.
- a pergunta do núcleo (checkpoint c): o brenner.
- a demo 5: a task-96 (proposta na sonda), aberta pelo orquestrador.
- o Claude de verdade nas superfícies da demo 4, e uma pessoa de fora nas
  demos 2 e 4: o orquestrador.
- P3, para o orquestrador decidir: a opção de assentar só as execuções no
  harness (se ele virar crate); marcar um veredito esperado; a lista de
  durações na Silences; se um campo de intenção deve ser secreto.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0050: sessions become regression tests](../../adr/Recording/0050-sessions-become-regression-tests/0050-sessions-become-regression-tests.md)
- [ADR 0070: generative runtime results](../../adr/Synthesis/0070-generative-runtime-results/0070-generative-runtime-results.md)
- [task-60: accessibility as a projection of the contract](../task-60-accessibility-projection/task-60-accessibility-projection.md)
- [task-62: proof in the ide: stale-state rejection and human-agent handoff](../task-62-ide-human-agent-proof/task-62-ide-human-agent-proof.md)
- [task-63: session flight recorder](../task-63-flight-recorder/task-63-flight-recorder.md)
- [task-64: session inspector, bug report and session for testing](../task-64-session-inspector-and-tests/task-64-session-inspector-and-tests.md)
- [task-68: plev IR and verifier](../task-68-plev-ir-and-verifier/task-68-plev-ir-and-verifier.md)
- [task-69: ephemeral and generated surfaces](../task-69-ephemeral-generative-surfaces/task-69-ephemeral-generative-surfaces.md)
- [task-70: adaptive transformations](../task-70-adaptive-transformations/task-70-adaptive-transformations.md)
- [task-80: proof of qualitatively different software](task-80-proof-qualitatively-different-software.md)
- [task-84: an old refresh does not undo what the ide has already accepted](../task-84-ide-refresh-never-goes-back/task-84-ide-refresh-never-goes-back.md)
- [task-86: the ide people use opens the agent port](../task-86-the-ide-opens-the-agent-port/task-86-the-ide-opens-the-agent-port.md)
- [task-87: a terminal stage carries the revision the app was left in](../task-87-a-terminal-stage-carries-the-revision/task-87-a-terminal-stage-carries-the-revision.md)
- [task-89: an act relies only on what was shown to the agent](../task-89-an-act-rests-only-on-what-was-shown/task-89-an-act-rests-only-on-what-was-shown.md)
- [task-92: the agent asks the person](../task-92-the-agent-asks-the-person/task-92-the-agent-asks-the-person.md)
- [task-93: the person in the lab answers](../task-93-the-lab-person-answers/task-93-the-lab-person-answers.md)
- [task-94: the refusal that names a person points to the question](../task-94-the-refusal-points-at-the-question/task-94-the-refusal-points-at-the-question.md)
- [task-96: a generated surface in the ide](../task-96-ide-generated-surface/task-96-ide-generated-surface.md)
- [five demos of qualitatively different software, each with its method written before it ran](../../Research/Exploring/five-demos.md)
- [plev: thesis of the generative interaction runtime](../../Research/Synthesis/generative-runtime-thesis.md)
<!-- /related -->
