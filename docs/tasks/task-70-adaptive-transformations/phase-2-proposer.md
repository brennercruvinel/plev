---
id: task-70/phase-2-proposer
title: "task-70: phase 2, the transformation proposer"
project: plev
type: section
status: reference
parent: task-70
date: 2026-09-24
date_basis: text
topics: [llm, agents, ir, generated-surfaces]
relations: [task-70/phase-1-operations, adr-0067, task-58, task-69]
source: docs/log/steps/task-70-adaptive-transformations.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
evidence: [793adf98, c01021a4, bdb828ea, 66b5abd5, 38426a12, e80d33dd, 19fea13d, 5d1ca12f]
---

# phase 2: the transformation proposer

Records phase 2 of task-70: an intent in words, with an open surface, turns into phase 1 operations, through the rules or through Claude, and each operation is verified and applied as a request by the dispatcher, with the actor in the history and undo through the same path. Also included are the policy kept per surface in the host, the emphasis of `stat` (via `typography`) and of `badge` (via the background), and the three gaps from checkpoint b in the generated review. The proof with the real Claude was left to the orchestrator, pending the release of spending.

branch `task/70b-transform-proposer`, sobre a `main` em `3dde4976`. a
fase 2 é isto: uma intenção em palavras, com uma superfície aberta, vira
operações da fase 1 (pelas regras, sem modelo, ou pelo Claude), cada uma
verificada e aplicada como um `Request` pelo dispatcher do app, com o
ator no histórico e desfazer pelo mesmo caminho; a política de cada
superfície guardada pelo host; ênfase em `stat` e `badge`; e as três
lacunas do checkpoint b na revisão gerada.

## sonda (antes do primeiro tipo, 2026-09-24)

### (a) o proponente de transformação

o `Proposer` (`rust/offers/src/lib.rs:74` a `76`) devolve um `Ir`, e
a saída é `propose_verified` (`:82` a `88`), que só deixa sair
`VerifiedIr`. a task-69 usa os dois pelas páginas
(`apps/uigallery/src/views/apps/silences/generative/page.rs:218`,
`apps/uigallery/src/views/apps/surface/page.rs:244`, e o worker do
Claude em `surface_page/asking.rs:260`). mexer no `Proposer` mexeria em
tudo isso.

o desenho: um trait irmão no mesmo crate, sem tocar no `Proposer`.

- `Adapt<'a>`: o que se pede a um transformador. a intenção, o registro
  (`Capabilities`), o snapshot, a superfície como está (`&Adapted`, o
  `VerifiedIr` e o que está de lado, `rust/plevir/src/transform/mod.rs:197`),
  a política com que ela foi verificada e quem pediu (`&Actor`, que o
  `promote` lê, `mod.rs:257` a `261`).
- `Transformer::transform(&mut self, &Adapt) -> Result<Adaptation,
  ProposeError>`, onde `Adaptation` é `Ops(Vec<Op>)` ou `Undo` (desfazer
  não é uma operação: é o host que sabe o que foi a última mudança).
- `transform_verified(t, ask) -> Result<Transformed, ProposeError>`:
  pede ao transformador e aplica cada operação, em ordem, sobre o
  `Adapted` da pergunta (`Adapted::apply`, que verifica,
  `mod.rs:241` a `264`); devolve `Changed { ops, adapted }` só quando
  todas aplicaram e o resultado é um `Adapted` verificado, ou `Undo`.
  lista vazia é erro (nada a mudar). é o par do `propose_verified`:
  nada sai sem passar pelo verifier.
- `Rules` e `Llm<M>` implementam os dois traits. o `Model` é o mesmo.

### (b) a gramática das regras

`words.rs` lê as intenções de superfície (`Part`, `Form`, "compare",
"everything", `rust/offers/src/words.rs:17` a `230`). as de mudança
entram num leitor irmão, com o mesmo `fold` (caixa, acento, plural,
`:132`) e as mesmas duas línguas:

- **menos** ("too much", "cluttered", "busy", "simpler", "less";
  "demais", "poluído", "mais simples", "menos"): esconde o que não é da
  tarefa, agrupa, e aperta o espaçamento (`density denser`).
- **comparar** ("compare", "comparar") só com uma palavra de dificuldade
  ("can't", "cannot", "hard"; "não consigo", "difícil", "dificuldade"):
  "compare the original with the cuts" continua sendo a comparação de
  vistas da mídia da 69 (`words.rs:106` a `116`, `rules.rs:85` a `99`).
- **maior** ("bigger", "larger", "grow"; "maior", "maiores", "grande",
  "aumentar"), e "não consigo ler/enxergar" ("can't see", "can't read";
  "enxergar", "ler" com dificuldade): `emphasize`.
- **desfazer** ("undo", "revert"; "desfazer", "voltar"): a última
  mudança pedida, inteira.

o resto da intenção: palavras de enchimento das duas línguas, ou
palavras do app (as mesmas de `Parts::words`, `parts.rs:257`), que
escolhem o alvo de "maior" e "comparar". uma palavra que não é nada
disso, ou nenhuma palavra de mudança, e a intenção não é mudança: segue
pelo caminho da 69 (uma superfície nova, `Rules::propose`).

como as regras escolhem o alvo, pela estrutura e pelo registro, nunca
por nome de app:

- **a tarefa** é o que liga ação (um nó com `bind.actions`, ou que tem
  um assim abaixo, um `repeat` inteiro). **contexto** é um bloco que
  liga estado e nenhuma ação na sua subárvore (um vídeo grande, um
  cartão de contagens). "menos" esconde cada bloco de contexto,
  descendo da raiz só pelos contêineres que guardam a tarefa, nunca
  dentro de um `repeat` (o item é a tarefa inteira), nunca filho de
  `split`; nada se a superfície não liga ação nenhuma. depois agrupa:
  num contêiner que mistura leituras e controles em fila (dois controles
  ou mais e uma leitura ou mais), os controles vão para um grupo novo
  (`<pai>.actions`, que no template carrega o `[{item}]` do pai). por
  fim `density denser`. cada operação é tentada em sequência sobre o
  `Adapted`; a recusada é pulada; nenhuma aplicada é erro com o motivo.
- **comparar**: um nó ligado a um `List`/`Set` mostrado como `table` vira
  gráfico de barras quando todo valor da lista lê como número e a lista
  não é a chave dos filhos do seu nó (a estrutura de `list.rs:1` a `15`:
  uma lista de itens é de ids, não de medidas); um gráfico vira tabela.
  uma lista com `repeat` não troca (a sonda da fase 1: os ids do
  template sairiam sem `hide`, `view.rs:70` a `75`). nada que caiba:
  recusa, dizendo o que há e por que não troca. **na Silences não cabe**,
  como a fase 1 achou (o único `List` do app são os ids dos cortes; a
  duração é um `num` por corte): a demo mostra a recusa com o motivo, e
  a troca é provada numa superfície com uma lista de números.
- **maior**: `emphasize` em cada `stat` e em cada `text` ligado a um
  `int` ou `num` (os que as palavras do app nomeiam, quando nomeiam
  algum); num template vale para todo item; recusa no topo da escada.

### (c) o Claude

`Llm<M>` (`rust/offers/src/llm.rs:36` a `118`) implementa o
`Transformer` com um prompt próprio, num arquivo novo ao lado de
`prompt.rs` (que já tem 341 linhas), reaproveitando o que é igual: as
regras do verifier (`prompt.rs:21` a `67`), os componentes (`:119` a
`173`) e o registro (`:186` a `229`, onde um valor secreto já sai como o
tamanho que o `sem` guarda, `rust/accord/src/redactor.rs:1` a `15`, e uma
imagem é um texto `<id>@<segundos>`, nunca pixel). o que é novo: a
superfície aberta (o JSON do IR e os nós de lado, por id e pai), as
nove operações no JSON da fase 1 com o que cada uma faz e recusa, as
regras de identidade (todo id na árvore ou de lado, só `hide` tira, o
`into` novo, `[{item}]` no template, id de instância recusado), as
famílias do `swap_view`, e a resposta: um objeto `{"ops": [...]}`, sem
prosa.

a resposta é lida do primeiro `{` ao último `}` (como `llm.rs:78` a
`85`), e cada operação é tentada em ordem sobre o `Adapted`, verificada;
a primeira recusa, com o índice e o motivo, volta ao modelo junto com a
resposta ("attempt N was refused", a mesma frase que o `claude` falso
lê, `rust/claude/tests/fakes/claude`), no máximo
`Llm::ATTEMPTS` vezes; esgotado, o erro traz os motivos da última e a
superfície não muda. o `SYSTEM` (`rust/claude/src/lib.rs:55`)
já diz "responda exatamente o que ele pede, na forma que ele pede":
serve às operações sem mudança.

### (d) o caminho pela página

o desenho que a pergunta sugeria era uma ação nova (`silences_surface.adapt`
e `undo`). não cabe sem mudar baseline fora do que o item 7 deixa: o
`regress` compara a projeção antes dos pixels (`tool/bench/regress/src/compare.rs:74`,
`Snapshot::diff`), e um nó novo na linha, ou uma ação nova num nó dela,
muda o `semantics.json` de toda baseline que mostra a linha. a linha é
projetada em `surface_page/proposing.rs:340` a `394`: com a superfície
aberta, as baselines da revisão do Claude (fixture), a adaptada da fase
1 e a revisão pelas regras mudariam, e sem superfície as duas da tela
fixa (`silences@core`, `silences@hoff`) também, se o nó aparecesse
sempre.

o caminho que não muda nada disso é o que o ADR da fase 1 já escreveu
("through the same `Proposer` path"): o mesmo `propose`
(`proposing.rs:396` a `415`, `generated/page.rs:194` a `248`,
`surface_page/page.rs:221` a `273`). com uma superfície aberta, a
intenção é lida primeiro como mudança:

- pelas regras: `transform_verified(Rules)`; `Changed` aplica as
  operações de uma vez pelo host, com o ator da request; `Undo` desfaz a
  última mudança pedida; "não é mudança" segue para `propose_verified`
  (a 69, uma forma nova no lugar); outra recusa deixa a superfície como
  está e a linha diz por quê.
- pelo Claude: se o leitor da (b) diz que é mudança, o worker do Claude
  (`surface_page/asking.rs:159` a `276`) roda `transform_verified(Llm)`
  fora da thread do app, o `propose` responde `Execution::Started`, a
  linha mostra a tentativa e os segundos, e o `settle` aplica as
  operações e encerra a request com as tentativas, os segundos e o custo
  (como a 69). "desfazer" não chama o Claude. se não é mudança, é a 69.
- um agente pede pelo port como pede o `propose`: `set_value` na
  intenção e `propose`. o protocolo não muda; o ator da request vai no
  `Change.by`. o desfazer é a intenção "undo"/"desfazer", pelo mesmo
  `propose`.

### (e) a política por superfície

o host recebe a política a cada `apply` e `undo`
(`rust/screen/src/host/mod.rs:201` a `259`) porque o `VerifiedIr` não
guarda a sua, e o `VerifiedIr` é do verifier (proibido). o host guarda
a política de cada superfície ao abrir e ao trocar de forma:
`open_under(ir, app, policy)` e `transform_under(ir, app, policy)`;
`open` e `transform` (`host.rs:123` e `:143`) ficam com a assinatura e
guardam `GenerationPolicy::default()`, a política com que a página
Surface verifica as suas formas (`surface_page.rs:190` e `:268`, fora
da minha lista), então ela não muda. `apply(id, op, by, app)` e
`undo(id, app)` usam a da superfície; `policy(id)` a lê. as páginas
abrem uma proposta com a política da proposta (`GenerationPolicy::by`
do autor).

e o histórico por pedido: `apply_all(id, ops, by, app)` aplica a lista
inteira sobre o `Adapted` antes de tocar na superfície (tudo ou nada),
toma a forma uma vez (uma reconciliação, uma transição) e guarda cada
`Change` com a fronteira do pedido; `undo_asked(id, app)` desfaz o
último pedido inteiro. `apply` é um pedido de uma operação; `undo`
continua desfazendo uma.

### (f) `stat` e `badge`

o materializador desenha o `stat` com `Stat::new(valor, rótulo)` sem
tamanho (`rust/screen/src/draw.rs:83` a `86`, `:139` e `:305`), e o
`badge` com `Badge::tag(texto)` (`:138` e `:304`); o `emphasize` recusa
os dois (`rust/plevir/src/transform/look.rs:108` a `112`). o comps já tem
o que falta, sem mudança:

- `Stat` tem `StatSize` `Sm`, `Md`, `Lg`, com o valor em `base_m`,
  `title` e `headline` (`rust/engine/src/widgets/content/stat.rs:15` a `23` e
  `:61` a `68`). o token é o `typography` do nó, o mesmo da rampa do
  texto: até `base` é `Sm`, `title` (o de hoje, sem token) é `Md`,
  `headline` e acima é `Lg`. a escada do `emphasize` num stat: `base_m`,
  `title`, `headline`, topo.
- `Badge` tem dois formatos, `Tag` (o vidro) e `Count` (preenchido com
  a cor da intenção, texto em contraste,
  `rust/engine/src/widgets/content/badge.rs:17` a `24` e `:88` a `101`). o token
  é o `background` do nó: sem ele é `Tag`; com um papel de intenção
  (`accent`, `success`, `danger`, `info`) é o `Count` com essa intenção.
  a ênfase de um badge é ele preenchido (`accent`); o topo é ter fundo.
  maior de tamanho o comps não faz, e mudar o comps é parada: a ênfase
  do badge é de peso, não de tamanho.

o `emphasize` ganha as duas linhas em `look.rs`: é a falta que a sonda
mostra no `rust/plevir/src/transform/**`. o vocabulário e o verifier não
mudam (nenhuma regra olha `props`, só o tipo).

### as três lacunas do checkpoint b

1. o `READS` (`rust/offers/src/prompt.rs:73` a `83`) ganha: revisar
   itens é ver o que cada um é (a imagem do item, quando o registro tem
   uma, e o seu estado), e "só o que a intenção precisa" não tira isso.
   sem nome de app.
2. a linha do item das regras (`rust/offers/src/rules.rs:381` a
   `440`, sobre `list.rs:153` a `242`) põe todo texto e número numa fila
   só, sem rótulo. passa a separar por sentido: o que o item é (imagem,
   texto que não é número, selo), os números (`stat` pequeno, com a
   chave do estado como rótulo, `typography: base_m`, como os `Count` já
   fazem com a chave, `parts.rs:225`), as ações (checkbox, botões). cada
   sentido com dois nós ou mais vira um grupo, com id do template
   (`[{item}]`); uma linha com um nó por sentido (o todo: texto e
   checkbox) fica como está.
3. o estado de poucos valores: um estado de texto que duas ações ou mais
   sem parâmetro do mesmo item leem, e só ele (as `reads` da declaração,
   como o `flip` do checkbox em `list.rs:185` a `194`), é uma decisão:
   sai como `badge`. na Silences, `decision`, que `apply_cut` e
   `reject_cut` leem; o `status` do ide não (o `stage` lê dois estados).

as baselines que mudam por isso são só as da revisão pelas regras
(`silences-review-by-the-rules@*`). nada disso toca a fixture do
Claude.

### nenhuma parada

nada pede mudar o protocolo v1, o vocabulário ou o verifier do IR, nem o
comps. a mudança em `rust/plevir/src/transform/look.rs` é a da (f).

## handoff da fase 2 (2026-09-24)

```text
Goal: task-70 fase 2: uma intenção em palavras, com uma superfície
      aberta, vira operações (pelas regras ou pelo Claude), verificadas e
      aplicadas pelo dispatcher do app como request, com o ator no
      histórico e desfazer pelo mesmo caminho; a política por superfície;
      ênfase em stat e badge; as três lacunas do checkpoint b
Status: pronto para o orquestrador; adição ao ADR proposta
```

### commits

- 793adf98 task-70: the probe of phase 2, the transformation proposer
- c01021a4 task-70: a transformer answers operations, and only a verified surface comes out
- bdb828ea task-70: the rules read a change of the surface shown, in english and portuguese
- 66b5abd5 task-70: the Claude answers a change as operations, and a refused one goes back with why
- 38426a12 task-70: a change asked through the row's propose, by the rules or the Claude, and the policy kept per surface
- e80d33dd task-70: a stat grows by its typography and a badge fills by its background
- 19fea13d task-70: the review by the rules shows what each item is, its numbers named and its decision as a badge
- 5d1ca12f task-70: the demo, too much information and can't compare over the open review, in a test and in the regress
- o commit deste texto e do ADR

### a sonda

está na seção "sonda" da fase 2, acima, de (a) a (f), com as lacunas.
nenhuma condição de parada aconteceu: nada pediu mudar o protocolo v1,
o vocabulário ou o verifier do IR, nem o comps; nenhum baseline fora da
revisão gerada mudou. a única mudança em `rust/plevir/src/transform/**`
é a da (f): `look.rs` ganhou as escadas de stat e badge.

um desvio do desenho que a pergunta sugeria, na (d): não há ação nova
(`silences_surface.adapt`, `undo`). um nó ou uma ação nova na linha muda
o `semantics.json` de toda baseline que mostra a linha (o `regress`
compara a projeção antes dos pixels), inclusive as da revisão do Claude
e as duas da tela fixa. o pedido é o mesmo `propose` (o que o ADR da
fase 1 já dizia: "through the same `Proposer` path"), e o desfazer é a
intenção "undo"/"desfazer" por ele.

### o proponente, a gramática e o prompt

- `rust/offers/src/adapt.rs`: `Adapt` (`:28`), `Adaptation`
  (`:57`), `Transformer` (`:66`), `Transformed` (`:72`) e
  `transform_verified` (`:98`), que aplica cada operação em ordem pelo
  `Adapted::apply` e só devolve as operações com o `Adapted` verificado
  que elas fazem; a primeira recusa sai como `ProposeError::Unapplied`
  com o lugar; nenhuma operação é `NoChange`. erros novos em
  `rust/offers/src/error.rs:55` a `80`.
- a gramática: `rust/offers/src/change.rs` (menos, comparar com
  dificuldade, maior, desfazer, em inglês e português;
  `reads_as_change` para a página). as regras por estrutura e registro:
  `rust/offers/src/reshape.rs` (`less`, `compare`, `bigger`, e o
  `impl Transformer for Rules`). nenhuma cita app.
- o prompt: `rust/offers/src/operations.rs` (o prompt da mudança, a leitura
  de `{"ops": [...]}` e o reparo com "attempt N was refused"), e o
  `impl Transformer for Llm` em `rust/offers/src/llm.rs:129`. o
  `SYSTEM` do `claude-model` serve como está (não mudou).

testes: `rust/offers/tests/adapt.rs` (a saída verificada, a recusa
com o lugar, o desfazer, a promoção conferida por quem pede),
`tests/change.rs` (as regras em inglês e português: menos sobre a
revisão do Claude e sobre o todo, comparar que troca uma tabela de
números por gráfico e que recusa na Silences com o motivo, maior em
textos de número e em stats, desfazer, e o que não é mudança),
`tests/llm_change.rs` (resposta boa aplicada, resposta recusada volta
com o motivo e o modelo responde de novo, três recusas devolvem os
motivos e nada muda, o prompt com as operações, o registro com o valor
secreto como tamanho e nunca o texto, a superfície e o que está de
lado, e um agente que não promove ação só da pessoa).

### o caminho pela página e pelo port

`apps/uigallery/src/views/apps/surface/offering/changing.rs`,
compartilhado pela página Surface e pela Silences: com uma superfície
aberta, o `propose` lê a intenção como mudança primeiro. pelas regras,
aplica na hora pelo `SurfaceHost::apply_all` com o ator da request, e a
request termina com `steps` e `ops` (as operações em linha); "desfazer"
é `undo_asked`. pelo Claude, quando a intenção lê como mudança, o worker
(`surface_page/asking.rs`, `Job::Change`) pede operações fora da thread
do app, a request fica `Started`, a linha mostra "asking the Claude for
...: attempt n of 3, s", e o `settle` aplica quando chegam, com
`attempts`, `seconds` e `cost_usd`; "undo" não chama modelo. o que não é
mudança segue para a proposta da 69. a linha diz o que houve ("changed
by the rules for "...", 3 steps", "changed by the Claude for "...", 1
step, in 2.1 s, 1 attempt, 0.050 USD", "undone: the last change, 3
steps"), e o status projeta `steps` ou `undone`.

testes: `apps/uigallery/src/views/apps/silences/tests/generative/change.rs`
(uma pessoa pede menos e desfaz, byte a byte; um agente pede pelo
`act`, as mudanças são dele; comparar recusa e nada muda; uma intenção
de superfície ainda propõe uma; e com o `claude` falso: a mudança fora
da thread, com custo; uma resposta recusada pedida de novo; três
recusas não mudam nada; "undo" sem prompt novo) e
`apps/uigallery/src/views/apps/surface/tests/agent.rs` (um agente
pelo port de verdade escreve "too much", propõe, e depois "undo": as
requests são dele, o protocolo é o do `propose`).

### a política

`rust/screen/src/host/mod.rs`: `open_under` e `transform_under` guardam a
política com que a superfície foi verificada; `open` e `transform`
guardam a padrão (a das formas da página Surface, que não é minha).
`apply(id, op, by, app)` e `undo(id, app)` não recebem mais política:
usam a da superfície. `apply_all` (tudo ou nada, uma forma, um pedido),
`undo_asked`, `last_ask`, `current` e `policy`. as páginas mostram uma
proposta com a política dela (`GenerationPolicy::by` do autor).
testes: `rust/screen/tests/policy.rs` (a operação de um agente
verificada sob a política da pessoa que escreveu a superfície; os
pedidos tudo ou nada e desfeitos inteiros) e os de `tests/adapt.rs`,
no desenho novo.

### a ênfase

`rust/screen/src/draw.rs` e `token.rs`: o stat lê `typography` como
o tamanho do valor (`StatSize` `Sm`, `Md`, `Lg`), o badge lê
`background` como a intenção do formato preenchido (`Badge::count`);
`rust/plevir/src/transform/look.rs`: a escada do stat (`base_m`, `title`,
`headline`) e a do badge (preenchido com o accent). o badge ganha peso,
não tamanho: o comps não tem badge maior, e mudá-lo seria parada.
testes: `rust/screen/tests/emphasis.rs` e o teste de `look.rs`.

### as três lacunas, antes e depois

1. o `READS` (`rust/offers/src/prompt.rs:75` a `89`) diz que revisar
   itens é ver o que cada um é (a imagem, o nome, os números com nome,
   o estado) e que "só o que a intenção precisa" não tira isso; teste
   de texto em `rust/offers/tests/llm.rs`, sem nome de app.
2. e 3. a linha do item das regras por sentido
   (`rust/offers/src/row.rs`): o que o item é (frame, faixa, e a
   decisão como badge, porque `apply_cut` e `reject_cut` leem só ela,
   `list.rs`), os números como stat pequeno com o nome do estado
   ("start", "end", "duration"), as ações em botões pequenos numa linha.

| | antes (`main` @ 3dde4976) | depois |
| --- | --- | --- |
| core 1280x800 | `git show 3dde4976:tool/bench/regress/baselines/silences-review-by-the-rules@core@1280x800/frame.png` | `tool/bench/regress/baselines/silences-review-by-the-rules@core@1280x800/frame.png` |
| hoff 1280x800 | idem, `@hoff@1280x800` | `tool/bench/regress/baselines/silences-review-by-the-rules@hoff@1280x800/frame.png` |
| core 1200x800 | idem, `@core@1200x800` | `tool/bench/regress/baselines/silences-review-by-the-rules@core@1200x800/frame.png` |

no antes, cada corte é "3.02 5.02 2 0:03.0 to 0:05.0 pending" sem nome,
e em 1200 "reject cut" desce sozinho. no depois, cada corte tem o frame,
a faixa e o selo "pending"; embaixo "3.02 5.02 2" com "start end
duration"; embaixo os três botões numa linha, também em 1200. os dois
cortes cabem na janela. aceitei as três depois de olhar cada uma.

### a demo

pelas regras, com a revisão aberta, em teste
(`apps/uigallery/src/views/apps/silences/tests/generative/demo.rs`) e no
regress (três estados novos em `examples/regress/state.rs`, pedidos pela
linha como um agente pede):

| estado | antes | depois |
| --- | --- | --- |
| "tem informação demais" sobre a revisão do Claude | `tool/bench/regress/baselines/silences-review-by-the-claude-fixture@core@1280x800/frame.png` | `tool/bench/regress/baselines/silences-review-by-the-claude-fixture-too-much-in-portuguese@core@1280x800/frame.png` |
| "too much information" sobre a revisão das regras | `tool/bench/regress/baselines/silences-review-by-the-rules@core@1280x800/frame.png` | `tool/bench/regress/baselines/silences-review-by-the-rules-too-much-information@core@1280x800/frame.png` |
| "I can't compare these" sobre a revisão das regras | o mesmo antes | `tool/bench/regress/baselines/silences-review-by-the-rules-can-t-compare@core@1280x800/frame.png` |

- "tem informação demais" na revisão do Claude: saem o cartão do vídeo
  e o das contagens (contexto, nenhuma ação), os três botões de cada
  corte viram um grupo, o espaçamento aperta; os dois cortes, que
  estavam abaixo da dobra, ficam dentro de 1280x800, cada um com faixa,
  selo, duração e frame. 4 passos, do harness.
- "too much information" na revisão das regras: saem o vídeo grande e
  as contagens, aperta; a linha já vem agrupada. 3 passos.
- "I can't compare these": recusado, a linha diz por quê em vermelho
  (`proposed.silences.cuts` repete os itens sobre `silences:cuts`, e
  nenhuma lista de números aparece), a revisão fica como estava. a troca
  para gráfico é provada numa superfície com uma lista de números
  (`rust/offers/tests/change.rs`).
- o estado do que continua (`demo.rs`): um corte aplicado antes continua
  aplicado, no app e no selo da gerada; um slider do Claude no meio do
  arraste continua o arraste no lugar novo depois que um agente pede
  "too much information" (-45, -36, e -12 dB depois da mudança).

### baselines

- novos (3): `silences-review-by-the-rules-too-much-information@core@1280x800`,
  `silences-review-by-the-claude-fixture-too-much-in-portuguese@core@1280x800`,
  `silences-review-by-the-rules-can-t-compare@core@1280x800`, com imagem.
- mudaram (3), pela regra do item 7: `silences-review-by-the-rules@core@1280x800`,
  `@hoff@1280x800` e `@core@1200x800`. nenhum outro.

### contagem do gate e o regress

| | workspace | perna urnaui | total |
| --- | --- | --- | --- |
| antes, `main` @ 3dde4976 (a contagem da fase 1) | 2776 passam, 25 ignorados (142 suítes) | 85 passam, 1 ignorado (6 suítes) | 2861 |
| depois, esta branch @ 5d1ca12f (`./tool/tasks/fullcheck.sh` offline, saída 0, 22:50:45 a 22:52:58) | 2809 passam, 0 falhas, 25 ignorados (147 suítes) | 85 passam, 0 falhas, 1 ignorado (6 suítes) | 2894 |

33 testes novos e cinco suítes (`propose`: `adapt`, `change`,
`llm_change`; `surface`: `policy`, `emphasis`); o gate inteiro verde,
com a perna urnaui e o check wasm com `-D warnings`. `df -h /`: 19 GB
livres antes e depois. `cargo run --offline -p uigallery --example
regress -- check` (com o `main.rs` tocado): 173 baselines, 173 iguais,
nenhum frame desenhado, nenhuma GPU aberta (167 iguais aos da `main`,
3 abençoados de novo pela regra do item 7, 3 novos).

### um incidente

na primeira bênção do estado da fixture em português, o harness não
escolhia o proponente antes de pedir, e o seletor do regress escolhe o
`claude` que estiver no PATH: o `propose` da mudança começou pelo
Claude, e o processo do regress saiu logo depois (a request ficou
`Executing` e o estado falhou). não achei nenhum `claude -p` rodando
logo depois (`ps`), mas não consigo garantir que nenhuma chamada tenha
começado: no máximo uma, com o teto de 0,5 USD do `claude-model`. o
harness agora escolhe "rules" antes de todo pedido
(`examples/regress/state.rs`, `ask`), e nenhum teste do gate alcança o
programa de verdade (`Claude::find` não procura nos testes do crate).

### roteiro para o orquestrador provar com o Claude de verdade

na janela de 1200x800 do checkpoint b, com o binário desta branch (ou
da `main` depois de integrar):

```text
cargo build -p uigallery                     # e copiar o binário, como no checkpoint
PLEV_AGENT=1 PLEV_MIRROR=1 PLEV_BACKGROUND=1 PLEV_RECORD=1 <showcase> Silences

# a revisão pelo Claude (a cota é do orquestrador)
plev act silences_surface.proposer select --param '"claude"' --app showcase --name orquestrador
plev act silences_surface.intent set_value --param '"cut the silences of this video, then I want to review the cuts"' --app showcase --name orquestrador
plev act silences_surface.propose propose --app showcase --name orquestrador
plev observe --app showcase --name orquestrador --scope silences_surface.status --keys busy,message   # até a resposta
cargo run -p mirroring -- --app showcase --frames 1 --png 70b-01-review.png

# a mudança pelo Claude
plev act silences_surface.intent set_value --param '"tem informação demais"' --app showcase --name orquestrador
plev act silences_surface.propose propose --app showcase --name orquestrador
plev observe --app showcase --name orquestrador --scope silences_surface.status --keys busy,message,steps,attempt,cost_usd
cargo run -p mirroring -- --app showcase --frames 1 --png 70b-02-less-claude.png

# comparar e números maiores pelo Claude
plev act silences_surface.intent set_value --param '"I can'"'"'t compare these"' --app showcase --name orquestrador
plev act silences_surface.propose propose --app showcase --name orquestrador
plev observe --app showcase --name orquestrador --scope silences_surface.status --keys busy,message
plev act silences_surface.intent set_value --param '"números maiores"' --app showcase --name orquestrador
plev act silences_surface.propose propose --app showcase --name orquestrador
plev observe --app showcase --name orquestrador --scope silences_surface.status --keys busy,message
cargo run -p mirroring -- --app showcase --frames 1 --png 70b-03-bigger-claude.png

# desfazer (sem modelo) e as mesmas frases pelas regras
plev act silences_surface.intent set_value --param '"desfazer"' --app showcase --name orquestrador
plev act silences_surface.propose propose --app showcase --name orquestrador
plev act silences_surface.proposer select --param '"rules"' --app showcase --name orquestrador
plev act silences_surface.intent set_value --param '"too much information"' --app showcase --name orquestrador
plev act silences_surface.propose propose --app showcase --name orquestrador
cargo run -p mirroring -- --app showcase --frames 1 --png 70b-04-less-rules.png
```

o que olhar: na 01, se a revisão nova mantém o frame de cada corte (a
lacuna 1 do prompt); na 02, as operações que o Claude escolhe (o
`steps` e o `ops` do receipt, o custo e as tentativas na linha), se os
cortes ficam dentro da janela e se nada do que a pessoa decide saiu; em
"I can't compare these", se o Claude recusa com `{"ops": []}` ou tenta
um `swap_view` que o host recusa (três tentativas no máximo, a
superfície intacta); em "números maiores", se sobe os stats; "desfazer"
não gasta chamada; pelas regras, o mesmo resultado da baseline
`...-too-much-information`. e pelo port, que cada receipt é do
`orquestrador`.

### linha de ADR para o índice (o [`adr/`](../../adr/README.md) não está na minha lista)

| 2026-09-24 | [adaptive-transforms-preserve-identity](../../adr/generated/0067-adaptive-transforms-preserve-identity.md) | adição proposta | fase 2: `propose::Transformer` ao lado do `Proposer`, `transform_verified` só devolve operações com o `Adapted` verificado; a gramática de mudanças das regras (menos, comparar com dificuldade, maior, desfazer, en e pt) por estrutura e registro; o prompt de mudança com `{"ops": [...]}` e reparo; o mesmo `propose` da linha com uma superfície aberta, sem ação nova (a projeção das baselines não muda); o host guarda a política de cada superfície e os pedidos (`apply_all`, `undo_asked`); ênfase de stat por `typography` e de badge por `background`; a linha do item das regras por sentido, a decisão como badge (task-70 fase 2) |

### o que a fase 3 precisa desta

- `SurfaceHost::changes` com o ator e `last_ask` (o que um pedido fez),
  ao lado dos receipts da 63; um pedido é uma request `propose` com
  `steps` e `ops` no outcome, então o detector de sequência também lê
  os receipts.
- instalar uma composta é um `promote` depois que o app a declarar; o
  prompt de mudança já descreve o `promote` e a recusa por ator.

### o que ficou aberto

- comparar na Silences: recusado com o motivo. para trocar por gráfico,
  o app teria que declarar uma lista de durações (a Silences, fora
  daqui).
- o Claude só recebe uma mudança quando a gramática das regras lê a
  intenção como mudança ("put the cuts first" iria para a proposta de
  superfície); crescer a gramática, ou deixar o modelo escolher entre
  mudança e superfície numa chamada só, é decisão do orquestrador.
- `PERSON_ONLY` pelo port: nem a Silences nem o todo têm ação só da
  pessoa, então a promoção recusada para um agente está provada no
  `propose` (o transformador e o `Llm`) e no host (fase 1), não pelo
  port de uma página.
- não há refazer; "undo" desfaz o último pedido inteiro, e o `undo` do
  host ainda desfaz uma operação.
- a ênfase do badge é de peso (preenchido), não de tamanho.
- a legenda da página Surface (`surface_page.rs`, fora da minha lista)
  não fala de mudança: depois de uma mudança ela continua dizendo o que
  dizia.
- "reduz densidade": segui o briefing, `density denser` (o espaçamento
  aperta); se a intenção deveria afrouxar, é uma linha em `reshape.rs`.
- a adição ao ADR está como proposta; o índice é do orquestrador.
