---
id: task-80/fase-demo-5-ui-generativa
title: "task-80, demo 5: UI generativa em aplicação real"
project: plev
type: secao
parent: task-80
status: referencia
date: 2026-09-25
date_basis: cabecalho
topics: [superficies-geradas, gravacao, ir]
relations: [task-69, task-93, task-96, task-97]
evidence: [2e6c9e9a, e42bd571, 10f66729]
source: docs/log/steps/task-80-proof-qualitatively-different-software.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# demo 5, UI generativa em aplicação real (feita pela task-96)

A demo 5 da task-80, uma superfície gerada numa aplicação real, foi feita pela task-96 e medida de novo com o `code` do IR pela task-97 e pela fase b da task-96. Rodada pelo laboratório sem janela e sem Claude, com uma pessoa roteirizada, a revisão de três commits no ide saiu com resultado misto: a superfície mostra os arquivos dos três commits de uma vez, e o painel fixo não; ela perde em atos (5 cliques e 26 teclas contra 3 cliques) e em linhas do diff à vista, diferença que a task-97 reduziu sem eliminar. Pelo critério escrito antes, a superfície não venceu o painel fixo na tarefa inteira.

rodada pelo laboratório sem janela, `ide-lab --arm review`, no build
`2e6c9e9a` da branch `task/96a-ide-surface`, sobre o repositório da
revisão do laboratório (`Repo::build_review` em
`apps/idestudio/src/repository/eval_repo.rs`: o commit inicial, três commits por cima em
datas fixas, os shas iguais em toda corrida, e um `notes.txt` na árvore
que nenhum commit tem). só as regras: sem porta, sem agente, sem Claude.
a pessoa roteirizada clica no meio de cada controle como o frame na tela
o desenha (achado pelo id nas ligações do frame, pelo `Workbench::click`,
como a resposta dela na task-93) e digita a intenção uma tecla por vez.
primeiro o controle: os três cartões de commit do painel de stacks, um
clique cada, o diff no painel de diff. depois a superfície: o campo da
linha, "review these three commits", propor, o "show diff" de cada
commit na superfície, fechar. o teste `apps/agenteval/tests/review.rs`
roda o mesmo braço no gate.

## a gravação

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

## a leitura pelo inspector

```text
$ plev-inspect review.monster --control proposed --at 100
control proposed at +0.100s
  node          proposed: region "review these three commits", enabled
  source        rust/screen/src/project.rs:96:11, in build 2e6c9e9a

$ plev-inspect review.monster --control proposed --at 179
control proposed at +0.179s
  node          not in the recorded state at that instant

$ plev-inspect review.monster --control 'proposed.stacks.commit[78f371df...].files' --at 128
  node          proposed.stacks.commit[78f371df...].files: table "files", enabled
  state         rows = [src/login.rs]
  source        rust/screen/src/project.rs:104:23, in build 2e6c9e9a

$ plev-inspect review.monster --control 'proposed.stacks.commit[78f371df...].show_diff' --at 128
  node          ...show_diff: button "show diff", enabled
  actions       activate (reads proposed.diff.target:text)
  source        rust/screen/src/project.rs:104:23, in build 2e6c9e9a
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
(`rust/screen/src/project.rs`); os commits vêm de
`apps/idestudio/src/capability/project.rs` (o teste confere nó a nó no estado
gravado). os dois achados da task-69 fase 4c continuam: o ato pela
superfície chega ao dispatcher como a capacidade do app, e só o input da
pessoa nomeia o botão gerado; e a observação da pessoa não é colocada. no
ide ela é ainda um passo mais longe: o request da superfície é observado
no snapshot do app sozinho (o que a superfície mostra), e o gravador
guarda o do workbench inteiro (o app, a linha e a superfície), então a
revisão que o request carrega nunca aparece na gravação.

## o resultado contra o painel fixo, pelo critério do método

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
no comps primeiro e um ADR (`rust/plevir/src/vocabulary.rs`), decisão do
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

## privacidade da gravação

- a entrada da pessoa: 9 `pointer_down`, cada um com o controle sob ele e
  a posição, e 26 `text`, sem alvo e sem conteúdo (o tipo do evento não
  tem campo para ele).
- a intenção chega à gravação como o estado projetado da linha
  (`ide_surface.intent:text`), como a demo 4 anotou para a Silences.
- segredo: nenhum nó do ide é `secret`; nada a redigir aqui.
- o `notes.txt`, fora dos commits, está na gravação só como o que o app
  projeta no painel de changes, nunca na superfície.

## de novo, com o `code` no IR (task-97)

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

### a gravação

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
  source        rust/screen/src/project.rs:104:23, in build 10f66729

$ plev-inspect review.monster --control proposed --at 232
control proposed at +0.232s
  node          not in the recorded state at that instant
```

### o resultado lado a lado, pelo mesmo critério

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
  números antigo e novo (`apps/idestudio/src/capability/project.rs`, `diff`).
- os atos: a intenção digitada e o propor, como na primeira corrida.

o que isso não cobre, dito como na primeira: só as regras; a pessoa
roteirizada, então o tempo é de máquina (e varia de corrida a corrida: o
painel fixo levou 47 ms na primeira e 64 ms nesta); diffs pequenos; a
demo com janela pelo brenner segue aberta. com a superfície aberta, o
painel fixo, mais estreito, continua mostrando o mesmo diff à esquerda,
então a pessoa vê o diff duas vezes; o critério compara os dois braços
separados, como antes.

### privacidade da gravação

a mesma da primeira: 9 `pointer_down` com o controle sob ele e a
posição, 26 `text` sem alvo e sem conteúdo; a intenção só como o estado
projetado da linha; nenhum nó `secret`; o `notes.txt` fora dos commits
nunca na superfície. o que entrou: `diff:kinds`, os nomes dos tipos das
linhas do diff que o app já mostra, no estado gravado ao lado de
`diff:lines`.

## de novo, com a linha embaixo (task-96, fase b)

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

