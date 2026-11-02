---
id: task-69/checkpoint-b-fixes
title: "task-69, checkpoint b: the corrections"
project: plev
type: section
parent: task-69
status: reference
date: 2026-09-24
date_basis: text
recorded: 2026-09-23
topics: [generated-surfaces, rendering, layout]
relations: [adr-0051, task-75, task-77]
evidence: [253be85d, 210564b8, 039b5cf6, f4b46794, b8665828, 25633cba]
source: docs/log/steps/task-69-ephemeral-generative-surfaces.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# checkpoint b, the corrections (task/69i)

Records the corrections of checkpoint b: a line of a surface does not shrink the text below the longest word and wraps when it passes the edge; the slider shows label and value; numbers have at most two decimal places; the prompt says how a surface is read; and the regress now stages the generated screen. The result is six new baselines and 169 equal in the regress, and the generated screen fits the default window.


branch `task/69i-checkpoint-b`, sobre a `main` em `5cfb0dfb`.

## sonda das correções (antes do primeiro tipo, 2026-09-24)

reproduzido em teste, na galeria de 1200x800 com o worker falso
(`silences/tests/generated/checkpoint.rs`, a versão de sonda que só
imprime): a coluna da superfície vai de x = 740 a 1160 (420 px); o card
das regras tem `padding: lg` (20 px), então a linha do corte tem 380 px.

(a) a linha. `rust/screen/src/layout.rs:138` passa o texto ao engine
como `Leaf::Text` sem largura mínima, e o `LayoutStyle` nasce com
`flex_shrink: 1.0` e `flex_wrap: false` (`rust/engine/src/compose/layout/style.rs:75`
e `:77`); o `stack` só põe direção, gap e alinhamento (`layout.rs:175` a
`183`), nunca `flex_wrap` (só o `grid` quebra, `layout.rs:184` a `188`).
no taffy 0.9.2 o mínimo automático de um item flex é o seu min-content
(`compute/flexbox.rs:798` a `818`), e o engine mede o min-content de um
texto com largura 0 (`rust/engine/src/compose/layout/solver.rs:143`): o shaper
quebra dentro da palavra, e o texto encolhe até um glifo. os botões e a
imagem são `Leaf::Fixed` com `flex_shrink = 0` (`layout.rs:133` a `137`),
não encolhem, e numa linha que não quebra passam da borda. a linha das
regras é um `stack` horizontal (`rust/offers/src/rules.rs:429`) com a
imagem `xl` (`:386`), cinco textos com `flex: 1` (`:395`) e três botões.
os números: os cinco textos com 9 a 10 px de largura e 158 a 360 px de
altura (um caractere por linha), quando a palavra mais longa de cada um
mede 43,8 a 63,8 px; "preview segment", "apply cut" e "reject cut"
terminam em x = 1107, 1224 e 1341, este 181 px além da borda do card. a
fixture do Claude (item 5 abaixo) repete o defeito na linha do corte: a
faixa com 9 px e os botões até x = 1348. não precisa mexer no engine
nem no comps: a largura mínima do texto e a quebra da linha são estilos
que a superfície já passa ao `LayoutEngine`.

(b) o slider. `rust/screen/src/widget.rs:185` cria o
`Slider::new(0.0, 1.0, 0.0).sem(id)` sem rótulo; o `sync_slider`
(`widget.rs:438` a `454`) põe faixa, passo, valor e `disabled`; o
`draw.rs:141` dá à folha só a altura do controle (`ControlSize::Xs`, 36
px) e o `draw.rs:330` chama `s.render`, que desenha trilho e botão e
nenhum texto (`rust/engine/src/widgets/form/slider.rs:243` a `330`). o rótulo do
nó nunca é desenhado. a tela fixa desenha ela mesma o nome em
`caption_sm`/`text_mid` e o valor em `caption_r`/`text`, alinhado à
direita, numa linha acima do slider (`silences/draw.rs:209` a `247`,
`silences/layout.rs:201` a `211`: a linha do rótulo, `spacing.xs`, o
slider).

(c) o número. `rust/screen/src/widget.rs:94`, `Value::Num(n) =>
n.to_string()`, no `text_of`, que serve o texto e o badge (`shown_text`,
`draw.rs:63`), o stat (`draw.rs:84`) e o rótulo que o texto e o badge
projetam (`project.rs:43`). a tela fixa formata com o que sabe do app:
`"{:.0} dB"` (`silences/draw.rs:222`), `"{:.1} s"` (`:228`), `"{:.1} s of
silence"` (`:319`). o IR não tem campo de unidade nem de formato (os
`prop` do stat e do texto são só `Text`, `Int`, `Num`,
`rust/plevir/src/vocabulary.rs:246` e `:290`), e não precisa: a unidade já vem
no rótulo, o do app ("threshold (dB)", "minimum silence (s)") e o do
Claude ("duration (s)"). o formato mais justo sem tocar o IR: no máximo
duas casas, sem zero à direita. o vocabulário fica como está; não há
parada.

(d) o prompt. `rust/offers/src/prompt.rs:271` a `306`: o pedido, a
intenção, as regras do verificador, a política, os componentes, o
registro, a superfície de agora e o schema. nada sobre ordem, sobre o
que vem primeiro, sobre repetir um controle; de rótulo, só a regra de
acesso (`prompt.rs:49` a `53`: todo controle tem rótulo), que o Claude
cumpriu e o slider não desenhou. o `SYSTEM`
(`rust/claude/src/lib.rs:55` a `57`) só pede a resposta na forma
pedida: não precisa mudar.

(e) o regress. `run::stage` (`examples/regress/run.rs:52` a `76`) chama
`showcase::app::stage_page` (`src/app/headless.rs:77` a `111`): a view
nova, a página, o tema, 120 passos de `tick` e `render`. só o estado
inicial; a chave do baseline é página, tema e viewport
(`examples/regress/baseline.rs:40` a `64`). a Silences parada nunca chama
o ffmpeg (`silences/worker.rs:10` a `15`), mas uma superfície com imagem
pede os frames no `wake` (`silences/jobs.rs:70` e `184` a `190`), e fora
dos testes o worker é o ffmpeg de verdade (`worker.rs:88` a `95`); o falso
(`worker.rs:98` a `103`, `256` a `283`) só existe sob `cfg(test)` e o
exemplo, que é outro crate, não o vê. do exemplo, a página só é
alcançável pela API pública da `ShowcaseView` (`view/mod.rs:10`, `mod
apps` privado; `panes` privado em `view/mod.rs:97`): `act`, `semantics`,
`tick`, `render`. as regras chegam por `act` (proposer, intenção,
propose, como no roteiro da 4c); a fixture do Claude não tem caminho
público (as operações da linha são escrever, propor, escolher, abrir,
fechar e cancelar, `surface_page/proposing.rs:401` a `412`). o que
precisa: no regress, encenar um estado com nome (o roteiro pelo `act`) e
um tipo de página encenada que não seja o `StagedPage` (campos privados,
`run.rs:32`); no showcase, em `generated/page.rs`, dois ganchos de
encenação: os frames dos instantes que a página mostra postos no cache
antes do `wake` pedir (o conteúdo vem do exemplo, sintético e
determinístico; o hash de uma imagem é o tamanho, `examples/regress/hash.rs:30`
a `33`), e a fixture verificada pelo `uir` contra o registro do app e
mostrada por `Page::show` (`page.rs:156` a `170`).


## handoff das correções do checkpoint b (2026-09-24)

```text
Goal: task-69 checkpoint b, as correções: a tela gerada usável na janela
      padrão (a linha não encolhe o texto, os números têm formato, o
      slider mostra rótulo e valor, a proposta põe a tarefa primeiro) e a
      imagem de referência da tela gerada aberta no regress
Status: pronto para o orquestrador rodar o checkpoint de novo
```

### commits

- 253be85d task-69: the probe of the checkpoint b fixes
- 210564b8 task-69: a row keeps every word whole and wraps instead of overflowing
- 039b5cf6 task-69: the IR slider shows its label and its value
- f4b46794 task-69: a generated surface shows a number with two decimals at most
- b8665828 task-69: the prompt says how a surface reads, before the rules
- 25633cba task-69: the Claude's review as a fixture and the review open in the regress
- o commit deste texto

### a sonda

está na seção "sonda das correções" acima, com a reprodução em teste:
(a) a linha do corte das regras a 1200x800 tem 380 px; os cinco textos
encolhem a 9 ou 10 px (um glifo: o mínimo automático do taffy é o
min-content, que o engine mede com largura 0) e os três botões, fixos
numa linha que não quebra, terminam em x = 1107, 1224 e 1341, 181 px além
do card; (b) o slider do IR desenha só trilho e botão, o rótulo do nó
nunca; (c) `widget.rs:94` era `n.to_string()`, e o IR não precisa de
unidade nem formato (a unidade está no rótulo), então não houve parada;
(d) o prompt não falava de ordem nem de repetição; o `SYSTEM` fica; (e) o
regress só encenava o estado inicial e o exemplo só alcança a galeria pela
API pública: dois ganchos de encenação resolvem.

### cada defeito e a correção

- **a linha encolhe o texto e os botões passam da borda.** em
  `rust/screen/src/layout.rs`: um texto nunca fica mais estreito que a
  sua palavra mais longa (`longest_word`, `:102`, medida palavra a palavra
  com o estilo do texto, no cache do medidor; aplicada como `min_width` em
  `:214` e depois do `size`), e uma linha (`stack` horizontal, `is_row`,
  `:90`) que ainda passa da sua caixa de conteúdo depois de encolher o que
  encolhe quebra (`flex_wrap`, `:265`): `compute` (`:57`) faz a passada,
  `overflowing` (`:150`) acha as linhas que passam por mais de meio pixel
  de um lado ou do outro, e a árvore é disposta de novo com elas quebrando,
  até nenhuma passar. encolher vem antes de quebrar: um texto longo ao lado
  de um botão continua na mesma linha e quebra entre as palavras (o teste
  `a_row_shrinks_its_text_before_it_wraps`). nada no engine nem no comps.
- **o slider sem rótulo nem valor.** em `rust/screen/src/draw.rs:352`
  (`slider`) e `widget.rs:477` a `498`: a folha do slider ganhou uma faixa
  acima do trilho, como a da tela fixa: o rótulo do nó à esquerda em
  `caption_sm` e `text_mid`, o valor do widget à direita em `caption_r` e
  `text` (o valor do widget, então um arraste mostra para onde vai), e o
  trilho embaixo, na altura do controle. o trilho é o que o ponteiro
  arrasta (`widget.rs:385`) e onde o slider se liga, como na fixa.
- **o número cru.** `widget.rs:107`, `number_text`: no máximo duas casas,
  sem zero à direita (1.996916 é "2", 3.018594 é "3.02", 0.5 fica "0.5",
  -30 fica "-30", -0.001 é "0"; o que não é finito sai como o Rust
  escreve). vale na tela e no rótulo que um texto ou badge projeta (o que
  um agente lê como nome); o estado do app guarda todas as casas. o IR não
  mudou.
- **a proposta põe a tarefa por último.** em
  `rust/offers/src/prompt.rs:73` (`READS`) e `:297`: logo depois da
  intenção e antes das regras, a seção "how the surface reads", a mesma
  para todo app: primeiro o que a intenção pede para fazer (a lista a
  revisar, a ação a rodar, o valor a mudar), depois o contexto (o arquivo,
  as contagens, os ajustes), porque a janela pode mostrar só o topo; todo
  controle com um rótulo que diz o que ele muda, com a unidade; cada
  controle uma vez e só o que a intenção pede (a tela do app já oferece
  toda capacidade do registro). é orientação, o verificador não confere
  nada disso. o `SYSTEM` do `claude-model` não mudou. a prova com o Claude
  de verdade é do orquestrador.

### a fixture do Claude

`rust/plevir/tests/fixtures/silences-review-claude.ir.json`: a
superfície `review` ("Review Silence Cuts") com os nós `review*` da
observação da primeira rodada (`obs-claude.json`), na ordem: o título, o
card do vídeo (arquivo, duração, frame original e resultado), o card da
detecção (os dois sliders e o botão), o card do resumo (a mensagem e os
cinco stats), aplicar tudo, e a lista que repete sobre `silences:cuts` com
faixa, estado, duração, frame, prévia, aplicar e rejeitar por corte. ids,
papéis, rótulos, ligações e ordem vêm da observação; o layout da parte
visível foi inferido dos retângulos dela (a raiz vertical com gap `md`, os
cards verticais com gap `sm` e sem padding, o título em texto simples) e
bate no pixel: `the_claudes_review_fixture_is_the_first_rounds_structure`
confere os retângulos observados do título às duas imagens. abaixo da
dobra a observação não tem retângulo, então é escolha minha, dita aqui: o
card do resumo vertical com gap `sm`, o item do corte horizontal com gap
`sm` e centrado, o estado como badge, o frame do corte com `control_size:
xl`. verificada pelo verificador do `uir` contra o registro do app
(`GenerationPolicy::default()`, como as fixtures da 4b), no teste e no
regress. com o código de antes, a encenação dela reproduz a
[`04-generated-claude.png`](../../img/task-69-checkpoint-b/04-generated-claude.png) (os sliders sem rótulo nem valor, o botão de
detectar na dobra).

### o regress: a página num estado

`tool/bench/regress/src/state.rs` (novo): um estado tem nome, e
o nome faz as vezes da página na chave do baseline (`--page "<nome>"`,
qualquer caixa). os dois de hoje (`STATES`, `:52`): "Silences, review by
the rules" e "Silences, review by the Claude (fixture)". a encenação
(`StagedState::stage`) abre a página como o `stage_page` abre, põe os
frames, escreve a intenção pelo `act` como um ator de teste (`regress`),
propõe pelas regras pelo `act` (proposer "rules", propose) ou mostra a
fixture pelo gancho, e assenta no mesmo relógio fixo (120 passos a 60
Hz). os frames são sintéticos (`synthetic`, `:244`: sete barras de 320 por
180 e uma faixa cujo cinza segue o instante), então nenhum ffmpeg roda e o
frame é o mesmo em qualquer máquina. `run.rs` passou a guardar um
`state::Page` (a página da galeria ou o estado) no lugar do `StagedPage`;
o `capture` (task-77) encena pelo mesmo caminho. `--all` continua sendo as
páginas da galeria.

os ganchos, em `silences/generated/page.rs:337` a `380`, métodos
públicos da `ShowcaseView` só para o harness: `stage_silences_frames`
põe no cache da página um frame para cada instante que ela mostra ou que
a superfície liga (o mostrado, o do resultado, o meio de cada corte, cada
frame ref das imagens), feito pela função que o exemplo passa, e a página
não pede nenhum ao worker; `stage_silences_ir` verifica o JSON pelo `uir`
contra o registro do app e mostra como uma proposta mostra
(`Page::show`). a linha de status não diz nada da fixture: nenhum
proposer foi chamado.

### as imagens

| vista | antes, a rodada real | antes, a mesma encenação com o código de antes | depois (baseline) |
| --- | --- | --- | --- |
| regras, 1200x800 core | [`03-generated-rules.png`](../../img/task-69-checkpoint-b/03-generated-rules.png) | `before/silences-review-by-the-rules@core@1200x800/frame.png` | `tool/bench/regress/baselines/silences-review-by-the-rules@core@1200x800/frame.png` |
| Claude, 1200x800 core | [`04-generated-claude.png`](../../img/task-69-checkpoint-b/04-generated-claude.png) | `before/silences-review-by-the-claude-fixture@core@1200x800/frame.png` | `tool/bench/regress/baselines/silences-review-by-the-claude-fixture@core@1200x800/frame.png` |
| regras, 1280x800 core e hoff | | | `.../silences-review-by-the-rules@{core,hoff}@1280x800/frame.png` |
| Claude, 1280x800 core e hoff | | | `.../silences-review-by-the-claude-fixture@{core,hoff}@1280x800/frame.png` |

o "antes" encenado está na scratchpad desta sessão
(`/private/tmp/claude-501/-Users-nn-Dev-hoff-plevdev/ffe9cd40-0b24-414e-be8c-32dec2cf8b3d/scratchpad/before/`),
fora do repositório; para refazer: `git checkout 253be85d --
rust/screen/src`, `regress bless` dos dois estados com `--width 1200
--png --dir <dir>`, e `git checkout HEAD -- rust/screen/src`. ele
repete a 03 e a 04 ponto por ponto, fora os frames (barras no lugar do
testsrc) e os dados (a semente, cortes 1 e 2, no lugar da detecção real).

o que o depois mostra a 1200x800: pelas regras, a linha do corte com o
frame e os cinco textos inteiros ("3.02 5.02 2 0:03.0 to 0:05.0 pending")
numa linha, "preview segment" e "apply cut" na de baixo e "reject cut" na
seguinte, tudo dentro do card; a 1280x800 a linha tem 420 px e os três
botões cabem numa linha só. pela fixture do Claude, os sliders com
"threshold (dB)" e "-30", "minimum silence (s)" e "0.5" acima do trilho;
a lista dos cortes continua abaixo da dobra, porque é a estrutura que o
Claude propôs (o que o prompt corrige só aparece numa proposta nova).

### baselines novos e mudados

novos, seis, todos com a imagem: `silences-review-by-the-rules` e
`silences-review-by-the-claude-fixture`, cada um a 1280x800 em core e
hoff (o pedido) e a 1200x800 em core (a janela do checkpoint, para a
comparação com `checkpoint-b/`). mudados: nenhum. os 163 de antes ficaram
iguais, porque nenhum mostra uma tela gerada (não há baseline da página
Surface; o da Silences é a página parada, sem superfície), então não houve
antes e depois a aceitar.

### contagem do gate e o regress

| | workspace | perna urnaui | total |
| --- | --- | --- | --- |
| antes, `main` @ 5cfb0dfb (`./tool/tasks/fullcheck.sh`, saída 0, 20:29:09 a 20:31:12) | 2736 passam, 0 falhas, 25 ignorados (138 suítes) | 85 passam, 0 falhas, 1 ignorado (6 suítes) | 2821 |
| depois, esta branch @ 25633cba (`./tool/tasks/fullcheck.sh`, saída 0, 20:53:21 a 20:55:34) | 2747 passam, 0 falhas, 25 ignorados (139 suítes) | 85 passam, 0 falhas, 1 ignorado (6 suítes) | 2832 |

11 testes novos e uma suíte (`rust/screen/tests/checkpoint.rs`);
nenhum ignorado a mais; a contagem não cai. o gate inteiro verde nas duas
vezes, com a perna urnaui (`../urna` resolve deste worktree) e o check
wasm com `-D warnings`; `df -h /`: 19 GB livres antes, 18 GB depois.
`cargo run --offline -p regress -- check`: antes 163
baselines, 163 iguais (834 ms); depois 169, 169 iguais (879 ms), nenhum
frame desenhado.

### testes

- `rust/screen/src/layout.rs` (2): a linha do corte das regras num card
  de 420 e de 280 px quebra, nada passa da linha e nenhum texto fica
  abaixo da palavra mais longa (vermelho com o código de antes); a linha
  encolhe o texto antes de quebrar.
- `rust/screen/src/widget.rs` (1): o formato do número, doze casos, o
  não finito e o inteiro.
- `rust/screen/tests/checkpoint.rs` (2, suíte nova): o slider do
  mixer com "volume" e "0.5" acima do trilho, e "0.75" depois de um
  arraste; um stat e um texto sobre os números do corte 1 mostram "2" e
  "3.02", na tela e no rótulo projetado, e o app guarda 3.018594.
- `rust/offers/tests/llm.rs` (1): a seção vem depois da intenção e
  antes das regras, diz as três coisas, é a mesma no todo e no ide e não
  nomeia app nenhum.
- `silences/tests/generated/checkpoint.rs` (4): a revisão das regras na
  galeria de 1200x800 (o card em x = 740, 420 de largura; tudo dentro;
  os textos inteiros; os botões na linha de baixo) e numa página de 358
  px (as duas vermelhas com o código de antes); a fixture do Claude pelos
  ganchos é a estrutura da primeira rodada, na ordem, com os retângulos
  observados; e está usável: tudo dentro, os sliders com rótulo e valor,
  a duração "2" e nunca "1.996916", os sete frames desenhados e nenhum
  job para o ffmpeg.
- `examples/regress/tests.rs` (1): os dois estados abençoados e
  conferidos contra o próprio bless sem desenhar frame, o nome do baseline
  pelo estado, a superfície na projeção; uma página desconhecida é erro.

### decisões

- a linha quebra só quando precisa. quebrar sempre mudaria o caso comum
  (um texto longo e um botão: o botão desceria e o texto ficaria com a
  linha inteira); encolher primeiro e quebrar depois é o que o CSS faz com
  `flex-wrap` quando o mínimo do texto é a palavra. custa uma passada a
  mais por nível de linha que transborda, e nenhuma quando tudo cabe.
- o mínimo do texto é a palavra mais longa, medida pela superfície, e não
  uma mudança no `measure_text_node` do engine (que mede o min-content com
  largura 0): o engine fica fora do escopo e a folha de texto já é da
  superfície.
- o slider se liga ao trilho, não ao nó inteiro, como o da tela fixa; o
  rótulo e o valor ficam na faixa acima, fora do alvo do ponteiro.
- o formato vale também no rótulo projetado de um texto ou badge: o nome
  que o agente lê é o que a pessoa vê. o valor do estado não muda.
- a fixture foi verificada pelo `uir` e não pelo `propose_verified`: não
  há proposer nela, e o briefing aceita os dois.
- os ganchos de encenação são públicos na `ShowcaseView` porque o exemplo
  é outro crate e o worker falso existe só sob `cfg(test)`; ficam em
  `generated/page.rs`, a única casa permitida perto do cache de frames.
- os dois baselines a 1200x800 além dos quatro pedidos: são a vista das
  imagens do checkpoint, e ficam versionados em vez de só na scratchpad.

### limitações

- abaixo da dobra a fixture é inferida (acima, dito em "a fixture do
  Claude").
- a linha das regras é plana (frame, cinco textos, três botões): quebrada,
  ela desce em ordem, sem agrupar por sentido ("reject cut" sozinho na
  terceira linha a 1200x800). agrupar é da estrutura que as regras montam
  (`rules.rs` e `list.rs`), fora desta lista de arquivos.
- os frames do regress são barras sintéticas, não o testsrc; o hash de
  uma imagem é o tamanho, e os pixels são comparados só quando uma camada
  muda.
- como o baseline da Silences parada, os estados dependem do `claude`
  estar no PATH da máquina que abençoa (o proposer lista "claude" e o
  escolhe por padrão quando ele existe): numa máquina sem ele a projeção
  do seletor muda. é o mesmo de antes, não é novo.
- a quebra só olha o eixo horizontal de um `stack`; um filho fixo mais
  largo que a coluna inteira continua passando (nada a quebrar).

### linha de ADR (para o orquestrador; o ADR não está na minha lista)

em [`0051-materialized-surfaces-from-verified-ir.md`](../../adr/Synthesis/0051-materialized-surfaces-from-verified-ir/0051-materialized-surfaces-from-verified-ir.md), consequências:
"uma linha encolhe o que encolhe e, se ainda passa da borda, quebra; um
texto nunca fica mais estreito que a sua palavra mais longa; o slider
mostra rótulo e valor acima do trilho; um número aparece com no máximo
duas casas, na tela e no rótulo projetado (checkpoint b da task-69)".

### roteiro para o orquestrador rodar o checkpoint de novo

os mesmos passos da primeira rodada, pelo contrato, sem clique sintético,
na mesma janela de 1200x800, com o binário desta branch (ou da `main`
depois de integrar):

```text
cargo build -p uigallery                     # e copiar o binário, como na 1ª rodada
PLEV_AGENT=1 PLEV_MIRROR=1 PLEV_BACKGROUND=1 PLEV_RECORD=1 <showcase> Silences

# 02, a fixa com a detecção real
plev act silences.detect detect_silence --app showcase --name orquestrador
plev observe --app showcase --name orquestrador --scope silences.status --keys busy   # até busy = false
cargo run -p mirroring -- --app showcase --frames 1 --png 02-fixed-detected.png

# 03, pelas regras
plev act silences_surface.proposer select --param '"rules"' --app showcase --name orquestrador
plev act silences_surface.intent set_value --param '"the list with the video and the counts"' --app showcase --name orquestrador
plev act silences_surface.propose propose --app showcase --name orquestrador
cargo run -p mirroring -- --app showcase --frames 1 --png 03-generated-rules.png

# 04, pelo Claude (a cota é do orquestrador)
plev act silences_surface.close close --app showcase --name orquestrador
plev act silences_surface.proposer select --param '"claude"' --app showcase --name orquestrador
plev act silences_surface.intent set_value --param '"cut the silences of this video, then I want to review the cuts"' --app showcase --name orquestrador
plev act silences_surface.propose propose --app showcase --name orquestrador
plev observe --app showcase --name orquestrador --scope silences_surface.status --keys busy,message   # até a resposta
cargo run -p mirroring -- --app showcase --frames 1 --png 04-generated-claude.png

# 05, a tarefa: o primeiro corte aplicado pela fixa, o segundo rejeitado pela gerada
plev act silences.cut[<primeiro>] apply_cut --app showcase --name orquestrador
plev act <o reject do segundo corte na gerada, pelo id que a observação der> activate --app showcase --name orquestrador
cargo run -p mirroring -- --app showcase --frames 1 --png 05-after-task.png
```

sem janela e sem cota, antes: `cargo run -p regress --
check` (169 iguais) e as seis imagens acima. o que olhar na rodada nova:
na 03, a linha do corte inteira dentro do card, os números com duas
casas; na 04, se a proposta nova põe a lista dos cortes no topo (o
prompt), se os sliders mostram rótulo e valor, se as durações saem "2",
e se ela repete a detecção da fixa sem a intenção pedir.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0051: a surface materializes only from verified IR, projects from the IR and asks through the app's dispatcher](../../adr/Synthesis/0051-materialized-surfaces-from-verified-ir/0051-materialized-surfaces-from-verified-ir.md)
- [task-69: ephemeral and generated surfaces](task-69-ephemeral-generative-surfaces.md)
- [task-75: production-ready platform accessibility](../task-75-platform-accessibility/task-75-platform-accessibility.md)
- [task-77: visual determinism](../task-77-visual-determinism/task-77-visual-determinism.md)
- [generated silences screen against the fixed screen, checkpoint b of task-69](../../Research/Exploring/generated-surface-checkpoint-b.md)
<!-- /related -->
