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
| 3, inspectability | a fase a da task-64 achou um defeito real lendo a sessão do A2 da primeira rodada no instante do submit: o `changes:staged` observado "outside the scope observed", o que virou a task-89 (`ongoing/task-64-session-inspector-and-tests.md`, "saídas de aceite" e "fase a integrada"); a fase c transformou a sessão a2ppa em teste de regressão (`crates/ide/tests/rerun/lab.rs:165`) e provou à mão, fora do gate, que ele pega uma precondição quebrada de propósito (mesmo arquivo, "o teste gerado e a prova de que pega regressão") | um bug real do histórico, enviado como trecho, aberto no instante, com as perguntas da tese respondidas, e um teste no gate que pega o bug reintroduzido (a fase c provou com uma quebra provocada, e à mão) | este agente | não, com uma fixture nova só em teste (item c) |
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

fonte: `ongoing/task-64-session-inspector-and-tests.md`, "saídas de
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
