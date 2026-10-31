---
id: task-70/phase-4-accessibility
title: "task-70: phase 4, accessibility as a transformation"
project: plev
type: section
status: reference
parent: task-70
date: 2026-09-25
date_basis: text
topics: [accessibility, generated-surfaces, ir]
relations: [task-70/phase-1-operations, task-70/phase-3-composite-capabilities, adr-0067, task-75]
source: docs/log/steps/task-70-adaptive-transformations.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
evidence: [921e159a, 490fe38d, d39385b7, 89a97fa0, 7b34736f, c8c16ed1, 8124e8a7]
---

# phase 4: accessibility as a transformation (opened 2026-09-25)

Records phase 4 of task-70, opened on 2026-09-25: the person's preferences (larger text, more contrast and density) become phase 1 operations within the tokens, applied on their behalf to every open generated surface and to those opened later. Switching a preference off restores only what that preference changed, because the host draws the preferences as the last layer, and the `promote` of a template moves from the host's `registry` to the `uir`. The pairwise contrast measurement leaves a stat's label at 4.13 in the core, which no token reaches; the decision between theme and comps rests with brenner.

aberta pelo orquestrador depois da task-75 fase 1. as preferências da
pessoa (texto maior, mais contraste, densidade) mudam a superfície gerada
pelas operações da fase 1, dentro dos tokens, em nome dela; desligar
devolve só o que a preferência mudou; o contraste é medido como a task-75
mede. junto, a dívida P2 da fase 3: o `promote` de um template sai do
`registry` do host e vai para o `uir`. nenhuma leitura nova do sistema e
nenhuma cor de tema muda. fica de fora a prova com o Claude de verdade da
fase 2, que é do orquestrador.

branch `task/70d-a11y-transforms`, sobre a `main` em `91bade03`.

## sonda (antes do primeiro tipo, 2026-09-25)

o que existe hoje: nenhuma preferência além do reduced motion. nada em
`apps/uigallery/src`, `rust/screen/src` ou `rust/plevir/src` guarda
texto maior, contraste ou densidade da pessoa (grep). as operações da
fase 1 que mexem na aparência são três, só por token:
`rust/plevir/src/transform/look.rs`, `emphasize` (`:97` a `144`),
`density` (`:160` a `202`) e `restyle` (`:233` a `262`). o host guarda o
histórico como uma pilha (`rust/screen/src/host/mod.rs:106` a `114`,
`Adapting`) e desfaz só pelo topo (`take_back`, `:355` a `402`).

### (a) as preferências: quais, onde moram, quem liga

- **quais.** três nesta fase: texto maior, mais contraste, densidade
  (mais denso ou mais espaçado, uma das duas). o reduced motion já vale
  desde a 75: o valor é da thread (`rust/engine/src/compose/animation/mod.rs:15`
  a `40`), e a superfície gerada passa só por `Spring` (a entrada e a
  saída, `rust/screen/src/transition.rs:41` a `52`) e `Tween` (o deslizar,
  `:307` a `316`). falta um frame: a transição nasce no primeiro desenho
  depois da mudança (`rust/screen/src/surface.rs:295` a `305`, `enter`
  e `slide`) e é lida logo em seguida, antes de qualquer tick
  (`:307` e `:308`; o fantasma, `motion.rs:187` a `197`): o nó que nasce
  sai com opacidade 0, o que mudou de lugar sai no lugar de antes, o que
  saiu sai inteiro; só o tick seguinte (`Motion::wake`, `:345`) pousa. o
  flag do próprio host, que não cria nada disso (`host.rs:505` a `515`,
  `motion.rs:245` a `256`), nenhuma página liga (grep de
  `set_reduced_motion` em `apps/uigallery/src`: nada). a correção cabe
  no `surface`: o `Motion` lê também `engine::animation::reduced_motion()`,
  o valor que a ponte já põe; nenhuma leitura nova do sistema.
- **onde moram.** no app, como valor da pessoa: o `SurfaceHost` guarda as
  preferências e quem as ligou, e a página que mostra superfícies as
  passa a ele. um host novo nasce sem nenhuma, então uma thread (um
  teste, um frame sem janela, o regress) começa sem preferência, como o
  reduced motion. nada é lido do sistema; o que seria preciso para ler
  "aumentar contraste" e o tamanho de texto do macOS fica no fim.
- **quem liga, e onde.** só a pessoa. a primeira ideia, uma ação
  `PERSON_ONLY` sempre projetada na página, não cabe sem mudar baseline:
  o `semantics.json` de cada baseline guarda as ações de cada nó
  (`tool/bench/regress/baselines/silences-review-by-the-rules@core@1280x800/semantics.json`,
  a linha `silences_surface`), e o regress compara a revisão da projeção
  inteira (`tool/bench/regress/src/compare.rs:74` a `96`): as 14
  baselines da Silences mudariam. mudaria também o menu da página, que
  lista as ações simples da Silences (`apps/uigallery/src/shell/system/menu.rs:411`
  a `432`), e a lista do leitor de tela (`apps/uigallery/tests/access_tree.rs:323`
  a `339`), fora dos meus arquivos. a shell, onde mora o tema, não entra
  nas baselines, mas está fora dos meus arquivos, e o menu dela mostra o
  estado de um item com a marca, que é o valor.
  o caminho: uma faixa de acessibilidade sob a linha de proposta, como a
  oferta da fase 3 (`apps/uigallery/src/views/apps/silences/generative/offer.rs:1`
  a `9`), projetada só enquanto está aberta, então nada do que a página
  projeta hoje muda. a pessoa abre escrevendo "accessibility" ou
  "acessibilidade" na linha e pedindo, pelo mesmo `propose` da fase 2;
  um agente que pede isso é recusado ("the accessibility preferences are
  the person's"). na faixa, "larger text", "more contrast", "denser",
  "sparser" e "done", todos `PERSON_ONLY`
  (`apps/uigallery/src/views/dispatcher/shell/mod.rs:55` a `60`): um agente é
  recusado `not_authorized` pelo `check`. cada botão liga ou desliga a
  sua. arquivos: `apps/uigallery/src/views/apps/silences/generative/access.rs`
  (novo), `generated.rs`, `generated/page.rs`, `silences.rs` e os testes
  em `silences/tests/`.

### (b) cada preferência em operações da fase 1, dentro dos tokens

- **texto maior**: `emphasize` em cada `text` e em cada `stat` da árvore
  (num template, vale para todo item). o texto sobe um degrau da rampa do
  tema (`look.rs:27` a `57`: sem estilo é `body`, vai a `title`; do topo,
  `h4`, não passa); o stat sobe o tamanho do valor (`base_m`, `title`,
  `headline`, `look.rs:61` a `65`), e o rótulo dele cresce junto
  (`rust/engine/src/widgets/content/stat.rs:72` a `79`). a escada leva o estilo
  com peso do degrau (`look.rs:47` a `57`), então o texto também fica
  mais forte. o que não alcança: badge, botão, campo, select, abas,
  slider, tabela, imagem desenham o texto no tamanho do componente, e
  nenhum lê `typography` (`rust/screen/src/draw.rs:150` a `212`); o
  que já está no topo fica.
- **mais contraste**: `restyle` para o papel `text`, a tinta mais forte
  que todo tema desenha: nos quatro temas, `colors.text` passa AA sobre
  as cinco superfícies que a 75 mede (nenhum par de `colors.text` em
  `KNOWN`, `rust/engine/src/styling/theme/tests/contrast.rs:192` a `398`). o
  IR não sabe o tema (os tokens são nomes, `rust/plevir/src/token.rs:5` a
  `8`), e a mesma superfície desenha em qualquer um dos quatro, então a
  troca é a mesma em todos, e o teste mede. o que vai: um `text` em
  `text_dim` ou `text_mid` (no core as duas são `#7B7B7B`, 4,13 sobre o
  papel, `tests_contrast.rs:196` e `:197`). a Silences não tem nenhum:
  nem as regras nem o Claude da fixture pintam texto
  (`rust/plevir/tests/fixtures/silences-review-claude.ir.json`), e os
  pares que falham nela são de componentes. dois deles cabem num token,
  e a fase ganha as duas linhas, aditivas: o `restyle` passa a aceitar o
  fundo de um badge (um dos papéis que o materializador já lê como
  intenção do formato preenchido, `rust/screen/src/token.rs:65` a
  `73`; o texto dele vem em contraste com o fundo,
  `rust/engine/src/widgets/widget.rs:164` a `170`), e a tinta de um slider, de
  uma tabela e de uma imagem, que o próprio host desenha em `text_mid` e
  `text_dim` (`draw.rs:351` a `361`, `:382` a `390`, `:448` a `457`) e
  passa a ler de `props.color`. assim "mais contraste" enche o badge sem
  fundo (`accent`) e passa slider, tabela e imagem a `text`. o que nenhum
  token faz passar: o rótulo do stat (o comps desenha em `text_dim`,
  `stat.rs:164`, e não recebe cor), o cabeçalho da tabela
  (`glass.text_faint`, `rust/engine/src/widgets/content/table.rs:354`), o texto
  num papel de intenção (success, danger, warning, info: a cor é o
  sentido; no core o verde dá 2,41), o texto sobre um fundo escuro
  (`accent_dim` no core é `#323232`, e `text` também é escuro), o rótulo
  de botão (a receita do tema; o warning do core dá 3,37). mudar isso é
  mudar cor de tema ou o comps.
- **densidade**: `density` com `denser` ou `sparser`, uma vez: todo gap
  de stack e grid e todo padding que tem token andam um degrau
  (`look.rs:146` a `202`). o que não tem token é do componente e fica;
  nas pontas, para.

a escolha de cada operação é pura e mora no `uir`
(`rust/plevir/src/transform/prefer.rs`, novo): as preferências viram a
lista de operações sobre a superfície como ela está, e cada uma é
aplicada pelo `Adapted::apply` de sempre, verificada; a que é recusada
(o topo da escada, nada a mover) fica de fora, e a lista diz qual.

### (c) quando aplicam

- ao abrir (`open_under`, `host.rs:180` a `197`) e numa forma nova
  (`transform_under`, `:216` a `228`): a forma com as preferências é
  feita antes de `Surface::materialize` (antes do `take_form`, na forma
  nova), então o primeiro frame já é ela; nenhuma transição, e a pessoa
  nunca vê a superfície sem a preferência.
- quando a pessoa liga ou desliga com uma aberta: cada superfície do host
  toma a forma nova como uma mudança (reconciliada por `SemId`, com as
  transições de uma mudança, nenhuma com reduced motion).

### (d) desligar

o histórico é pilha: desfazer só o que está no topo. se a preferência
entrou e um agente escondeu um nó depois, desfazer a preferência pelo
histórico desfaria o esconder antes. a saída, sem mudar a fase 1: as
preferências são a última camada. o host guarda a superfície como foi
pedida (a forma e as mudanças pedidas, sem preferência) e a desenhada, que
é a pedida com as operações das preferências por cima, refeitas depois de
toda mudança. desligar é refazer sem aquela: o que foi pedido não é
tocado, e o resto volta byte a byte por construção. uma operação pedida
(pela pessoa, pelas regras, por um agente, pelo Claude) é aplicada sobre
a pedida, que é também a que o proponente e o modelo veem.

o custo: cada mudança refaz as operações das preferências (poucas, sobre
um IR pequeno, cada uma verificada). uma operação cuja aparência a
preferência cobre por cima pode ser recusada por já estar feita na pedida
(um `restyle` para o papel que a pedida já tem): a preferência da pessoa
ganha. a outra saída, refazer da materialização sem as entradas da
preferência, custa o mesmo e perde a preferência num nó mostrado depois
dela.

### (e) o ator e o histórico

a operação de uma preferência entra em nome de quem a ligou: a pessoa
(`Actor::Human`), a tecnologia assistiva (`Actor::Assistive`) ou um teste
(`Actor::Test`), os que o `PERSON_ONLY` deixa. nenhuma variante nova de
`Actor` (`rust/accord/src/requests.rs:23` a `28`) e nenhum tipo do `sem`
muda. o host recusa um agente também, além da política. o histórico
mostra a preferência no `Change`, com um campo novo, `preference`, que
diz qual (`rust/screen/src/host/mod.rs:100` a `104`): `changes` dá as
mudanças pedidas, na ordem, e depois as das preferências; `last_ask`,
`undo` e `undo_asked` só veem as pedidas ("desfazer" não tira uma
preferência; desligar tira).

privacidade: a projeção de uma superfície não tem `props`
(`rust/screen/src/project.rs:1` a `27`), então ela é a mesma com ou
sem preferência; a faixa só existe aberta, e os botões não têm estado. o
modelo vê a superfície pedida: a mudança já pergunta sobre `current`
(`apps/uigallery/src/views/apps/surface/offering/changing.rs:152`),
que passa a ser a pedida, e a proposta de uma superfície nova passa a
mandar o mesmo (`generated/page.rs:249` a `254` e
`surface_page/page.rs:265` a `270` leem hoje `Surface::verified`, a
desenhada). assim o prompt é o mesmo, byte a byte, com e sem preferência,
e um modelo nunca copia o texto maior para a forma nova (que a preferência
subiria de novo).

### (f) o `promote` do template no `uir`

hoje o host empresta ao `promote` um registro onde o template é mais um
membro da coleção (`registry`, `rust/screen/src/host/mod.rs:116` a `147`,
usado em `apply_all`, `:310`), porque o `promote` procura a ação pelo
alvo exato (`rust/plevir/src/transform/view.rs:154`, `caps.action`). o
mínimo no `uir`: quando o alvo carrega o item uma vez (`Pattern::of`,
`rust/plevir/src/item.rs:44` a `54`), a declaração é a do primeiro nó do
registro que cabe no template e declara a ação, a regra com que o
verifier liga um template (`rust/plevir/src/verify/bind.rs:167` a `213`):
todo nó que cabe declara igual, conferido depois, como sempre. o
`registry` do host sai. os testes da fase 3
(`rust/screen/tests/promote.rs`, a composta da Silences) não mudam.

### nenhuma parada

nenhum tipo do `sem` muda, o protocolo v1 não muda, nenhuma cor de tema
muda, nenhuma leitura nova do sistema, e o desligar não pede mudar a
fase 1 (as preferências são uma camada do host). nenhum baseline existente
muda: a faixa só é projetada aberta, e as superfícies das baselines não
têm preferência.

## handoff da fase 4 (2026-09-25)

```text
Goal: task-70 fase 4: a pessoa liga uma preferência (texto maior, mais
      contraste, mais ou menos denso) e toda superfície gerada aberta e
      toda que abrir depois muda por operações verificadas da fase 1,
      dentro dos tokens, em nome dela, no mesmo histórico; desligar
      devolve só o que ela mudou; e o promote de um template no uir
Status: pronto para o orquestrador; adição ao ADR proposta
```

### commits

- 921e159a task-70: the probe of phase 4, accessibility as transformation and the promote of a template in uir
- 490fe38d task-70: the promote of a template finds the action on its items in uir, and the host lends no registry
- d39385b7 task-70: each preference of accessibility is operations by token over the surface, verified, and restyle reaches a badge's fill and the ink a slider, a table and an image are drawn in
- 89a97fa0 task-70: the host draws the person's preferences as the last layer, from the first frame, and turning one off leaves what others asked; the thread's reduced motion makes a change instant
- 7b34736f task-70: only the person opens the accessibility band and turns a preference on; the page projects the same and a model is shown the surface as it was asked
- c8c16ed1 task-70: more contrast measured pair by pair in core as task-75 measures, and the stat's label no token reaches listed
- 8124e8a7 task-70: the demo, the review by the rules with larger text and more contrast, in the regress with its image
- o commit deste texto e do ADR

### a sonda

está na seção "sonda" da fase 4, acima, de (a) a (f). nenhuma condição
de parada aconteceu: nenhum tipo do `sem` mudou, o protocolo v1 não
mudou, nenhuma cor de tema mudou, nenhuma leitura nova do sistema,
nenhum baseline existente mudou, nenhuma dependência nova, o disco ficou
em 16 GB livres.

um arquivo fora da lista, só aditivo: `rust/plevir/src/lib.rs` ganhou o
`pub use` de `Preferences`, `Preference`, `Preferred` e `Applied` (sem
ele o `surface` não vê os tipos novos de `uir::transform`).

o que a sonda achou e mudou o desenho: uma ação `PERSON_ONLY` sempre
projetada na página mudaria as 14 baselines da Silences (o regress
compara a projeção), o menu da página e a lista do leitor; por isso a
faixa só existe aberta. e nem as regras nem o Claude da fixture pintam
texto em tinta secundária: os pares que falham na revisão são de
componentes, e dois deles couberam num token (o fundo do badge, a tinta
de slider, tabela e imagem).

### cada preferência e as operações

o código: `rust/plevir/src/transform/prefer.rs` (`Preferences`,
`Preference`, `Preferences::over`), `rust/plevir/src/transform/look.rs`
(o `restyle` com as duas partes novas), `rust/screen/src/draw.rs` (a
tinta de slider, tabela e imagem lida de `props.color`).

| preferência | operação | o que alcança | o que não alcança |
| --- | --- | --- | --- |
| texto maior | `emphasize` em todo `text` e todo `stat` | o texto um degrau da rampa (sem estilo: `body` para `title`), o valor do stat um tamanho (o rótulo cresce junto); num template, todo item | badge, botão, campo, slider, tabela, imagem (o texto no tamanho do componente); o que já está no topo (`h4`) fica, e a lista diz |
| mais contraste | `restyle` para `text` de um `text` em `text_dim`/`text_mid` e de todo slider, tabela e imagem; `restyle` do fundo de um badge sem fundo para `accent` | o texto secundário, o rótulo e o valor do slider, as linhas da tabela, o rótulo da imagem, o badge (formato preenchido, o texto em contraste) | o rótulo do stat (`text_dim` do comps), o cabeçalho da tabela, o texto numa cor de intenção, o texto sobre fundo escuro, o rótulo de botão |
| mais denso / mais espaçado | `density` uma vez | todo gap de stack e grid e todo padding com token | o que não tem token; as pontas |

testes: `rust/plevir/tests/prefer.rs` (cada preferência sozinha e todas
juntas: só operações de aparência, cada resultado verificado de novo,
nenhum id sai, o JSON do resultado lido de volta só com tokens, as
inversas da última para a primeira devolvem a superfície byte a byte; o
que cada uma alcança e deixa; o `restyle` novo), `rust/plevir/tests/transform.rs`
(os de antes, iguais).

### o contraste, antes e depois, par a par

medido como a 75 mede (`apps/uigallery/src/views/apps/silences/tests/generative/contrast.rs`):
a revisão desenhada, cada texto com a tinta composta em luz linear sobre
os fundos desenhados embaixo dele e o `bg` da página, luminância relativa
do WCAG, 4,5 (3 para texto grande). no core; o teste confere cada razão.

| par (texto sobre o que ele pisa) | antes | com mais contraste |
| --- | --- | --- |
| texto do IR (faixa do corte, título, arquivo) `#121212` sobre `#FCFCFC` | 18,26 | 18,26 |
| valor do stat `#121212` sobre `#FCFCFC` | 18,26 | 18,26 |
| rótulo de botão `#121212` sobre `#E2E2E2` | 14,46 | 14,46 |
| badge "pending" (vidro) `#7B7B7B` sobre `#F8F7F7` | 3,96 | 17,96 (`#FAFAFC` sobre `#121212`, preenchido) |
| rótulo de imagem sem quadro (o vídeo, "frame", "original frame", "result frame", "silence frame") `#7B7B7B` sobre `#FCFCFC` | 4,13 | 18,26 |
| rótulo do slider (Claude: "threshold (dB)", "minimum silence (s)") `#7B7B7B` sobre `#FCFCFC` | 4,13 | 18,26 |
| valor do slider (Claude: "-30", "0.5") `#121212` sobre `#FCFCFC` | 18,26 | 18,26 |
| rótulo do stat (regras: "start", "end", "duration", "shown", "value", "total", "applied", "rejected", "pending", "cut_secs"; Claude: "duration (s)", "total", "applied", "rejected", "pending", "seconds cut") `#7B7B7B` sobre `#FCFCFC` | 4,13 | 4,13, listado |

o par que nenhum token faz passar: o rótulo do stat. o comps desenha em
`colors.text_dim` (`rust/engine/src/widgets/content/stat.rs:164`) e não recebe
cor, e nenhum papel do IR chega nele. passar pede uma cor de rótulo no
`Stat` do comps ou o `text_dim` do core mais escuro: comps ou tema,
decisão do brenner. nenhuma cor de tema mudou.

### desligar

as preferências são a última camada do host
(`rust/screen/src/host/mod.rs`, `rust/screen/src/host/prefer.rs`): o
host guarda a superfície como foi pedida e desenha as preferências por
cima, refeitas depois de toda mudança. desligar refaz sem ela. teste
(`rust/screen/tests/prefer.rs`,
`turned_off_what_it_changed_goes_back_and_what_an_agent_asked_stays`):
texto maior e mais contraste ligados, um agente esconde
`review.detection`, a pessoa desliga uma (a outra fica) e depois as
duas: o nó continua escondido, o histórico tem só o `hide` do agente, e
a superfície é a revisão com o `hide`, byte a byte. também: uma
superfície abre já com elas, antes do primeiro frame (nenhum frame de
transição); ligar com uma aberta muda na hora; um nó mostrado depois
pega a preferência; "desfazer" tira só o pedido; uma forma nova pega as
preferências.

### o ator e o histórico

`SurfaceHost::prefer(preferences, by, app)` recusa um agente
(`SurfaceError::NotThePerson`). as operações entram em nome de quem
ligou (a pessoa, a tecnologia assistiva ou um teste), sem variante nova
de `Actor`. o `Change` ganhou `preference: Option<Preference>`:
`changes` dá as pedidas e depois as das preferências; `last_ask`,
`undo` e `undo_asked` só veem as pedidas.

na página (`apps/uigallery/src/views/apps/silences/generative/access.rs`,
novo; `generated.rs`, `generated/page.rs`, `silences.rs`): a pessoa
escreve "accessibility" ou "acessibilidade" na linha e pede; a faixa
abre sob a linha com "larger text", "more contrast", "denser",
"sparser" e "done", todos `PERSON_ONLY`, sem estado, rótulo fixo; a
preferência ligada aparece escura. testes
(`apps/uigallery/src/views/apps/silences/tests/generative/access.rs`): um
agente que pede "accessibility" falha com o motivo; um agente que aperta
um botão é recusado `not_authorized`; a pessoa liga e desliga, e o
histórico é dela; pelo port de verdade, com a faixa aberta, o agente vê
a faixa sem estado, os botões voltam `requested, rejected` e "accessibility"
pela linha termina `failed`; nada liga.

privacidade: a página projeta o mesmo, revisão igual, com e sem
preferências (a projeção da superfície não tem aparência); e o prompt do
Claude (o falso) é o mesmo, byte a byte, com e sem preferências, para
uma mudança e para uma superfície nova: a página propõe sobre `current`,
a superfície como foi pedida (`generated/page.rs` e
`surface_page/page.rs` liam `Surface::verified`, a desenhada; o teste
falha se voltar a ler).

### o reduced motion

`rust/screen/src/transition.rs`: o `Motion` lê também
`engine::animation::reduced_motion()` e não cria transição; o primeiro
frame depois de uma mudança já é a forma nova. teste em
`rust/screen/tests/transition.rs` (sem a correção, o primeiro frame
desenha o nó que saiu inteiro: conferido).

### o `promote` no `uir`

`rust/plevir/src/transform/view.rs`, `declared`: no template de um item,
a ação do primeiro nó do registro que cabe e a declara, com o template
como alvo; o verifier confere o resto. o `registry` saiu do host. teste
novo em `rust/plevir/tests/transform.rs`; os da fase 3
(`rust/screen/tests/promote.rs`, `apps/uigallery/tests/composites.rs`
e os de `silences/tests/composites.rs`) passam sem mudança.

### a demo

o estado novo "Silences, review by the rules, larger text and more
contrast" (`tool/bench/regress/src/state.rs`): a revisão pelas
regras, "accessibility" pela linha, "larger text", "more contrast" e
"done" pela faixa, como a pessoa (o harness é um ator de teste). em
teste também:
`the_demo_larger_text_and_more_contrast_over_the_review_by_the_rules`.

| | imagem |
| --- | --- |
| antes (baseline da fase 2) | `tool/bench/regress/baselines/silences-review-by-the-rules@core@1280x800/frame.png` |
| depois | `tool/bench/regress/baselines/silences-review-by-the-rules-larger-text-and-more-contrast@core@1280x800/frame.png` |

no depois, a faixa de cada corte ("0:03.0 to 0:05.0") passa de 15 para
20 px, os números dos cortes e das contagens sobem um tamanho, o selo
"pending" fica preenchido escuro com o texto claro, e os dois cortes
continuam dentro de 1280x800. os rótulos dos stats continuam cinza (o
par listado). a linha mostra "accessibility", o que a pessoa escreveu.

### baselines

- novo (1): `silences-review-by-the-rules-larger-text-and-more-contrast@core@1280x800`,
  com imagem.
- nenhum baseline existente mudou.

### contagem do gate e o regress

| | workspace | perna urnaui | total |
| --- | --- | --- | --- |
| antes, `main` @ 91bade03 (o número do briefing) | | | 2990 passam, 27 ignorados |
| depois, esta branch @ 8124e8a7 (`./tool/tasks/fullcheck.sh` offline, saída 0, 02:35:14 a 02:37:33) | 2925 passam, 0 falhas, 26 ignorados (156 suítes) | 85 passam, 0 falhas, 1 ignorado (6 suítes) | 3010 passam, 27 ignorados |

20 testes novos e três suítes (`uir`: `prefer`, 5, e 1 em `transform`;
`surface`: `prefer`, 6, e 1 em `motion`; a lib do `showcase`: 5 em
`access` e 2 em `contrast`). o gate inteiro verde, com a perna urnaui e
o check wasm com `-D warnings`. `df -h /`: 16 GB livres antes e depois.
`cargo run --offline -p regress -- check` (com o
`main.rs` tocado): 176 baselines, 176 iguais, nenhum frame desenhado,
nenhuma GPU aberta (os 175 de antes iguais, 1 novo). todo cargo com o
`PLEV_CLAUDE` apontando para o stub que só falha; nenhum `claude -p`
chamado (o teste do prompt usa o `claude` falso do repositório).

### linha de ADR para o índice (o [`adr/`](../../adr/README.md) não está na minha lista)

| 2026-09-25 | [adaptive-transforms-preserve-identity](../../adr/generated/0067-adaptive-transforms-preserve-identity.md) | adição proposta | fase 4: `uir::Preferences` (texto maior, mais contraste, densidade) viram operações da fase 1 por token, cada uma verificada (`emphasize` em texto e stat; `restyle` para `text` e badge preenchido, com o `restyle` alcançando o fundo do badge e a tinta de slider, tabela e imagem; `density`); o host desenha as preferências como a última camada, desde o primeiro frame, e desligar deixa o que outros pediram; só a pessoa liga (`SurfaceHost::prefer` recusa agente; a faixa `PERSON_ONLY` da Silences, projetada só aberta, sem estado); a projeção e o prompt são os mesmos com e sem preferência; contraste medido como a 75, o rótulo do stat listado; o reduced motion da thread sem transição no primeiro frame; o `promote` de template no `uir`, sem o `registry` do host (task-70 fase 4) |

### o que fica aberto

- ler do sistema "aumentar contraste" e o tamanho de texto: nada foi
  lido. o caminho seria um irmão de `engine::accessibility::motion` (a
  leitura do reduced motion) e a view passando o valor para
  `SurfaceHost::prefer`, como a ponte passa o reduced motion; a chave de
  cada um tem que ser achada na máquina.
- a prova com o Claude de verdade (da fase 2) é do orquestrador.
- o lugar da pessoa é só a Silences. o host da página Surface tem
  `prefer` e não tem a faixa; um lugar da galeria inteira (a shell, onde
  mora o tema) serviria as duas, mas o menu da shell mostra o estado com
  a marca, que é o valor: decisão.
- o rótulo do stat fica em 4,13 no core com mais contraste (comps ou
  tema).
- o prompt de mudança (`rust/offers/src/operations.rs`) não fala das duas
  partes novas do `restyle`.
- um leitor de tela ouve "larger text, button" e não se está ligado: é a
  troca que a promessa de privacidade pede; quem usa essas preferências
  vê o resultado na tela.
- uma operação pedida cuja aparência a preferência cobre pode ser
  recusada por já estar feita na superfície pedida (um `restyle` para o
  papel que ela já tem): a preferência da pessoa ganha.
- a adição ao ADR está como proposta; o índice é do orquestrador.
