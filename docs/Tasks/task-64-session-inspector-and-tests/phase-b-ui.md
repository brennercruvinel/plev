---
id: task-64/phase-b-ui
title: "task-64 phase b: inspector UI"
project: plev
type: section
parent: task-64
status: reference
date: 2026-09-25
date_basis: text
recorded: 2026-09-23
topics: [recording, rendering, mobile]
relations: [adr-0046, task-65]
evidence: [6de6d9b, a3ddb21, 26c8a86, 2d491d3, 48aca6b]
source: docs/log/steps/task-64-session-inspector-and-tests.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# phase b: inspector UI, plev-inspector window

This phase built the `plev-inspector` window, in the `apps/inspector` crate, with the time strip and the scrub, the recorded frame with each control's outline, and the control's panel at the chosen instant. The probe, read against `main` with the two recorded sessions, showed that the frame comes out at the right size without `rec` recording size or scale, and the tests prove that the window creates no request and sends nothing out. Opening it on a real session stayed in this page's script, and the orchestrator ran it at integration, together with the README and `arc.toml`.

## fase b: a sonda (2026-09-25)

lida contra a main `702f2827`, com as duas sessões gravadas do repositório
(`apps/idestudio/tests/fixtures/sessions/{a2ppa,bug84}.monster`).

### (a) onde a UI mora

um crate novo, `apps/inspector` (lib e o binário `plev-inspector`), não
um segundo binário no `inspect`:

- o `inspect` é o leitor, e um teste fixa as arestas dele em `rec`, `sem`,
  `serde` e `serde_json` (`rust/replay/tests/queries/readonly.rs:123`),
  como o ADR diz ([0046-the-inspector-reads-a-session-never-runs-it.md](../../adr/Recording/0046-the-inspector-reads-a-session-never-runs-it/0046-the-inspector-reads-a-session-never-runs-it.md):39-40).
  a UI precisa de `comps` e da janela; num binário atrás de feature, essas
  arestas entrariam no manifesto do leitor.
- a task já recomendava crate de app próprio, porque abre arquivo e vai
  crescer (arquivo da task, linha 44).
- arestas: `inspect`, `rec`, `sem`, `engine`, `comps`, todas do workspace.
  a janela vem pelo `engine`, que reexporta o `winit`
  (`rust/engine/src/lib.rs:64`), e o `WindowHost::open` abre o device
  sem executor próprio (`rust/engine/src/runtime/window/host.rs:170`). nenhum
  pacote de fora, nem `log` nem `web-time`.

### (b) o frame do instante

- o que foi desenhado vem de `Session::scene_at(t)`
  (`rust/record/src/session.rs:115`): os nós rebaixados do frame que estava
  na tela em `t`, em pixels lógicos da janela, o mesmo espaço dos
  contornos (`Bound.rect`, `rust/record/src/bound.rs:43-47`;
  `Session::bindings_at`, `session.rs:130`; `Session::hit_at`,
  `session.rs:146`). a UI escala cada nó por um fator e um deslocamento
  (retângulo, texto pelo tamanho da fonte, path pelos vértices) e desenha
  por cima o contorno de cada ligação, no mesmo fator. o frame gravado não
  entra na tabela de ligações do compositor da UI: é imagem, e o controle
  sob o ponteiro sai do `hit_at` da gravação.
- tamanho e escala: a sessão não guarda nenhum dos dois. o
  `Recorder::frame` recebe só o compositor (`rust/record/src/recorder.rs:105`)
  e o compositor não tem tamanho (`rust/engine/src/drawing/compositor/mod.rs:45-54`).
  mas o frame sai certo sem eles: nas duas sessões, o primeiro nó de todo
  frame é o fundo da janela em (0, 0, 1280, 800), e a extensão de todos os
  nós e de todas as ligações é exatamente 1280 x 800 (a2ppa, 12 frames;
  bug84, 7 frames). a escala não entra no desenho: os nós são lógicos e a
  UI desenha na escala da própria janela. então a UI tira o tamanho do
  frame da extensão dos nós e das ligações, recorta o desenho nela, e o
  `rec` não muda. fica pendência: a extensão deixa de ser a janela quando
  um nó passa da borda, porque o IR v0 do monster não guarda o recorte de
  uma camada (`recorder.rs:96-99`); uma lista rolada com linhas fora da
  tela alargaria o frame, e aí gravar o tamanho da janela resolve.
- a imagem de fluxo sem pixels: o arquivo grava toda imagem sem pixels
  (`rust/record/src/payload.rs:44-46`) e a lê de volta só com o tamanho
  (`payload.rs:16-18`); no rebaixamento ela vira um path vazio
  (`rust/record/src/visuals/reader.rs:77`) e o monster não desenha um nó de imagem
  cujo asset não é imagem (`rust/motion/src/mappings/lower.rs:106-116`). a UI
  não recebe nenhum nó de imagem e nenhum pixel, e o teste de privacidade
  prova isso em todo frame. mas ela também não sabe onde a imagem estava:
  desenhar a caixa no lugar pede do `rec` uma leitura dos nós de imagem sem
  pixels do frame, fora da permissão desta rodada (o `rec` só para tamanho
  e escala). vira pendência; nenhuma das duas sessões tem imagem.

### (c) a timeline e o scrub

- o instante é um `t` no relógio do recorder, entre `Inspector::start` e
  `Inspector::end` (`rust/replay/src/index.rs:111-119`). o scrub é o
  `Slider` do comps de 0 até a duração em ms; as setas andam de evento em
  evento.
- o frame em `t` usa o mesmo seek da fase a: `Screen::locate`
  (`rust/record/src/visuals/reader.rs:142`) acha o chunk por busca binária e o frame
  dentro dele, e o player do monster avalia aquele tempo
  (`rust/motion/src/play/mod.rs:148`). a UI guarda a cena pelo tempo do
  frame (`Session::frame_at`, `session.rs:120`), então um scrub que não
  troca de frame não decodifica nada.
- as faixas saem de `Inspector::timeline` (`rust/replay/src/timeline.rs:100`):
  uma faixa por ator (a pessoa inclusive) e uma do estado do app
  (snapshot e changes), cada evento uma marca no seu `at`, com a cor do
  tipo; as contagens por tipo e por ator são as do mesmo modelo.

### (d) quem responde cada pergunta do painel

- `Inspector::control(id, t)` (`rust/replay/src/control.rs:33`): o nó
  no estado gravado em `t` (role, label, estado, ações, o `source` com o
  sha do build) e os requests que tocaram o id até `t`, cada um com a
  cadeia causal como a janela até `t` conta.
- o texto é o do modelo: `inspect::text::control`
  (`rust/replay/src/prose/mod.rs:191`), que escreve uma linha por
  pergunta da tese para cada request com `text::why`
  (`rust/replay/src/prose/chain.rs:18`): control, state, initiated by,
  action, observed, precondition, before, stages, verdict, what changed,
  acted, result.
- o que veio depois de `t`: a mesma `control(id, end)`, e o painel mostra
  os requests dela que não estão inteiros na resposta em `t` (um request
  novo, ou um que em `t` ainda estava aberto). é o que o critério 1 pede:
  o botão só aparece no frame antes do commit (a2ppa: em +110.688s o form
  fecha junto com o `completed`), e o sha só existe depois.
- a lista de requests ao lado sai de `Inspector::summary`
  (`rust/replay/src/summary.rs:51`).

### (e) a prova de que a UI não cria request

- as fontes do crate não nomeiam `Dispatcher`, `Operable`, `ReceiptSink`,
  `Execution`, `Recorder`, `Request::` nem `Request {`, nem `.dispatch(`,
  `.execute(`, `.confirm(`, `.decline(`, nem socket ou processo; e o
  manifesto depende exatamente de `comps`, `engine`, `inspect`, `rec`,
  `sem` (o mesmo teste do `inspect`, `readonly.rs:88-124`).
- um teste passa por todo handler da view (ponteiro numa grade da janela
  inteira, clique em cada ligação de cada frame, cada request da lista,
  cada tecla, cada posição do scrub) e acha a sessão igual à aberta do
  arquivo e os bytes do arquivo iguais.
- o frame gravado não liga nada no compositor da UI
  (`Compositor::visible_bindings`, `rust/engine/src/drawing/compositor/binding.rs:189`):
  nenhum id gravado vira controle vivo.

## fase b: o roteiro para abrir a UI numa sessão de verdade

não rodado aqui (esta rodada não abre janela). na raiz do repositório:

1. gravar uma sessão de verdade no ide: `PLEV_RECORD=1 cargo run -p idestudio
   --bin ide <um repo git>`. usar o app (ou deixar um agente usar pela
   porta, com `PLEV_AGENT=1`), salvar com cmd-shift-s, ou fechar a janela.
   o log diz onde o arquivo foi: `$TMPDIR/plev/sessions/`.
2. abrir: `cargo run -p inspector --bin plev-inspector --
   $TMPDIR/plev/sessions/<arquivo>.monster`. sem sessão nova à mão, a do
   cenário do commit serve: `cargo run -p inspector --bin plev-inspector
   -- apps/idestudio/tests/fixtures/sessions/a2ppa.monster`.
3. o que olhar:
   - o cabeçalho diz o instante, a duração e o frame na tela; a lista à
     esquerda tem cada request com como terminou; um clique leva ao
     instante dele e escolhe o controle alvo.
   - a faixa embaixo do frame: uma linha por ator e uma do estado do app,
     cada evento uma marca (vermelha: recusa; verde: concluído ou
     observação); arrastar o scrub, ou clicar nas faixas, move o
     instante; setas esquerda e direita andam de evento em evento, home e
     end vão ao começo e ao fim.
   - o frame gravado com o contorno de cada controle; o ponteiro em cima
     mostra o `SemId`; um clique escolhe o controle e o painel à direita
     mostra as respostas do modelo naquele instante, e depois o que veio
     após ele. a roda rola o painel; esc solta a escolha.
   - na a2ppa: scrub até uns +105s (antes do commit, o form aberto),
     clique em "Commit": o painel mostra a observação do agente, o request
     7, a recusa stale com `changes:staged: removed src/session.rs`, o
     `acted` da pessoa, e depois o request 9 aceito com o sha `bf680c8…`.
4. o que conferir e anotar: se o frame bate com o que estava na tela
   (tamanho e posição dos controles), se algum texto do painel fica
   ilegível na largura da janela, e se a janela estreita (arrastar a
   borda até largura de celular) empilha frame, faixa e painel sem
   sobrepor.

## fase b: handoff (2026-09-25)

```text
Goal: task-64 fase b, a UI do inspector de sessão
Status: pronta para revisão, com um bloqueio de gate que é do orquestrador
        (branch task/64b-inspector-ui, base main 702f2827)
```

### commits

- `6de6d9b` task-64: phase b probe, where the ui lives and what it reads
- `a3ddb21` task-64: the inspector crate and its window: strip and scrub,
  the frame with its controls, the panel
- `26c8a86` task-64: the view's model without a window: the instant, the
  pick, the panel, the strip
- `2d491d3` task-64: criterion 1, the commit scenario in the inspector
- `48aca6b` task-64: narrow and wide viewports, the proof the window runs
  nothing, what it never shows
- este: task-64: phase b, the window in the ADR, the script for opening it
  and the handoff

o código da UI entrou num commit só: a view, a faixa, o frame e o painel
compilam juntos (a view guarda os três), e separar pedia versões
intermediárias escritas só para o histórico.

### a sonda

na seção "fase b: a sonda" acima. em resumo: crate próprio; o frame sai
certo sem tamanho e escala na sessão (a extensão dos nós e ligações é a
janela, 1280 x 800, nos 19 frames das duas sessões), então o `rec` não
mudou; a caixa no lugar da imagem de fluxo ficou pendente (abaixo).

### a UI

`apps/inspector` (lib e `plev-inspector <session.monster>`):

- `InspectorView` guarda a sessão e nada mais. os handlers
  (`pointer_move`, `pointer_down`, `pointer_up`, `scroll`, `key`, `seek`,
  `pick`, `go_to_request`) mexem só no instante, no ponteiro, na escolha
  e na rolagem, e dizem se um frame é devido; `render` desenha em
  qualquer `Compositor`.
- a moldura é o `AppShell`: cabeçalho com o nome do arquivo, o instante,
  a duração e o frame na tela; a lista de requests do `summary` na barra
  (gaveta no celular).
- a faixa (`Strip`): uma linha por ator e uma do estado do app, a marca
  de cada evento do `timeline` no tom do seu tipo (tokens do tema), um
  chip por tipo com a contagem, o cursor do instante, e o `Slider` do
  comps como scrub alinhado com as marcas.
- o frame: `scene_at(t)` escalado para caber (`Fit`: retângulos, texto
  pela tipografia, path pelos vértices), recortado na própria extensão;
  por cima, o contorno de cada ligação, o do controle sob o ponteiro
  mais forte, o anel de foco no escolhido, e o `SemId` sob o ponteiro no
  `Tooltip`. a cena é lida uma vez por tempo de frame.
- o painel (`Panel`): `control(id, t)` como o `text::control` escreve,
  depois os requests que tocaram o controle depois de `t` (ou estavam
  abertos em `t`), do `control(id, end)`, menos o que `t` já contou
  inteiro; no `CodeBlock` do comps, com as linhas longas quebradas na
  largura do painel sem perder caractere.
- nenhuma cor nova: tudo vem do tema ou da própria gravação.

### os testes (`apps/inspector/tests/ui`, 15)

- `accept::the_commit_scenario_in_the_inspector`: **critério 1**. abre a
  a2ppa pelo caminho, arrasta o scrub até +105s (antes do commit, depois
  da última observação), clica no centro do botão `commit.submit` no
  frame; o painel traz, nesta ordem, o controle, o request 7, quem
  iniciou, a observação, o `changes:staged` que o agente viu, a recusa
  stale, `changes:staged: removed src/session.rs`, o `acted`, e depois o
  request 9 aceito e o sha; rolando o painel, a recusa e o sha aparecem
  desenhados, nessa ordem.
- `model::*` (7): o instante pelo scrub, pelas faixas e por evento; o
  frame e as ligações iguais aos da sessão em cada tempo de frame; o
  `SemId` sob o ponteiro igual ao `hit_at` e desenhado; o painel igual a
  `Inspector::control` e ao `text::control`, respondido de novo quando o
  instante muda, solto por esc e por clique fora de controle; cada
  request da lista vai ao seu instante e ao seu alvo; cada evento na
  faixa do seu ator; a quebra de linha mantém todos os caracteres.
- `viewport::*` (3): **testes esperados, estreito e largo**. 390 x 844
  (gaveta, uma coluna) e 1440 x 900 (barra cheia, painel ao lado): cada
  parte dentro do conteúdo, nenhuma sobre outra, o frame no aspecto
  gravado, nada da tela passando da janela, as linhas cabendo no painel,
  e o clique no botão ainda escolhe ele; e um resize de largo para
  estreito.
- `readonly::*` (2): **critério 4 para a UI** e a prova (e). as fontes não
  nomeiam nada que rode, grave ou monte um request, nem socket, processo,
  porta ou ponte de accessibility; o manifesto depende de exatamente
  comps, engine, inspect, rec, sem, sem dev-dependencies; e todo handler
  (clique em cada ligação em cada instante, uma grade de cliques e rolagens
  na janela inteira, cada tecla, cada request da lista) deixa a sessão
  igual à aberta do arquivo e os bytes do arquivo iguais, sem nenhum id
  gravado ligado no compositor da janela. nas duas sessões.
- `privacy::*` (2): nenhuma imagem desenhada em nenhum frame das duas
  sessões, nem numa sessão lida no processo que gravou (onde o replay
  ainda tem a imagem, e o teste confere que tem); e um segredo (um login
  com a senha como parâmetro, eco no resultado, e um submit recusado
  sobre ela) nunca aparece em nenhum texto que a janela desenha ou que o
  painel lê, em nenhum instante, com o painel respondendo sobre a senha
  (não é um teste vazio). a preferência de accessibility: a janela não
  manda nada a lugar nenhum (o teste de fontes acima).

### arestas e o `Cargo.lock`

arestas do `inspector`: `comps`, `engine`, `inspect`, `rec`, `sem`, todas
do workspace; o `winit` vem pelo `engine` (`engine::winit`), sem aresta
própria; nada em dev. o diff do `Cargo.lock` é só o bloco novo:

```text
+[[package]]
+name = "inspector"
+version = "0.1.0"
+dependencies = [
+ "comps",
+ "engine",
+ "inspect",
+ "rec",
+ "sem",
+]
```

e a linha `"apps/inspector",` em `members` da raiz.

### validação

- disco antes do gate: 11 GB livres.
- as pernas do `./tool/tasks/fullcheck.sh`, na ordem dele, no target dir do agente:
  `cargo test --workspace` 3057 passaram, 0 falharam, 27 ignorados;
  clippy `-D warnings` limpo; `fmt --check` limpo; wasm do showcase com
  `-D warnings` limpo; perna urnaui (a `../urna` existe ao lado do
  worktree): 85 passaram, 0 falharam, 1 ignorado, clippy, fmt e wasm
  limpos.
- **o bloqueio:** o teste `every_crate_is_named_in_arc_toml_and_readme`
  (`tool/tests/test_repoaudit.rs:73`) falha com "README.md does
  not mention these workspace crates: ["inspector"]". todo crate de
  `members` precisa aparecer no README e no `arc.toml`, e os dois são
  proibidos nesta rodada. a contagem acima rodou com só esse teste de
  fora (`--skip`); com as duas linhas abaixo, ele passa.
- regress: `tool/bench/regress/src/main.rs` tocado, `check`:
  176 baselines, 176 iguais, 0 mudados.
- contagem de testes: nenhum teste fora de `apps/inspector` mudou; a
  branch soma 15 aos da main.

### o que o orquestrador põe na integração (README e arc)

README, lista de crates (linha 125): `inspector` depois de `inspect`; e
na seção de comandos, depois da linha do `plev-inspect`:

```text
cargo run -p inspector --bin plev-inspector -- <session.monster>   # the inspector's window: the timeline and the scrub, the recorded frame with its controls, the thesis answers about the control clicked; it never runs anything
```

`docs/ATLAS.toml`, depois de `[crates.inspect]`:

```text
[crates.inspector]
path = "apps/inspector"
role = "the session inspector's window (task-64 phase b, adr the-inspector-reads-a-session-never-runs-it): InspectorView owns a rec::Session and nothing else, shows the inspect model (the timeline as lanes by actor with the comps Slider as the scrub; the frame on screen at the instant, Session::scene_at fit into its rect, with the outline of every control from Session::bindings_at and the SemId under the pointer from Session::hit_at, never bound in its own compositor; the panel of a control, Inspector::control at the instant then what came after it, as inspect::text writes it); handlers and render run without a window, the plev-inspector binary (plev-inspector <session.monster>) is the only part that opens one; no dispatcher, no request, no socket, no process"
```

### linha para o índice de ADRs

o ADR já está no índice; a linha dele ganha a fase b:

```text
| 2026-09-23 | [the-inspector-reads-a-session-never-runs-it](the-inspector-reads-a-session-never-runs-it.md) | accepted | o crate `inspect` lê uma sessão gravada e responde as perguntas da tese sobre qualquer request ou controle, sem executar nada (nenhum app, nada que despacha); `why` acha a observação do mesmo ator na revisão do request, lê o estado depois de cada evento e põe cada mudança no que o request lê nos receipts que rodaram entre o estado anterior e a mudança, por correlação no tempo e nunca por escrita declarada (nenhum: de fora do app; dois ou mais: ambíguo, todos mostrados); CLI `plev-inspect` com resumo, timeline, why, control e json (task-64 fase a); a janela `plev-inspector` (crate `inspector`) mostra esse modelo sem recalcular nada: timeline e scrub, o frame gravado com os contornos (uma imagem, nunca ligada no compositor dela), e o painel do controle no instante e depois dele; nenhum caminho dela cria request (task-64 fase b) |
```

### pendências

- o tamanho da janela e a escala não estão na sessão: o frame usa a
  extensão dos nós e das ligações, exata nas sessões de hoje; um nó
  que passe da borda da janela (o recorte de camada que o IR v0 não
  guarda) alargaria o frame. gravar o tamanho da janela no `rec` resolve.
- a caixa no lugar de uma imagem de fluxo: a janela não desenha pixel
  nenhum, mas também não mostra onde a imagem estava, porque o replay a
  rebaixa a nada e o `rec` não diz onde ela estava. pede uma leitura no
  `rec` dos nós de imagem sem pixels de um frame (fora da permissão desta
  rodada).
- os controles da própria janela não projetam nada para accessibility e
  não passam por dispatcher (de propósito, o inspector nunca cria
  request; está no ADR); uma projeção só de leitura, sem ações, fica para
  depois.
- a faixa desenha as marcas com as receitas e os tokens do comps, mas é
  composição da janela; se outra tela precisar de uma linha do tempo com
  marcas, ela vira widget no comps, com a matriz e a página no showcase.
- fora do escopo desta rodada, como o briefing diz: marcar valores e
  escrever `.marks` pela UI, exportar trecho pela UI, o link para o
  código (task-65), a revisão servida no estágio terminal, web e mobile
  para a UI.
