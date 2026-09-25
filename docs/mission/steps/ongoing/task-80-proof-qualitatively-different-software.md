---
project: plev
audience: [ai-agents, contributors]
status: ongoing
last-updated: 2026-09-25
domain: generative-runtime
task: task-80
wave: 8
depends: [task-62, task-64, task-69, task-70]
branch: task/80-proof
---

# task-80: prova de software qualitativamente diferente

## objetivo

provar que o modelo produz software qualitativamente diferente, não só
mais rápido: interfaces efêmeras, handoff humano-agente, stale-state
rejection, inspectability e UI generativa, em aplicações reais, com
método, com números onde números cabem e com a pergunta do núcleo
respondida por quem usa.

## ligação com a tese

goal 10 do programa. tese seções 28 (não afirmar primeiro; a hipótese é o
contrato unificado; a frase "interfaces que humanos e agentes operam, e que
qualquer pessoa pode inspecionar depois"; a pergunta do núcleo), 33 (o que
o plev pode se tornar), e o experimento do arquivo de concepção.

## estado atual no código

depende das entregas: 62 (ide com rejeição e handoff, experimento com
métricas), 63 e 64 (sessão gravada e inspecionada, bug report, sessão para
teste), 69 (revisão de silêncios e comparação geradas), 70 (transformação
adaptativa e capacidade composta).

## escopo

cinco demonstrações, cada uma com roteiro, gravação (`.monster`) e
resultado escrito:

1. stale-state rejection: o experimento do ide completo (task-62, fase c),
   com as métricas (sucesso, tempo, dados e tokens para o agente,
   recuperação de erro, custo de gravação) e as comparações contra
   screenshot e contra a alternativa estruturada existente.
2. handoff humano-agente: o agente prepara um commit, o brenner assume,
   muda parte, devolve, o agente termina; o receipt de cada passo diz quem
   fez.
3. inspectability: um bug real (ou provocado) enviado como trecho de
   sessão, aberto no inspector no instante exato, com as perguntas da tese
   respondidas na tela, e virando teste de regressão que pega a quebra.
4. interface efêmera: a revisão de silêncios nasce da intenção, vira
   comparação, some; nunca existiu como tela.
5. UI generativa em aplicação real: além da demo de vídeo, uma intenção num
   app que já existe (no urnaui, "compara esses dois resultados de busca";
   ou no ide, "revisa esses três commits") gera uma superfície verificada
   que o usuário opera.

método:

- cada demo tem uma alternativa de controle (tela fixa bem feita; agente
  por screenshot; agente por snapshot de accessibility de terceiros) e o
  critério de comparação escrito antes.
- a pergunta do núcleo, com o brenner: ao abrir o primeiro `.monster` de uma
  delegação real, o que ele consegue descobrir, e isso basta para confiar
  na delegação? a resposta é escrita com as palavras dele.

critérios de parada, herdados do programa: se a rejeição e os receipts não
mudarem como a delegação é feita, ou se as superfícies geradas forem piores
que as fixas nas mesmas tarefas, a direção é reavaliada com os dados, não
defendida.

## fora de escopo

marketing, texto de blog (o destino blog passa pela skill de escrita, em
outra sessão), benchmarks isolados sem demonstração.

## dependências

62, 64, 69 e 70 integradas.

## superfície sob responsabilidade

roteiros e resultados em `docs/mission/steps/`, gravações de sessão (fora
do git se forem grandes), páginas Apps Demo do showcase para as demos que
couberem lá, ADR de resultado `docs/ADR/generative-runtime-results.md`.

## interfaces que consome

tudo do programa, sem alterar.

## interfaces que pode alterar

nenhuma. defeito encontrado vira task nova ou volta para a task dona.

## critérios de aceitação

- as cinco demonstrações executadas, gravadas, com resultado escrito.
- a comparação de cada uma contra a alternativa de controle, com o
  critério definido antes.
- a pergunta do núcleo respondida.
- a decisão do checkpoint c registrada no programa.

## testes esperados

as demos que viram teste (a 3 obrigatoriamente) entram no gate.

## riscos e perguntas abertas

- viés de quem constrói avaliando o que construiu: sempre que possível,
  uma pessoa de fora usa as demos e o relato dela entra no resultado.

## commits esperados

um por demonstração com roteiro e resultado, `task-80:` no início, e o ADR
de resultado.

## pare e reporte se

- uma demo exigir funcionalidade que nenhuma task entregou (vira task, não
  atalho).

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
| 1, stale-state rejection | o experimento do ide em cinco rodadas, com braços e hipóteses escritos antes: A0 roteirizado, A LLM pelo contrato, B o `git` na linha de comando, C o volume de um screenshot (`checked/task-62-ide-human-agent-proof-done.md`, "os braços", "hipóteses (escritas antes de rodar)", "resultado da fase c" e os resultados da segunda à quinta rodada) | as duas alternativas de controle do escopo que ninguém rodou: um agente por screenshot (o braço C é "comparação de volume, não de capacidade" e "nenhum agente por screenshot foi rodado", mesma task, "as hipóteses", H4) e um agente por snapshot de accessibility de terceiros (programa, seção 10: "se não houver comparação justa, a limitação é registrada") | o orquestrador, com o Claude de verdade | sim, para o braço por screenshot: o `wire` só age por identidade (`Act.target: SemId`, `crates/wire/src/proto.rs:135-146`), e clique por coordenada de tela é proibido aqui; um braço que receba "clique em x, y" do agente e resolva pelo `Compositor::hit` no frame headless é trabalho novo do laboratório (`crates/ide/src/bin/lab/**`). o de accessibility de terceiros pede a janela e a árvore do AccessKit lida por fora (task-60): orquestrador com janela |
| 2, handoff humano-agente | com uma pessoa roteirizada, não com o brenner: a quarta e a quinta rodada (a pessoa tira um arquivo antes do submit e responde ao modal; `task-62...`, "resultado da quarta rodada" e "resultado da quinta rodada"); a pessoa de verdade só como CGEvent sintético numa janela real (`checked/task-86-the-ide-opens-the-agent-port-done.md:148-163`) | o roteiro do escopo com o brenner: o agente prepara, ele assume, muda parte, devolve, o agente termina, e cada receipt diz quem fez | o brenner, com o orquestrador (janela, `PLEV_AGENT=1 PLEV_RECORD=1`, o Claude de verdade) | não: a porta na janela do ide (task-86), a gravação do ide (task-63d), a pergunta à pessoa (task-92) e a pessoa que responde no laboratório (task-93) existem |
| 3, inspectability | a fase a da task-64 achou um defeito real lendo a sessão do A2 da primeira rodada no instante do submit: o `changes:staged` observado "outside the scope observed", o que virou a task-89 (`checked/task-64-session-inspector-and-tests-done.md`, "saídas de aceite" e "fase a integrada"); a fase c transformou a sessão a2ppa em teste de regressão (`crates/ide/tests/rerun/lab.rs:165`) e provou à mão, fora do gate, que ele pega uma precondição quebrada de propósito (mesmo arquivo, "o teste gerado e a prova de que pega regressão") | um bug real do histórico, enviado como trecho, aberto no instante, com as perguntas da tese respondidas, e um teste no gate que pega o bug reintroduzido (a fase c provou com uma quebra provocada, e à mão) | este agente | não, com uma fixture nova só em teste (item c) |
| 4, interface efêmera | a demo do editor de vídeo gravada sem janela pela 69 fase 4c (`crates/showcase/fixtures/demo/silences-demo.monster`, 186.837 bytes, lida por `crates/showcase/tests/demo.rs`; `checked/task-69-ephemeral-generative-surfaces-done.md`, "a demo (arquivo, tamanho, o que tem dentro)"); o checkpoint b, duas rodadas, a revisão gerada contra a tela fixa na mesma tarefa (mesma task, "checkpoint b, primeira rodada" e "checkpoint b, segunda rodada") | a gravação com o código de hoje (a revisão das regras mudou na task-70 fase 2), a leitura pelo inspector mostrando que a superfície nunca foi tela, e a comparação com critério escrito antes para o passo da comparação, que o checkpoint b não mediu (ele comparou só a revisão) | este agente | não para o que as regras fazem. "não consigo comparar" pelas regras é recusado na Silences: o único `List` do app são os ids dos cortes e a duração é um `num` por corte (`checked/task-70-adaptive-transformations-done.md`, "(b) a gramática das regras" e "a demo"); a comparação que existe na Silences é a da gramática de superfície, "compare the original with the cuts" (`crates/propose/src/change.rs:5-8`). trocar as durações por gráfico pede o app declarar uma lista de durações (task nova, abaixo) |
| 5, UI generativa em app real | nada: as superfícies geradas vivem só nas páginas Surface e Silences do showcase | uma intenção no ide ("revisa esses três commits") ou no urnaui gerando uma superfície que a pessoa opera | uma task nova | sim: nem o ide nem o urnaui têm `SurfaceHost` (`SurfaceHost` aparece só em `crates/surface/src/host.rs` e em `crates/showcase/src/view/apps/**`; `crates/ide/Cargo.toml` não depende de `surface`, `propose` nem `uir`; `crates/urnaui/Cargo.toml` também não). o que já ajuda: o `uir` tem fixtures no domínio do ide, verificadas contra o registro que a projeção do ide dá (`crates/uir/fixtures/ide-review-commit.ir.json`, `ide-commit-form.ir.json`, `ide-workbench.snapshot.json`, escrito por `crates/ide/tests/uir_fixture.rs`) |

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
  integradas. superfície: `crates/ide/src/**` (a janela e a projeção),
  `crates/ide/Cargo.toml` (três dependências de workspace, nenhuma
  externa). o que a derruba: a superfície pior que o painel de stacks
  mais o diff para revisar três commits, pelo critério da demo 5 abaixo.
- **o gráfico das durações na Silences** (P3, já anotado pelo programa
  na linha da task-70): o app declarar a lista das durações como estado,
  para "não consigo comparar" trocar a lista por gráfico pelas regras.
  não bloqueia a demo 4.

### (b) gravar sem janela

sim, sem janela e sem GPU, pelos dois caminhos que já existem:

- o showcase: o exemplo `demo` roda a Silences servida pela porta num
  `Compositor` sem janela (`crates/showcase/examples/demo/main.rs:12-16`,
  `shell.rs:50` `draw`, `:61` `turn`), e grava pelo mesmo gancho do shell,
  `showcase::app::record::Recording::frame`
  (`crates/showcase/src/app/record.rs:171`), que chama
  `rec::Recorder::frame` (`crates/rec/src/recorder.rs:105`): a cena e as
  ligações de cada frame, sem pixel. o teste
  `crates/showcase/tests/record.rs` faz o mesmo no gate, com a página
  semeada (nenhum ffmpeg) e uma porta num diretório temporário.
- o ide: o laboratório (`crates/ide/src/bin/lab/lab.rs:286` `record`)
  grava estado e frame a cada volta em que algo mudou, com o renderer
  headless (`crates/ide/src/bin/lab/bench.rs:124`), sem janela.

então a demo 4 grava sem o orquestrador. o que muda na Silences sem o
ffmpeg: a detecção real e os frames do vídeo. um teste parte da semente
(os cortes 1 e 2) e põe frames sintéticos no cache pelo gancho público
`ShowcaseView::stage_silences_frames`
(`crates/showcase/src/view/apps/silences/generated/page.rs:530`), como o
`regress` faz (`crates/showcase/examples/regress/state.rs:314`, `:431`).
nenhuma janela: nada para o orquestrador rodar com `PLEV_BACKGROUND=1`.

### (c) a demo 3: qual bug, como reproduzir, o inspector e o teste

as perguntas da tese (`generative-runtime-thesis.md:885-901`, seção 12):
qual controle era esse, qual era seu estado, qual foi a sequência
anterior, qual ação ocorreu, quem iniciou, qual era a precondição, o que
mudou, qual foi o resultado. e a seção "sessão vira teste" (`:903`).

o caminho que transforma sessão em teste é o harness da task-64 fase c:
cada ato do trecho refeito na ordem gravada contra uma `Fixture` do app
(`crates/ide/tests/rerun/harness.rs:38-65`), e o veredito, o estágio
terminal e as marcas comparados com a gravação. a fixture do ide chama
`Workbench::dispatch` direto (`crates/ide/tests/rerun/lab.rs:102-104`), e
antes de cada ato o harness assenta o app (`harness.rs:329-336`), menos
onde a gravação tinha uma execução em voo (`harness.rs:242-246`); o
`settle` da fixture pede um refresh e aplica o worker até todo refresh
devido responder (`lab.rs:74-96`, `:117-121`).

os três candidatos:

- **o porto que não acordava (task-92).** a correção é do `wire` (o
  sink do porto acorda o loop, `crates/wire/src/port.rs:166-172` e
  `crates/wire/src/sink.rs:33`, `Waits`), e o harness nunca passa pela porta: o rerun despacha no
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
  `crates/ide/src/session.rs:262-269` (a leitura velha descartada em
  `apply`) sobre `Owed` e `Reads` (`:42-127`). ele mora no workbench, o
  mesmo caminho que o rerun roda, e aparece no estado projetado, que a
  gravação guarda e o inspector lê: um arquivo que o ide aceitou no index
  sai do `changes:staged` sem request nenhum. e o teste escrito à mão da
  época existe para comparar (`crates/ide/tests/human_agent.rs:491`,
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
(`crates/git/src/client.rs:125-130`, pela task-84), então a ordem
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
  `GitSession::apply` (`crates/ide/src/session.rs:266-269`, a essência do
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
  `crates/ide/tests/rerun/`, com marcas e a fixture que assenta só as
  execuções (sonda, item c); se o bug mudou algum veredito gravado, o
  teste nasce do mesmo cenário gravado em `task/80a-proof`; (5) o teste
  roda em `task/80a-bug84` e em `task/80a-proof`, o do a2ppa com o
  `settle` de sempre roda nas duas como controle, e o novo roda dez vezes
  seguidas em `task/80a-proof`.
- **alternativa de controle.** (i) o bug report como mídia: só os frames
  do mesmo trecho (a trilha visual, sem a semântica), o que um vídeo
  mostra; (ii) o rerun do harness como está (assenta tudo antes de cada
  ato); (iii) o teste escrito à mão da task-84
  (`crates/ide/tests/human_agent.rs:491-611`).
- **critério.** K1: das oito perguntas, quantas o `plev-inspect` responde
  pelo trecho sozinho, com a resposta conferida contra o código (o
  defeito em `crates/ide/src/session.rs`), contra quantas os frames
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

- **roteiro.** um teste em `crates/showcase/tests/` grava, sem janela, a
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
    (`crates/surface/src/**`), nunca um arquivo da tela do app, enquanto
    os nós da fixa apontam para `crates/showcase/src/view/apps/silences/**`;
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
  (`crates/ide/tests/rerun/lab.rs:165`), e com a precondição do `submit`
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
  Claude (`crates/showcase/fixtures/demo/silences-demo.monster`, 186.837
  B; `silences-demo-claude.monster`, 861.354 B), e
  `crates/showcase/tests/demo.rs` confere no gate que a superfície está no
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
(`crates/ide/src/session.rs:266-269`) a resposta continua contada, mas é
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
  `crates/ide/tests/fixtures/sessions/bug84.monster` (abaixo de 1 MB).

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
  source        crates/ide/src/workbench/project.rs:88:26, in build 84161012
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

as oito perguntas da tese (`generative-runtime-thesis.md:893-900`),
respondidas só pelo arquivo:

| pergunta | resposta do inspector |
| --- | --- |
| qual controle era esse | `changes` e as linhas `changes.file[src/session.rs]` e `changes.file[tests/login_test.rs]`, projetadas em `crates/ide/src/workbench/project.rs:74` e `:88` |
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
(`crates/ide/src/session.rs:271-275`, `set_files`), e a resposta que
chegou ali foi lida antes do stage.

### o controle (i): os frames sozinhos

a trilha visual guarda a cena e as ligações de cada frame, nada de
estágio, ator ou request (`crates/rec/src/visual.rs:1-23`). pelos frames,
um vídeo mostraria a linha do arquivo mudando de grupo e voltando:
responde, no máximo, qual controle, o estado desenhado e que algo mudou
(3 das 8). não diz que um stage estava em execução, que ninguém agiu,
qual era a precondição do submit nem que o ponto de verdade recusou. e os
dois lapsos duraram 29 ms (+0.048 a +0.077 s) e 24 ms (+0.077 a +0.101
s), menos que um quadro de um vídeo a 30 fps (33 ms): uma gravação de
tela pode não ter nem o quadro.

### o teste

`crates/ide/tests/rerun/inflight.rs` (novo), no binário `rerun` do ide
(`cargo test -p ide --test rerun`):

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
thread 'inflight::the_bug84_excerpt_keeps_every_staged_file_with_the_reads_in_flight' panicked at crates/ide/tests/rerun/inflight.rs:103:5:
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
  contra o código (`crates/ide/src/session.rs:266-275`); os frames
  sozinhos respondem no máximo três, e os lapsos são mais curtos que um
  quadro de vídeo a 30 fps.
- **K2:** o teste falha na branch com a corrida (3 marcas divergem) e
  passa em `task/80a-proof`, dez de dez; o controle em repouso passa nas
  duas.
- **K3:** 43.916 bytes no git; o bug report inteiro, 71.580.
- **K4:** para ter o teste: o trecho exportado por um comando, 26 linhas
  de marcas (6 marcas), e a `InFlight`, que serve a qualquer sessão do
  laboratório. os testes à mão da task-84 são 146 linhas de Rust
  (`crates/ide/tests/human_agent.rs:466-611`) e cobrem quatro cenários
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

o teste `crates/showcase/tests/ephemeral.rs` (novo, no gate:
`cargo test -p showcase --test ephemeral`) roda o roteiro do método sem
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
  source        crates/surface/src/project.rs:96:11, in build b26dc988

$ plev-inspect showcase-ephemeral-test.monster --control proposed --at 130
control proposed at +0.130s
  node          not in the recorded state at that instant

$ plev-inspect showcase-ephemeral-test.monster --control 'proposed.silences.cut[1].apply_cut' --at 50
control proposed.silences.cut[1].apply_cut at +0.050s
  node          proposed.silences.cut[1].apply_cut: button "apply cut", enabled
  actions       activate (reads proposed.silences.cut[1].decision:text)
  source        crates/surface/src/project.rs:104:23, in build b26dc988
  requests      0 touched it up to +0.050s (their target, or a state they read)

$ plev-inspect showcase-ephemeral-test.monster --control 'silences.cut[1]' --at 130
control silences.cut[1] at +0.130s
  state         ... decision = "apply", frame = "silences.video@4.017052"
  source        crates/showcase/src/view/apps/silences/semantics.rs:256:30, in build b26dc988
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
(`crates/surface/src/project.rs`), nunca de um arquivo da tela do app; o
controle do app que a decisão mudou vem de
`crates/showcase/src/view/apps/silences/semantics.rs`, e é ele que guarda
a decisão depois que a superfície some. o teste confere isso nó a nó: os
40 nós da superfície carregam `source`, todos em `crates/surface/src/`; os
9 da tela fixa, todos em `crates/showcase/src/view/apps/silences`.

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
  (`crates/rec/src/event.rs:57-63`, `:79-91`).
- segredo: nenhum nó com `constraints.secret` na Silences, então nada a
  redigir nesta gravação; a redação de valor secreto como tamanho não foi
  exercida aqui (ela é da `sem::redact`, com testes próprios).
- câmera: nenhum nó com "camera" no id ou no rótulo; nenhuma câmera é
  aberta pela página.
- accessibility: a faixa das preferências não abriu, então nenhum nó
  dela está na gravação; aberta, os botões dela não têm estado
  (`crates/showcase/src/view/apps/silences/generated/access.rs:169-186`),
  e o teste confere que um nó de accessibility, se houver, não guarda
  estado.
- uma observação: o texto da intenção fica gravado como valor do campo
  (`silences_surface.intent:text`), porque é estado projetado e não
  secreto. aqui quem escreveu foi o agente; se uma pessoa digitar, o que
  ela digitou chega à gravação pelo estado do campo, não pelo input. é o
  desenho da gravação (a task-63 grava o estado), não um vazamento da
  promessa sobre o input, e fica anotado para quem decidir se um campo de
  intenção deve ser secreto.

## demo 5, UI generativa em aplicação real (feita pela task-96)

rodada pelo laboratório sem janela, `ide-lab --arm review`, no build
`2e6c9e9a` da branch `task/96a-ide-surface`, sobre o repositório da
revisão do laboratório (`Repo::build_review` em
`crates/ide/src/lab_repo.rs`: o commit inicial, três commits por cima em
datas fixas, os shas iguais em toda corrida, e um `notes.txt` na árvore
que nenhum commit tem). só as regras: sem porta, sem agente, sem Claude.
a pessoa roteirizada clica no meio de cada controle como o frame na tela
o desenha (achado pelo id nas ligações do frame, pelo `Workbench::click`,
como a resposta dela na task-93) e digita a intenção uma tecla por vez.
primeiro o controle: os três cartões de commit do painel de stacks, um
clique cada, o diff no painel de diff. depois a superfície: o campo da
linha, "review these three commits", propor, o "show diff" de cada
commit na superfície, fechar. o teste `crates/ide/tests/lab_review.rs`
roda o mesmo braço no gate.

### a gravação

`review.monster`, guardada fora do git (na scratchpad da sessão, com o
`report.json` e os PNGs dos três momentos), 128.824 bytes, 84 eventos,
19 frames, sha256 `9dfac51d...`.

```text
$ plev-inspect review.monster
session   rec 0.1.0, built from 2e6c9e9a
span      +0.000s to +0.179s, 84 events, 19 frames
actors    human: 8 requests, 32 stages, 35 inputs
requests  8
  +0.017s  request 1  human  show_diff on stacks.commit[13ed9dbe...]  completed
  +0.035s  request 2  human  show_diff on stacks.commit[78f371df...]  completed
  +0.049s  request 3  human  show_diff on stacks.commit[847f583a...]  completed
  +0.085s  request 4  human  propose on ide_surface.propose           completed: surface = "proposed", nodes = 24
  +0.115s  request 5  human  show_diff on stacks.commit[13ed9dbe...]  completed
  +0.129s  request 6  human  show_diff on stacks.commit[78f371df...]  completed
  +0.142s  request 7  human  show_diff on stacks.commit[847f583a...]  completed
  +0.176s  request 8  human  close on ide_surface.close               completed: closed = "proposed"
refusals  0
```

(os shas cortados aqui; na saída eles vêm inteiros.) a timeline, sem as
linhas de estágio e com as 26 teclas numa linha só:

```text
+0.017s  input     pointer_down on stacks.commit[13ed9dbe...]
+0.017s  changes   diff:target: "" -> "stacks.commit[13ed9dbe...]"
+0.030s  changes   diff:lines: added Cargo.toml: @@ -1,4 +1,4 @@, ...; diff:count: 0 -> 14
...
+0.082s  input     pointer_down on ide_surface.intent
+0.083s  input     text (26 vezes, nenhuma com conteúdo)
+0.083s  changes   ide_surface.intent:text: "" -> "review these three commits"; ...
+0.085s  input     pointer_down on ide_surface.propose
+0.086s  snapshot  keyframe, 43 nodes: proposed added; proposed.card added; proposed.stacks added; and 26 more
+0.115s  input     pointer_down on proposed.stacks.commit[13ed9dbe...].show_diff
+0.115s  changes   diff:target: "stacks.commit[847f583a...]" -> "stacks.commit[13ed9dbe...]"; ...
...
+0.176s  input     pointer_down on ide_surface.close
+0.176s  changes   proposed removed; proposed.card removed; proposed.stacks removed; and 25 more
```

### a leitura pelo inspector

```text
$ plev-inspect review.monster --control proposed --at 100
control proposed at +0.100s
  node          proposed: region "review these three commits", enabled
  source        crates/surface/src/project.rs:96:11, in build 2e6c9e9a

$ plev-inspect review.monster --control proposed --at 179
control proposed at +0.179s
  node          not in the recorded state at that instant

$ plev-inspect review.monster --control 'proposed.stacks.commit[78f371df...].files' --at 128
  node          proposed.stacks.commit[78f371df...].files: table "files", enabled
  state         rows = [src/login.rs]
  source        crates/surface/src/project.rs:104:23, in build 2e6c9e9a

$ plev-inspect review.monster --control 'proposed.stacks.commit[78f371df...].show_diff' --at 128
  node          ...show_diff: button "show diff", enabled
  actions       activate (reads proposed.diff.target:text)
  source        crates/surface/src/project.rs:104:23, in build 2e6c9e9a
  requests      0 touched it up to +0.128s (their target, or a state they read)

$ plev-inspect review.monster --why 6
request 6 (origin 0): show_diff on stacks.commit[78f371df...] by human, at +0.129s
  control       stacks.commit[78f371df...]: list_item "login remembers the user", enabled
  state         summary = "login remembers the user", author = "Lab Author", files = [src/login.rs]
  initiated by  human
  observed      not recorded: no observation by human before the request
                the recorded state never had revision cebc8640d6caed52 in this window
  precondition  diff:target = "stacks.commit[13ed9dbe...]"
  stages        +0.129s requested, accepted, executing, completed
```

a superfície e cada controle dela vêm do materializador
(`crates/surface/src/project.rs`); os commits vêm de
`crates/ide/src/workbench/project.rs` (o teste confere nó a nó no estado
gravado). os dois achados da task-69 fase 4c continuam: o ato pela
superfície chega ao dispatcher como a capacidade do app, e só o input da
pessoa nomeia o botão gerado; e a observação da pessoa não é colocada. no
ide ela é ainda um passo mais longe: o request da superfície é observado
no snapshot do app sozinho (o que a superfície mostra), e o gravador
guarda o do workbench inteiro (o app, a linha e a superfície), então a
revisão que o request carrega nunca aparece na gravação.

### o resultado contra o painel fixo, pelo critério do método

| critério | superfície gerada | painel fixo | fonte |
| --- | --- | --- | --- |
| atos para ver os arquivos e o diff dos três | 5 cliques (o campo, propor, três "show diff") e 26 teclas; fechar é mais 1 clique | 3 cliques, um cartão de commit por vez | receipts e inputs gravados |
| tempo, relógio do workbench (máquina, não pessoa) | 73 ms do primeiro ato ao terceiro diff; a proposta pelas regras em 0 ms (o receipt do `propose` abre e fecha no mesmo milissegundo, 24 nós) | 47 ms | `report.json` |
| arquivos dos três à vista de uma vez | 3 de 3: a tabela de arquivos de cada commit, inteira no frame, desde a proposta | 1 de 3 no máximo: o painel de stacks não mostra arquivo nenhum, o de diff mostra um commit por vez, e o cabeçalho do hunk só nomeia o arquivo quando o commit muda mais de um (o `src/login.rs` do commit de um arquivo nunca aparece) | ligações e `diff:lines` |
| linhas do diff à vista sem rolar | 5 de 14, 5 de 7 e 5 de 18: uma tabela com a altura limitada pelo token `dropdown_max_h`, linhas de 48 px, sem cor e sem número de linha; o resto rola dentro dela | 14 de 14, 7 de 7 e 18 de 18: o painel de diff, mono, colorido, com número de linha | ligações e a geometria de cada vista |
| os três commits à vista | 3 cartões inteiros, com a mensagem inteira, o autor, os arquivos e o botão | 3 cartões inteiros, a mensagem cortada, o sha e o autor | ligações |
| cada ato no dispatcher, com o receipt da pessoa | os 5 requests (`propose`, três `show_diff`, `close`) de `human`, `completed`; o `show_diff` chega como a capacidade do app | os 3 requests de `human`, `completed` | summary do `plev-inspect` |
| nada da superfície depois de fechar | 0 nós, 0 ligações, 0 superfícies no host; no estado gravado só de +0.086 s (logo depois do propose) até antes do close (+0.176 s); o último frame não liga nada dela; o diff que ela abriu fica no painel fixo, que é o estado do app | não se aplica | `report.json` e a sessão |

o que isso diz, pelo critério escrito antes: o resultado é misto, e numa
parte a superfície sai pior. ela é melhor no que o painel fixo não tem:
os arquivos dos três commits de uma vez (3 contra no máximo 1, e o arquivo
do commit de um arquivo só nunca aparece no painel fixo), e os três com
a mensagem inteira num lugar só. ela é pior em atos (5 cliques e 26
teclas contra 3 cliques, para ver os arquivos e o diff dos três) e no
diff: mostra 5 linhas por vez contra o diff inteiro destes três commits,
sem cor e sem número de linha. o diff é um por vez nos dois. então, na
tarefa inteira ("ver os arquivos e o diff dos três"), a superfície não
foi melhor que o painel fixo, e no diff foi pior: pelo método, a demo 5
cai no diff e se sustenta nos arquivos. os atos fora do dispatcher e o
rastro depois de fechar, as outras duas coisas que a derrubam, não
aconteceram.

o que falta para a superfície não perder no diff não é do ide: o
vocabulário do IR não tem um componente de código ou de diff, e a tabela
é o único jeito de mostrar uma lista de textos (linhas de 48 px, um
único token de altura, `dropdown_max_h`). um componente novo pede o widget
no comps primeiro e um ADR (`crates/uir/src/vocab.rs`), decisão do
orquestrador. os três diffs de uma vez, que o painel fixo também não tem,
pediriam um estado de diff por commit na projeção (P3 da sonda da
task-96).

o que isso não cobre, dito: só as regras (o Claude pelo mesmo caminho
foi testado só com o modelo falso, e a prova com o Claude de verdade é do
orquestrador); a pessoa roteirizada, sem hesitação, então o tempo é de
máquina e o custo humano está nos atos e nas teclas; diffs pequenos, que
cabem inteiros no painel fixo; e o painel fixo medido com a linha ligada,
como o ide abre agora: a coluna tira a largura de um card da janela, os
painéis de changes e de diff guardam a largura deles e só a coluna de
stacks fica mais estreita, então a altura do diff é a mesma. a demo com
janela pelo brenner fica aberta.

### privacidade da gravação

- a entrada da pessoa: 9 `pointer_down`, cada um com o controle sob ele e
  a posição, e 26 `text`, sem alvo e sem conteúdo (o tipo do evento não
  tem campo para ele).
- a intenção chega à gravação como o estado projetado da linha
  (`ide_surface.intent:text`), como a demo 4 anotou para a Silences.
- segredo: nenhum nó do ide é `secret`; nada a redigir aqui.
- o `notes.txt`, fora dos commits, está na gravação só como o que o app
  projeta no painel de changes, nunca na superfície.

### de novo, com o `code` no IR (task-97)

a task-97 pôs no vocabulário do IR o componente que faltava ao diff: o
`code`, desenhado pelo `CodeBlock` do comps, uma linha por caixa de linha
do passo mono, com o tipo de cada linha (contexto, adicionada, removida,
cabeçalho) pelos tokens do tema, e rolagem dentro quando o `size.max` o
limita. o ide passou a projetar o tipo de cada linha ao lado dela
(`diff:kinds`), e a revisão das regras mostra o diff num `code` limitado
pelo mesmo token de antes (`dropdown_max_h`) no lugar da tabela. o
critério é o de cima, escrito antes da primeira corrida; nada nele mudou.
o laboratório conta as linhas à vista pela geometria do `code` (a área
das linhas sobre a altura de uma linha) em vez da tabela, e diz dos dois
lados quantas das linhas à vista a largura corta, um dado a mais, lido do
mesmo jeito nos dois.

rodada pelo laboratório sem janela, `ide-lab --arm review`, no build
`10f66729` da branch `task/97a-code-in-ir`, o mesmo repositório da
revisão, o mesmo roteiro, a mesma pessoa roteirizada, só as regras.

#### a gravação

`review.monster`, guardada fora do git (na scratchpad da sessão, com o
`report.json` e os PNGs), 138.262 bytes, 84 eventos, 19 frames, sha256
`598b83d2...`.

```text
$ plev-inspect review.monster
session   rec 0.1.0, built from 10f66729
span      +0.000s to +0.232s, 84 events, 19 frames
actors    human: 8 requests, 32 stages, 35 inputs
requests  8
  +0.022s  request 1  human  show_diff on stacks.commit[13ed9dbe...]  completed
  +0.046s  request 2  human  show_diff on stacks.commit[78f371df...]  completed
  +0.064s  request 3  human  show_diff on stacks.commit[847f583a...]  completed
  +0.112s  request 4  human  propose on ide_surface.propose           completed: surface = "proposed", nodes = 24
  +0.147s  request 5  human  show_diff on stacks.commit[13ed9dbe...]  completed
  +0.166s  request 6  human  show_diff on stacks.commit[78f371df...]  completed
  +0.184s  request 7  human  show_diff on stacks.commit[847f583a...]  completed
  +0.227s  request 8  human  close on ide_surface.close               completed: closed = "proposed"
refusals  0
```

na timeline, cada diff que chega traz agora o tipo das linhas junto
(`+0.061s changes diff:lines: added @@ -1,5 +1,6 @@, ...; diff:kinds:
added header, context, context, added, context, context, context;
diff:count: 0 -> 7`). a leitura do `code` depois do terceiro diff pela
superfície:

```text
$ plev-inspect review.monster --control proposed.diff.lines --at 205
control proposed.diff.lines at +0.205s
  node          proposed.diff.lines: region "lines", enabled
  state         kinds = [header, context, context, added, context, context, context, context,
                removed, added, context, context, header, added, added, added, added, added],
                lines = [src/session.rs: @@ -1,9 +1,10 @@,  pub struct Session {, ...]
  actions       none
  source        crates/surface/src/project.rs:104:23, in build 10f66729

$ plev-inspect review.monster --control proposed --at 232
control proposed at +0.232s
  node          not in the recorded state at that instant
```

#### o resultado lado a lado, pelo mesmo critério

| critério | superfície, task-96 | superfície, task-97 | painel fixo (nesta corrida) | fonte |
| --- | --- | --- | --- | --- |
| atos para ver os arquivos e o diff dos três | 5 cliques e 26 teclas; fechar é mais 1 | 5 cliques e 26 teclas; fechar é mais 1 | 3 cliques | receipts e inputs gravados |
| tempo, relógio do workbench (máquina, não pessoa) | 73 ms; a proposta em 0 ms, 24 nós | 94 ms; a proposta em 0 ms, 24 nós | 64 ms (47 ms na corrida da task-96) | `report.json` |
| arquivos dos três à vista de uma vez | 3 de 3 | 3 de 3 | 1 de 3 no máximo | ligações e `diff:lines` |
| linhas do diff à vista sem rolar | 5 de 14, 5 de 7, 5 de 18 (tabela, linhas de 48 px) | 14 de 14, 7 de 7, 14 de 18 (`code`, linhas de 19,6 px, limitado por `dropdown_max_h`, 320 px; o resto rola dentro) | 14 de 14, 7 de 7, 18 de 18 | ligações e a geometria de cada vista |
| das linhas à vista, cortadas na largura (dado novo) | não medido | 0, 0 e 3 | 0, 0 e 4 | a largura de cada vista, medida pelo `TextMeasurer` |
| o tipo de cada linha | sem cor | a lavagem do token do tipo atrás da linha (verde, vermelha, cinza no cabeçalho) e a marca `+` ou `-` no texto | a linha tingida, a marca colorida | o frame e `diff:kinds` |
| número de linha | não | não | sim, antigo e novo | o frame |
| os três commits à vista | 3 cartões inteiros | 3 cartões inteiros | 3 cartões, a mensagem cortada | ligações |
| cada ato no dispatcher, com o receipt da pessoa | os 5 de `human`, `completed` | os 5 de `human`, `completed`; o `show_diff` chega como a capacidade do app | os 3 de `human`, `completed` | summary do `plev-inspect` |
| nada da superfície depois de fechar | 0 nós, 0 ligações, 0 superfícies | 0 nós, 0 ligações, 0 superfícies; no estado gravado de +0.114 s (a proposta) até antes do close (+0.227 s); o último frame não liga nada dela | não se aplica | `report.json` e a sessão |

o que isso diz, pelo critério escrito antes: o diff deixou de ser o
ponto fraco nos dois primeiros commits e ficou perto no terceiro. a
superfície mostra agora 14 de 14 e 7 de 7 linhas, igual ao painel fixo,
com a cor pelo tipo de cada linha; no terceiro commit, 18 linhas, mostra
14 sem rolar contra 18 do painel fixo, e as outras 4 rolam dentro do
bloco. e o número de linha continua só no painel fixo. nos arquivos ela
segue melhor (3 contra no máximo 1) e nos atos segue pior (5 cliques e 26
teclas contra 3 cliques). então, na tarefa inteira ("ver os arquivos e o
diff dos três"), a superfície ainda não é melhor que o painel fixo: pelo
método, a demo 5 continua caindo no diff, agora por 4 linhas de um dos
três commits e pelo número de linha, e continua de pé nos arquivos. os
atos fora do dispatcher e o rastro depois de fechar, as outras duas coisas
que a derrubam, não aconteceram.

onde ela perde, e por quê:

- as 4 linhas do terceiro diff: o `code` é limitado pelo único token de
  altura do vocabulário (`SizeToken::DropdownMaxH`, 320 px no hoff), que
  dá 14 linhas de 19,6 px abaixo da legenda; o painel fixo usa a altura
  da janela. a superfície se desenha pelo conteúdo, então um bloco não
  sabe quanto sobra da coluna; sem o limite, um diff de 200 linhas
  passaria da janela sem rolar. fechar isso pede um token de altura para
  um bloco de leitura (tema, decisão de design do brenner) ou um layout em
  que um nó toma o que sobra do retângulo da superfície (decisão do
  orquestrador).
- o número de linha: a projeção leva a marca de cada linha, não os
  números antigo e novo (`crates/ide/src/workbench/project.rs`, `diff`).
- os atos: a intenção digitada e o propor, como na primeira corrida.

o que isso não cobre, dito como na primeira: só as regras; a pessoa
roteirizada, então o tempo é de máquina (e varia de corrida a corrida: o
painel fixo levou 47 ms na primeira e 64 ms nesta); diffs pequenos; a
demo com janela pelo brenner segue aberta. com a superfície aberta, o
painel fixo, mais estreito, continua mostrando o mesmo diff à esquerda,
então a pessoa vê o diff duas vezes; o critério compara os dois braços
separados, como antes.

#### privacidade da gravação

a mesma da primeira: 9 `pointer_down` com o controle sob ele e a
posição, 26 `text` sem alvo e sem conteúdo; a intenção só como o estado
projetado da linha; nenhum nó `secret`; o `notes.txt` fora dos commits
nunca na superfície. o que entrou: `diff:kinds`, os nomes dos tipos das
linhas do diff que o app já mostra, no estado gravado ao lado de
`diff:lines`.

### de novo, com a linha embaixo (task-96, fase b)

a validação da task-97 viu nas imagens dois defeitos que o critério não
conta: o cartão do diff na superfície tinha como título o id do commit
(`stacks.commit[847f583a…`), e a coluna da linha de intenção tirava 368
px da janela mesmo sem superfície aberta, então o painel fixo desta demo
era medido com a coluna de stacks mais estreita. a fase b da task-96 pôs
a linha numa faixa ao longo de baixo da janela, na largura inteira, a
coluna só com superfície aberta, e o cartão do diff abrindo com "diff".
rodada pelo laboratório sem janela, `ide-lab --arm review`, no build
`e42bd571` da branch `task/96b-ide-surface-polish`, o mesmo repositório,
o mesmo roteiro, a mesma pessoa roteirizada, só as regras; o critério não
mudou.

`review.monster`, fora do git (na scratchpad, `demo5b/after/`, com o
`report.json` e os PNGs), 136.414 bytes, 84 eventos, 19 frames, sha256
`874bd206...`; os mesmos 8 requests de `human`, todos `completed`, 0
recusas (`plev-inspect`). o laboratório diz agora também quantas das
três mensagens o painel de stacks corta na largura do cartão, pela
geometria do painel (um dado a mais, lido só do lado fixo, onde a
mensagem é cortada; na superfície ela quebra dentro do cartão).

| critério | antes (task-97) | agora (fase b) | por quê |
| --- | --- | --- | --- |
| atos | 5 cliques e 26 teclas contra 3 cliques | igual | o roteiro não mudou |
| tempo (máquina) | superfície 94 ms, painel fixo 64 ms | superfície 70 ms, painel fixo 44 ms | varia de corrida a corrida: a base desta fase, antes da mudança, deu 71 e 43 ms |
| arquivos dos três de uma vez | 3 contra 1 no máximo | igual | |
| linhas do diff à vista sem rolar | superfície 14, 7 e 14 de 14, 7 e 18; painel fixo 14, 7 e 18 | igual | o `code` segue limitado por `dropdown_max_h`; o painel fixo perdeu 60 px de altura para a faixa, e as 18 linhas ainda cabem |
| das linhas à vista, cortadas na largura | superfície 0, 0 e 3; painel fixo 0, 0 e 4 | igual | o painel de diff guarda a largura dele; é a coluna de stacks que ganhou a largura |
| os três commits no painel fixo | 3 cartões, a mensagem cortada (3 de 3, medido pelo dado novo sobre o layout de antes) | 3 cartões, a mensagem inteira (0 de 3) | sem superfície o workspace tem a largura inteira da janela, e a coluna de stacks tem de volta os 368 px |
| o título do cartão do diff na superfície | o id do commit, `stacks.commit[847f583a…` | "diff" | o `target` é a palavra do dispatcher; a região abre com o rótulo dela |
| nada da superfície depois de fechar | 0 nós, 0 ligações, 0 superfícies | igual | |

o que isso diz, pelo critério escrito antes: nada do resultado da
task-97 mudou de lado. a superfície continua melhor nos arquivos e pior
nos atos e no diff (4 linhas do terceiro commit e o número de linha); o
painel fixo, medido agora com a largura que ele tinha antes da task-96,
mostra as três mensagens inteiras. com a superfície aberta nada mudou
para ela: a coluna é a mesma metade da janela, agora do topo até a
faixa. o botão "show diff" de cada commit na superfície passou a ler
`diff:target` como o app o guarda (antes, `proposed.diff.target:text`,
o texto que mostrava o id), porque a superfície não mostra mais esse
estado; o stale continua o mesmo (o teste da task-96 passa igual).

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

- `docs/mission/steps/ongoing/task-80-proof-qualitatively-different-software.md`:
  sonda, método, provado, demos 3 e 4, handoff.
- `docs/ADR/generative-runtime-results.md` (novo, proposed).
- `crates/ide/tests/rerun/inflight.rs` (novo): a fixture `InFlight` e o
  teste do trecho.
- `crates/ide/tests/rerun/lab.rs`: `answer_while` e `take` (o `drain` usa
  o mesmo), `fixture` público no módulo. `crates/ide/tests/rerun/main.rs`:
  o módulo novo.
- `crates/ide/tests/fixtures/sessions/bug84.monster` (43.916 bytes) e
  `bug84.marks`.
- `crates/showcase/tests/ephemeral.rs` (novo): a demo 4.

nenhum código de produção, nenhum `Cargo.toml`, nenhuma linha do
`Cargo.lock`, nenhum baseline.

### interfaces

nenhuma adicionada nem alterada fora dos testes. o `Fixture` do harness,
as marcas e o relatório estão como estavam.

### validação

com o ambiente do briefing (o `PLEV_CLAUDE` no stub) e
`CARGO_NET_OFFLINE=true` em todo comando cargo:

- `./script/gate`: verde (exit 0, 03:17 a 03:19), com a perna urnaui e o
  check wasm com `-D warnings`. workspace 2927 passam, 0 falham, 26
  ignorados (157 suítes); urnaui 85 passam, 0 falham, 1 ignorado (6
  suítes). total 3012 passam e 27 ignorados, contra 3010 e 27 na base: os
  dois testes novos. `df -h /`: 16 GB livres antes e depois.
- `cargo run --offline -p showcase --example regress -- check` (com o
  `main.rs` tocado): 176 baselines, 176 iguais, nenhum frame desenhado,
  nenhuma GPU aberta.
- `cargo test -p ide --test rerun inflight`: dez de dez em
  `task/80a-proof`; `cargo test -p showcase --test ephemeral`: dez de dez.
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
