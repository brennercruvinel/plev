---
id: task-69/phase-4c
title: "task-69, phase 4c: recording and the recorded demo"
project: plev
type: section
status: reference
parent: task-69
date: 2026-09-24
date_basis: text
topics: [recording, agents]
relations: [adr-0037, task-63]
source: docs/log/steps/task-69-ephemeral-generative-surfaces.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
evidence: [29efeb93, fb57e228, 8c0652e7, 94942d80, 82db3ddf]
---

# phase 4c: the showcase records the session and the demo is kept as a recording

Records phase 4c: the showcase now records its own session, as the ide does, with `PLEV_RECORD=1`, and the video editor demo, driven through the agent gate, is recorded in two `.monster` fixture files. The result is that `plev-inspect` reads the demo's ten requests with the origin of each act, and three findings about generated surfaces were recorded in ADR 0037, with no fix.


branch `task/69h-recorded-demo`, sobre a `main` em `a9edcbaa`.

## sonda da fase 4c (antes do primeiro tipo, 2026-09-24)

(a) o showcase não grava. por leitura: `grep -rn "rec::\|Recorder\|Recording\|PLEV_RECORD"` em `apps/uigallery/{src,Cargo.toml,examples}` não acha nada (o ide tem os três em `record.rs`, `recording.rs` e `lib.rs`); `apps/uigallery/Cargo.toml` não depende de `rec` (chega a ele só por `ide`). por binário: `strings target/debug/showcase | grep -c PLEV_RECORD` dá 0 (o `ide`, 1) e "plev/sessions" não aparece. por um teste descartável: `PLEV_RECORD=1 showcase --render Silences --semantics probe-a.json` escreveu o json (17 nós, revisão `d3c64aa094fd9c52`) e `$TMPDIR/plev/sessions/` ficou sem nenhum `showcase-*` antes e depois.

os cinco pontos do shell do ide, por linha, e o ponto equivalente no shell do showcase (linhas da base):

| o que | onde o ide grava | onde o showcase tinha o ponto e não gravava |
| --- | --- | --- |
| sink no dispatcher único | `apps/idestudio/src/main.rs:530` `recording::start` -> `recording.rs:16` `attach` -> `record.rs:129` `bench.add_sink(rec.sink())` | `apps/uigallery/src/views/dispatcher/mod.rs:316` `add_sink` (o dispatcher do todo, emprestado à Surface e aos silêncios); só a porta chamava (`dispatch/agent.rs:56`) |
| a porta: quem observou o quê, e o estado depois de um ato | `main.rs:188` `ide::agent::serve_recorded(port, bench, compositor, recording)` -> `agent.rs:57` (`Served.recorder`, `Host::observed` em 112, o estado em 69) | `app.rs:179` `serve_agent` -> `view.serve_agent(port, compositor)` -> `dispatch/agent.rs:21` (`Served { view, compositor }`, sem `observed`, nada gravado) |
| o estado depois de clique ou tecla que mudou algo, depois de um resultado do git, antes do frame | `main.rs:258` (git), `371` e `390` (`changed()` depois do clique), `keys.rs` (tecla), `record.rs:143` (antes do frame) | `app.rs:285` (git: só `invalidate`), `410` e `421` (cliques: só `invalidate`), `341` a `363` (teclas: só `invalidate`) |
| cada frame com as ligações | `main.rs:441` `recording.frame(&bench, &compositor)` depois do `render_frame` | `app.rs:460` `render_frame` e a árvore da acessibilidade em `470`; nada gravado |
| os inputs sem caractere | `main.rs:369` (esquerdo), `388` (direito), `keys.rs:36` (`text`), `keys.rs:50` (`key`) | `app.rs:393` (esquerdo), `415` (direito), `334` (teclado): nada gravado |
| salvar: atalho, fechar, pânico | `keys.rs:82` (`cmd-shift-s`), `main.rs:463` (`exiting`), `recording.rs:17` (`catch_panics`) | `app.rs:327` `CloseRequested => event_loop.exit()` sem `exiting`; sem atalho; sem hook |

o relógio: o ide grava em `bench.now()`, o relógio dos receipts. no showcase os receipts levam o relógio do dispatcher do todo (`apps/app/semantics.rs:259` e `279`, `clock.elapsed()`), lido só dentro de `with_dispatcher` (`pub(crate)`, `&mut self`); a `ShowcaseView` não tinha `now()`.

(b) o showcase tem um modo servido sem janela: `ShowcaseView::serve_agent(&mut AgentPort, &Compositor)` sobre um `AgentPort::open_with` num diretório, a forma dos testes de `view/dispatch/agent.rs` e de `silences/tests/agent.rs` (este thread é o shell: serve, dá o tick, desenha num `Compositor` sem GPU; um thread é o agente pelo `wire::Client`). o roteiro rodou assim, por `apps/uigallery/examples/videos.rs`, sem janela nenhuma e sem `PLEV_BACKGROUND`: 3,3 s, 17 observações, 10 requests, 101 frames desenhados. três coisas apareceram na corrida: as regras recusam a frase da tese ("then" e "review" não são palavras do app, o que a 4b já tinha dito), então pelas regras a revisão é "the list with the video and the counts" e a frase da tese fica para o Claude; em 1200 por 800 uma linha da revisão das regras (thumbnail, dois textos, dois botões) é mais larga que a metade e o botão de rejeitar fica em x = 1232..1341, fora da janela, onde um clique não chega (pelo id o agente chega), então a demo desenha em 1600 por 1000 (`--width`, `--height`); e a rolagem da página não resolve isso (é horizontal), o exemplo diz o retângulo no erro. com janela de verdade não foi preciso; se um dia for, `PLEV_BACKGROUND=1 PLEV_AGENT=1 PLEV_RECORD=1 cargo run -p uigallery -- Silences` abre a janela por baixo e o mesmo roteiro cabe em `plev observe`/`act` contra `--app showcase`.

(c) o tamanho: a sessão da demo (3 s, 100 frames, 73 eventos) tem 186.837 bytes: eventos 81.078 (4 keyframes de até 51 nós, 10 passos de mudança, 40 estágios, 17 observações, 2 inputs), índice 6.812, chunks 67.984 (100 frames em 12 chunks, uns 680 B por frame), assets 21.085 (134: estilos de texto e glifos; 3 imagens só com o tamanho), ligações 9.600. o `rec` guarda de imagem `Asset::Live(LoweredAsset::Image(handle))`, o handle do atlas, e persiste só largura e altura (`Persisted::images_without_pixels = 3`): os pixels de um frame do vídeo nunca entram no arquivo, por construção (a retenção de pixels ficou fora da task-63 fase d, sem task). então 30 s custam o que os frames desenhados e os eventos custam, não o tempo: render on demand, a página parada não desenha; mil frames seriam uns 700 KB. a fixture cabe no repositório (187 KB, limite de 2 MB) e fica sem os pixels dos frames: leva a cena de cada frame (retângulos, textos, as imagens com tamanho) e as ligações; `Session::scene_at` lê a cena sem nó de imagem, `to_bytes` conta as três imagens sem pixels.

## handoff da fase 4c (2026-09-24)

```text
Goal: task-69 fase 4c: o showcase grava a própria sessão como o ide
      (PLEV_RECORD=1, debug) e a demo do editor de vídeo, dirigida pela
      porta do agente, fica gravada num .monster de fixture que o
      plev-inspect lê, com os frames
Status: pronto para validação do orquestrador
```

### commits

- 29efeb93 task-69h: the probe of phase 4c, before the recording
- fb57e228 task-69h: the showcase records its session as the ide does
- 8c0652e7 task-69h: the demo's script through the port, with no window
- 94942d80 task-69h: the demo recorded by the rules and by the Claude, and the test that reads it
- 82db3ddf task-69h: the demo example split under the file limit
- o commit deste texto: o adendo do ADR e o handoff

### a sonda

está na seção "sonda da fase 4c" acima: (a) o showcase não gravava
(leitura, binário, uma corrida descartável), com os cinco pontos do ide
por linha e o ponto equivalente do showcase; (b) o modo servido sem
janela existia (`ShowcaseView::serve_agent` sobre um `AgentPort`) e o
roteiro rodou por ele sem `PLEV_BACKGROUND`; (c) a sessão da demo pesa
187 KB, os frames custam uns 680 B cada e as imagens entram só com o
tamanho.

### a gravação (os cinco pontos)

`apps/uigallery/src/shell/window/record.rs` (novo): `ENV`, `enabled`,
`default_dir`, `SAVE_KEYS`, `is_save_key`, `Recording` (`from_env`,
`new`, `recorder`, `dir`, `attach`, `state`, `frame`, `input`,
`next_path`, `save`, `catch_panics`) e os ganchos do shell (`App::record`,
`record_state`, `record_key`, `record_pointer`, `record_frame`,
`save_session`). o relógio é o dos receipts: `ShowcaseView::now(&mut self)`
em `view/dispatch.rs`, o `now` do dispatcher do todo lido por
`with_dispatcher` (daí o `&mut`; um `TodoPage::now(&self)` seria uma linha
em `apps/app/semantics.rs`, fora da lista desta fase).

1. sink: `Recording::attach` põe `rec.sink()` no dispatcher único
   (`ShowcaseView::add_sink`) e grava o estado agora, o primeiro keyframe;
   `App::record` chama em `run()`, logo depois de `App::new()`, e instala o
   hook de pânico.
2. porta: `ShowcaseView::serve_agent_recorded(port, compositor,
   Option<&Recorder>)` em `view/dispatch/agent.rs`; `Served` ganhou
   `recorder` e `Host::observed` (ator, escopo e revisão, no relógio da
   galeria); depois de um drain que rodou um ato, o estado. `serve_agent`
   continua com a mesma assinatura, delegando com `None`; `App::serve_agent`
   passa `self.recording.as_ref().map(Recording::recorder)`.
3. estado: `App::changed()` (o estado quando grava, e o frame) onde havia
   `invalidate()` depois de algo que mudou: um resultado do git na tela
   (`UserEvent::Wake`), o leitor de tela (`UserEvent::Access`), Escape que
   fechou um overlay, tecla do editor, tecla nomeada, caractere, clique
   esquerdo e direito; e antes de cada frame (`Recording::frame` grava o
   estado primeiro). estado igual custa um diff e não guarda nada; página
   sem projeção (Button, Card) não guarda nada.
4. frame: `App::redraw` (o corpo de `RedrawRequested`, movido de `app.rs`
   para `app/pace.rs`) chama `record_frame()` depois de `render_frame` e da
   árvore da acessibilidade: `rec.frame(now, compositor)`, com as ligações.
5. input: `record_key(&Key)` antes do match do teclado (o atalho de salvar
   consome a tecla; qualquer outra entra como `key` ou `text` pelo tipo,
   nunca o caractere); `record_pointer(state, x, y)` antes de `handle_event`
   no botão esquerdo (press e release) e no direito (press): tipo, posição
   e o controle sob o ponteiro (`Compositor::hit`).

salvar: `exiting` (a janela fechada, ou Escape sem overlay) ->
`save_session("close")`; `cmd-shift-s` (`ctrl-shift-s` fora do mac) só
enquanto grava (a galeria não tem keymap: `is_save_key` confere `s` com
shift e o modificador primário); o hook de pânico do `rec`. arquivos
`showcase-<pid>-<n>.monster` em `$TMPDIR/plev/sessions` (0700), o caminho
no log (`run()` liga `warn,showcase=info` quando grava). sem a variável:
`from_env()` é `None`, nenhum diretório, nenhuma tecla, um `Option` por
frame e por input. `app.rs`: 595 linhas antes, 611 depois (o `redraw` foi
para `pace.rs`, 66 -> 120 com testes); `dispatch.rs` 363 -> 371;
`dispatch/agent.rs` +30 na produção. wasm e mobile: nada disso compila lá
(`record` e `rec` sob o cfg do desktop; o check wasm com `-D warnings`
verde no gate).

### o roteiro (o programa e cada passo)

`apps/uigallery/examples/videos/{main,shell,agent}.rs`:

```text
PLEV_RECORD=1 cargo run -p uigallery --example videos -- [--out f.monster]
    [--review rules|claude] [--compare rules|claude] [--name demo]
    [--width 1600] [--height 1000]
```

sem janela: este thread é o shell (`Shell`: serve a porta com
`serve_agent_recorded`, `tick` pelo tempo real, `render` num `Compositor`,
`Recording::frame` a cada frame; `click` faz a pessoa; `settle` no fim), o
outro é o agente (`wire::Client`, `hello demo`, todo ato `from_revision`
de uma observação da página inteira feita logo antes). a gravação com
`Limits { max_age: 600 s, max_bytes: 64 MiB }` (a corrida inteira, não os
20 s do shell), salva mesmo quando o roteiro falha no meio. os passos,
como ficaram na fixture pelas regras (requests 1 a 10):

1. observe com bounds;
2. `detect_silence` em `silences.detect` (request 1): o ffmpeg gera o
   vídeo se faltar e detecta (61 ms; os cortes viram 3 e 4); a página pede
   os frames do instante e dos cortes, e o agente observa
   `silences.status:busy` até ficar ocioso duas vezes seguidas;
3. `select` "rules" em `silences_surface.proposer` (2), `set_value` "the
   list with the video and the counts" em `silences_surface.intent` (3),
   `propose` (4: `surface = "proposed"`, 33 nós); os frames da superfície
   chegam (ocioso de novo);
4. `activate` em `proposed.silences.cut[4].apply_cut`, o segundo corte
   (o agente age por id; o cartão dele fica abaixo da dobra): request 5,
   gravado como `apply_cut on silences.cut[4]` (roteado por
   `Surface::route`);
5. a pessoa: `Poke::Click` em `proposed.silences.cut[3].reject_cut`, o
   primeiro corte; o shell rola a página com a roda se o botão não estiver
   inteiro na janela (não precisou em 1600 por 1000), grava `pointer_down`
   e `pointer_up` com o botão sob o ponteiro, e o clique vira request 6,
   `reject_cut on silences.cut[3]` por `human`;
6. `select` (7), `set_value` "compare the original with the cuts" (8),
   `propose` (9: o mesmo `proposed`, 27 nós, a comparação no lugar);
7. `close` em `silences_surface.close` (10: `closed = "proposed"`); a
   saída em vários frames.

3,3 s, 17 observações, 101 frames desenhados (100 guardados: um igual ao
anterior). com o Claude na revisão, o mesmo roteiro com a frase da tese:
127,5 s.

### a demo (arquivo, tamanho, o que tem dentro)

`apps/uigallery/tests/fixtures/sessions/silences-demo.monster`: 186.837 bytes
(eventos 81.078, índice 6.812, chunks 67.984, assets 21.085, ligações
9.600); 73 eventos (4 keyframes, 10 mudanças, 40 estágios, 17
observações, 2 inputs), 100 frames em 12 chunks, 134 assets, 3 imagens sem
pixels. `silences-demo-claude.monster`: 861.354 bytes (eventos 495.271,
chunks 255.517, assets 45.056, ligações 32.760, índice 32.472); 200
eventos, 226 frames em 63 chunks: o contador da espera muda a cada segundo
por 124 s, uma mudança e um frame por segundo.

`apps/uigallery/tests/demo.rs` abre os dois como o `plev-inspect`
(`rec::Session::open`) e confere por dentro: os 10 requests na ordem, com
ator, alvo, ação e `completed` inteiros, o `close` por último com `closed
= <superfície>`; as intenções como escritas; as observações só do agente,
a página inteira e o escopo `silences.status`; o clique da pessoa
(`pointer_down` e `pointer_up` com o botão da superfície sob eles, e
`hit_at` no mesmo ponto dá o mesmo id); no instante do apply, as ligações
da superfície na tela ao lado das da fixa (`bindings_at`) e a superfície
no estado (`state_at`), sumida no fim; os frames do vídeo no arquivo como
imagens sem pixels (`to_bytes().images_without_pixels >= 3`); e o `why`
do inspector sobre o apply (a observação da página inteira como base, a
`decision = "pending"` sem mover, `Completed`). na do Claude, mais:
`attempts = 2`, `seconds > 60`, `cost_usd` entre 0 e 1. nenhum teste do
gate roda o ffmpeg, o demo ou o claude.

### o que o plev-inspect disse

`plev-inspect apps/uigallery/tests/fixtures/sessions/silences-demo.monster`:

```text
session   rec 0.1.0, built from a9edcbaa
span      +0.000s to +2.951s, 73 events, 100 frames
actors    agent:demo: 17 observations, 9 requests, 36 stages
          human: 1 request, 4 stages, 2 inputs
requests  10
  +0.039s  request 1   agent:demo  detect_silence on silences.detect     completed: cuts = 2, seconds = 0.060720708
  +0.835s  request 2   agent:demo  select on silences_surface.proposer   completed
  +0.839s  request 3   agent:demo  set_value on silences_surface.intent  completed
  +0.845s  request 4   agent:demo  propose on silences_surface.propose   completed: surface = "proposed", nodes = 33
  +1.565s  request 5   agent:demo  apply_cut on silences.cut[4]          completed: decision = "apply"
  +1.594s  request 6   human       reject_cut on silences.cut[3]         completed: decision = "reject"
  +1.604s  request 7   agent:demo  select on silences_surface.proposer   completed
  +1.613s  request 8   agent:demo  set_value on silences_surface.intent  completed
  +1.621s  request 9   agent:demo  propose on silences_surface.propose   completed: surface = "proposed", nodes = 27
  +2.353s  request 10  agent:demo  close on silences_surface.close       completed: closed = "proposed"
refusals  0
```

`--why 5` (o apply pela superfície gerada):

```text
request 5 (origin 0): apply_cut on silences.cut[4] by agent:demo, at +1.565s
  control       silences.cut[4]: list_item "0:08.0 to 0:10.0", enabled
  state         start = 8.010884, end = 10.0078, duration = 1.996916, range = "0:08.0 to 0:10.0", decision = "pending", frame = "silences.video@9.009342"
  initiated by  agent:demo
  action        apply_cut, no parameter
  observed      +1.560s, the whole app, at revision 19e5c9c8117320ad
                (the last observation by agent:demo at the revision the request carries)
  precondition  what the action reads, as agent:demo saw it:
                  silences.cut[4]:decision = "pending"
  before        from the observation (+1.560s) to the request (+1.565s):
                  silences.cut[4]:decision did not move
  stages        +1.565s requested
                +1.565s accepted
                +1.565s executing
                +1.565s completed: decision = "apply"
  verdict       accepted at +1.565s
  result        completed at +1.565s: decision = "apply"
```

`--control proposed.silences.cut[4].apply_cut --at 1600` (o botão da
superfície, com ela aberta) e `--control silences.cut[4] --at 1600`:

```text
control proposed.silences.cut[4].apply_cut at +1.600s
  node          proposed.silences.cut[4].apply_cut: button "apply cut", disabled
  state         none of its own
  actions       activate (reads proposed.silences.cut[4].decision:text), disabled
  source        rust/screen/src/project.rs:104:23, in build a9edcbaa
  requests      0 touched it up to +1.600s (their target, or a state they read)

control silences.cut[4] at +1.600s
  node          silences.cut[4]: list_item "0:08.0 to 0:10.0", enabled
  ...
  requests      1 touched it up to +1.600s (their target, or a state they read)
  request 5 (origin 0): apply_cut on silences.cut[4] by agent:demo, at +1.565s
```

`--why 6` (o clique da pessoa na superfície), o trecho que importa:

```text
request 6 (origin 0): reject_cut on silences.cut[3] by human, at +1.615s
  observed      not recorded: no observation by human before the request
                (a person acts on the frame on screen; the recorder keeps only the observations the app reports)
                the recorded state never had revision 3e714d915fb24cbe in this window
  before        the recording does not say when the state was at that revision, so not what moved
```

os achados, registrados no ADR e não corrigidos:

1. um ato numa superfície gerada chega ao dispatcher como a capacidade
   roteada (`Surface::route`): o receipt diz `apply_cut on
   silences.cut[4]`, nunca `proposed.silences.cut[4].apply_cut`. o `--why`
   explica sobre o nó do app, o `--control` do botão da superfície diz "0
   requests touched it" e o do nó do app diz request 5. a sessão não
   guarda que o ato veio pela superfície gerada; só o input da pessoa
   nomeia o botão dela.
2. o clique da pessoa na superfície (e na tela fixa) se apoia no snapshot
   só do app (`Surface::shown`, `SilencesPage::snapshot()`), com uma
   revisão que a gravação nunca tem, porque o gravador guarda a projeção
   da página inteira (`ShowcaseView::semantics()`, a `projection()`): duas
   projeções, duas revisões, e o inspector não coloca a observação da
   pessoa.
3. a revisão do Claude nomeou os controles com os ids do app
   (`silences.threshold`, `silences.detect`, `silences.cut[3]` como raiz da
   instância): o engine avisa "bound twice in one frame; the first binding
   holds" e a cópia da superfície não é atingível por `hit` nem aparece
   nas ligações (o botão `silences.cut[3].reject_btn` é só da superfície e
   ficou ligado). a 4b tinha visto o template; aqui são os controles.

### a corrida com o Claude

uma corrida, 08:04:52 a 08:07:00, pelo exemplo com `--review claude
--compare rules`, com `PLEV_CLAUDE` apontando para um script no
scratchpad que conta as invocações e recusa a terceira sem rodar nada (o
laço da página tenta até 3 vezes por proposta; o orçamento era 2).
intenção "cut the silences of this video, then I want to review the
cuts"; sonnet; 2 chamadas reais (08:04:53 e 08:06:19): a primeira recusada
pelo verifier (a página mostrou "attempt 2 of 3" a +87,2 s), a segunda
verificada a +125,1 s. `attempts = 2`, `seconds = 124,27`, `cost_usd =
0,194`. superfície `silence_review`, 34 nós: título, preview, um card de
controles com os dois sliders e o detectar, status, cinco stats, e a
lista com `range_text`, `frame_img`, `duration_stat`, `decision_text` e
três botões por corte. a comparação pelas regras sobre ela: o mesmo id,
27 nós, no lugar. a sessão está em
`fixtures/demo/silences-demo-claude.monster`; o resumo do inspector:

```text
span      +0.000s to +127.174s, 200 events, 226 frames
actors    agent:demo: 17 observations, 9 requests, 36 stages
          human: 1 request, 4 stages, 2 inputs
  +0.773s    request 4   agent:demo  propose on silences_surface.propose   completed: surface = "silence_review", nodes = 34, attempts = 2, seconds = 124.2747015, cost_usd = 0.194234
  +125.793s  request 5   agent:demo  apply_cut on silences.cut[4]          completed: decision = "apply"
  +125.826s  request 6   human       reject_cut on silences.cut[3]         completed: decision = "reject"
  +125.844s  request 9   agent:demo  propose on silences_surface.propose   completed: surface = "silence_review", nodes = 27
  +126.579s  request 10  agent:demo  close on silences_surface.close       completed: closed = "silence_review"
```

na timeline: `+87.194s changes silences_surface.status:attempt: 1 -> 2` e
`+125.057s snapshot keyframe, 46 nodes: silence_review added; ...`. a
segunda chamada de reserva não foi usada; nenhuma outra chamada nesta
task.

### contagem do gate e o regress

| | workspace | perna urnaui | total |
| --- | --- | --- | --- |
| antes, `main` @ a9edcbaa (o handoff da 4b, integrado nela) | 2444 passam, 0 falhas, 16 ignorados (111 suítes) | 82 passam, 1 ignorado (6 suítes) | 2526 |
| depois, esta branch @ 82db3ddf (`./tool/tasks/fullcheck.sh`, saída 0) | 2451 passam, 0 falhas, 16 ignorados (113 suítes) | 82 passam, 0 falhas, 1 ignorado (6 suítes) | 2533 |

7 testes novos (4 unitários em `app/record.rs`, 1 em `tests/record.rs`, 2
em `tests/demo.rs`) e 2 suítes; nenhum ignorado a mais; a contagem não
cai. o gate rodou duas vezes: a primeira vermelha só no
`file_hygiene_guard` (`examples/demo.rs` com 736 linhas: o guard conta
exemplos como produção), corrigida pelo split; a segunda inteira verde em
2 min 6 s (08:15:50 a 08:17:56), com a perna urnaui (`../urna` resolve
deste worktree) e o check wasm com `-D warnings`. `df -h /` antes e
depois: 16 GB livres. `cargo run -p regress -- check`:
159 baselines conferidas, 159 iguais, nenhum frame desenhado (a gravação
não muda o que uma página desenha). `cargo test -p uigallery -p record -p
inspect` verdes.

### testes

- `app/record.rs` (4): só "1" liga, nada sem a variável; as teclas de
  salvar (`s` com shift e cmd ou ctrl; não sem shift, não `t`, não Enter);
  a tecla gravada pelo tipo; a página Silences gravada guardando o estado,
  um frame, um `pointer_down` com `silences.apply_all` sob ele e um `text`
  sem caractere, os nomes `showcase-<pid>-0` e `-1`, `hit_at` no frame.
- `tests/record.rs` (1): o shell sem janela pela porta real: o agente
  observa com bounds e aplica o corte 1; a pessoa clica em "apply all"
  (press e release gravados com o controle); o agente rejeita o corte 2 da
  revisão velha e volta `stale`. o arquivo salvo tem as observações com
  escopo e revisão, os 3 receipts inteiros, os dois inputs, os frames com
  as ligações iguais aos bounds servidos no primeiro olhar; o inspector diz
  que o `reject_cut` se apoiou nesse olhar e que `silences.cut[2]:decision`
  mudou pelo `apply_all` da pessoa, com o clique no trecho. o arquivo fica
  em `$CARGO_TARGET_TMPDIR/showcase-record-test.monster`.
- `tests/demo.rs` (2): as duas fixtures, acima.

### seção de ADR

[`0037-session-recording-replay-not-reexecution.md`](../../adr/recording/0037-session-recording-replay-not-reexecution.md), consequências,
um item datado "the showcase records itself too (added on 2026-09-24,
task-69 phase 4c)": o que grava e onde, a demo e as fixtures, e os três
achados sobre superfície gerada com o que cada um pede do checkpoint.
escolhi o ADR da gravação, não o do inspector: os achados são sobre o que
a sessão guarda (o request roteado, a revisão do snapshot só do app, a
ligação duplicada), não sobre como o inspector lê. para o orquestrador
(fora da lista): a linha do índice [`adr/`](../../adr/README.md) pode ganhar no
fim "; desde a task-69 fase 4c o showcase grava como o ide, e a demo do
editor de vídeo é uma sessão em `apps/uigallery/tests/fixtures/sessions`";
`docs/ATLAS.toml`: `showcase::app::record`, o exemplo `demo` e
`tests/{record,demo}.rs`; o CHANGELOG (`PLEV_RECORD=1` no showcase).

### decisões

- `record.rs` tem a `Recording` e os ganchos do `App` num módulo só: o
  shell do showcase vive na lib (o ide tem `record.rs` na lib e
  `recording.rs` no bin);
- o `redraw` saiu de `app.rs` para `pace.rs`: sem isso `app.rs` passaria
  de 639; o frame é do ritmo do loop, e `pace.rs` já tinha `frame_wake` e
  `pace`;
- `ShowcaseView::now(&mut self)` pelo `with_dispatcher`: a única forma de
  ler o relógio do todo sem tocar `apps/**`;
- o estado depois de um ato do agente gravado dentro de
  `serve_agent_recorded` (como o `serve_recorded` do ide), para o shell e
  o exemplo não esquecerem;
- as teclas de salvar como uma conferência de `Key` e `ModifiersState`: a
  galeria não tem keymap nem `ActionRegistry`;
- os inputs: press e release do esquerdo, press do direito, teclas pelo
  tipo; roda e movimento não (o ide também não);
- a demo como exemplo (`examples/demo`), não teste `#[ignore]`: é o
  programa que o brenner roda para gravar de novo, com flags; um teste
  ignorado não teria `--out` nem `--review`;
- a frase da revisão pelas regras é a delas, a da tese fica para o Claude
  (as regras recusam "then" e "review"); a comparação é a mesma frase nos
  dois;
- 1600 por 1000 para a demo, pelo botão fora da janela em 1200 por 800; o
  exemplo aceita outra medida;
- a pessoa clica na superfície gerada, não na tela fixa: é a tese; a
  rolagem até o botão fica no shell da demo (`reveal`), não na página;
- a sessão do Claude commitada (861 KB, abaixo de 2 MB) e lida pelo mesmo
  teste: é a demo de verdade que o brenner abre sem rodar nada;
- o guard de linhas conta exemplos: o exemplo em três arquivos.

### limitações

- o relógio da gravação é o da `ShowcaseView` (o todo nasce nela), então
  a sessão começa em +0.012 s, não em 0;
- as duas fixtures foram gravadas pelo exemplo antes do split (o mesmo
  roteiro; o split juntou o select, a intenção e o propose numa função e
  faz uma observação a mais, 18 em vez de 17); o cabeçalho da sessão diz o
  git do `rec` (`a9edcbaa`), não o do exemplo;
- os frames do vídeo não têm pixels no arquivo (a retenção continua fora,
  task-63); `scene_at` lê a cena sem imagem;
- a revisão das regras em 1200 por 800 não cabe na metade (o botão de
  rejeitar em x = 1232..1341): a galeria no tamanho de lançamento mostra a
  linha cortada à direita; achado para o checkpoint b, a página está fora
  da lista desta fase;
- o cartão do segundo corte fica abaixo da dobra nas duas superfícies em
  1600 por 1000 (o agente age por id; a pessoa rolaria);
- a sessão do Claude tem 124 mudanças de status e 124 frames só do
  contador da espera (uma linha por segundo);
- a gravação é só desktop; `PLEV_BACKGROUND` não foi preciso (sem janela);
- o `plev-inspect` não diz que um ato veio pela superfície gerada (achado
  1), nem coloca a observação de um clique da pessoa (achado 2);
- o script guarda do claude e a contagem ficaram no scratchpad, nada
  commitado.

### dependências pendentes e riscos de integração

- `Cargo.toml` do showcase: `rec` (desktop) e `inspect` como
  dev-dependency (desktop); `Cargo.lock` duas linhas; o ciclo `rec -> dev
  -> showcase -> rec` já existia por `ide`;
- `Served` de `dispatch/agent.rs` ganhou o campo `recorder`: quem monta à
  mão passa `None`; ninguém fora do arquivo;
- `ShowcaseView::now` e `serve_agent_recorded` novos; `serve_agent`,
  `add_sink`, `semantics`, `act` iguais;
- `app.rs`: `render_frame` agora é chamado de `pace.rs`; quem mexer no
  `RedrawRequested` acha `redraw` lá;
- as baselines do regress não mudaram; `.gitattributes` já trata
  `*.monster` como binário.

### o que falta para a task-69 fechar

- o checkpoint b, do brenner: as duas telas lado a lado (a fixa da 4a e a
  gerada da 4b), agora com as duas demos gravadas para abrir com
  `plev-inspect` (`apps/uigallery/tests/fixtures/sessions/*.monster`) e regravar
  com o exemplo; os achados 1 a 3 e a linha fora da janela em 1200 por
  800 são material para ele;
- um mp4 da tela é decisão do brenner (a demo é a sessão `.monster`);
- as linhas do índice de ADR, do arc e do CHANGELOG, pelo orquestrador.
