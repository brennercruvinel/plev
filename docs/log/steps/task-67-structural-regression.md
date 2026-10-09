---
project: plev
audience: [ai-agents, contributors]
status: ongoing
last-updated: 2026-09-23
domain: generative-runtime
task: task-67
wave: 5
depends: [task-59, task-66]
branch: task/67-structural-regression
integrated: 49e2822
---

# task-67: regressão estrutural antes de pixel

## objetivo

um teste de regressão descobre primeiro o que mudou (semântica, layer),
depois se aquilo deveria ter mudado, e só então compara pixels, quando é
relevante. a regressão chega nomeada pelo controle, não como contagem de
pixels.

## ligação com a tese

seção 26 (testes estruturais e visuais coexistem; "settings.security
changed / control: encryption.toggle / semantic state unchanged / visual
representation changed" no lugar de "snapshot mismatch: 37,821 pixels").
da rodada de ideias: regressão visual por hash de cena, não por pixel;
custo zero quando nada mudou; a diferença vem com o nome da layer.

## estado atual no código

- cada layer tem hash de conteúdo para dirty tracking
  (`rust/engine/src/drawing/compositor/layer/mod.rs:43`, `FxHasher`). o hash é
  determinístico dentro do mesmo build; muda quando a estrutura de dados do
  nó muda, mesmo sem mudança visual.
- validação de visual por pixel é regra do repositório (convenções,
  tests); não há harness de baseline por página.
- depois de 59: ligações `SemId -> rect, layer`. depois de 66: render
  headless e `--semantics`.

## escopo

1. baseline por (página, tema, viewport): o snapshot semântico (JSON), a
   lista de layers com um nome estável e o hash de conteúdo, e a imagem só
   quando pedida.
2. `check`: projeta e constrói a cena; compara semântica (diff da task-57)
   e hashes; se nada mudou, termina sem renderizar; se um hash mudou,
   renderiza os dois lados pelo headless e faz o diff de pixel só na região
   da layer; cruza a região mudada com as ligações para dizer quais
   controles foram afetados.
3. relatório por `SemId`: semântica mudou ou não, representação visual
   mudou ou não, qual layer, quantos pixels, e a região.
4. `bless`: atualiza o baseline, com o relatório do que está sendo aceito.
5. hash estável entre commits: o hash de dirty tracking não serve para
   baseline de longo prazo; esta task define um hash de conteúdo estável e
   documentado para o baseline (sem mexer no `FxHasher` do dirty
   tracking), ou documenta que baseline muda com a estrutura do nó e o
   fluxo cai no diff de pixel.
6. job de CI (macOS, onde o render é o mesmo do desenvolvimento).

## fora de escopo

comparação entre plataformas (77); regressão de interação (64).

## dependências

59 e 66 integradas.

## superfície sob responsabilidade

um exemplo ou binário de regressão (`cargo run -p showcase --example
regress -- check|bless` ou nome aprovado), os baselines em fixtures do
showcase, `.github/workflows` (o job), proposta de ADR
`docs/adr/workflows/0045-regression-by-meaning-before-pixels.md`.

## interfaces que consome

`Snapshot::diff` (57), `Compositor::bindings` (59), headless (66), layers
do compositor (leitura).

## interfaces que pode alterar

leitura de hash e nome de layer no compositor, por adição.

## critérios de aceitação

- sem mudança: `check` roda sem renderizar nada e termina verde.
- mudar a cor de um token em `scales.rs` ou `hoff.rs`: relatório diz
  "semântica igual, visual mudou" para os controles afetados, com região.
- mudar uma label: relatório diz "semântica mudou" para aquele `SemId`.
- tempo do `check` sem mudança medido e registrado.
- `./tool/tasks/fullcheck.sh` verde.

## testes esperados

testes do comparador com cenas pequenas (igual, só hash, só semântica,
os dois), do cruzamento região para `SemId`, e do relatório.

## riscos e perguntas abertas

- página que anima (spinner, blink de caret) precisa de relógio fixo no
  modo de regressão; o harness injeta.

## commits esperados

1. `task-67: baseline semântico e de layers por página`
2. `task-67: check sem render quando nada mudou`
3. `task-67: diff de pixel por layer e cruzamento com SemId`
4. `task-67: relatório por controle e bless`
5. `task-67: job de CI`

## pare e reporte se

- o nome estável de layer não existir sem mudar a API do compositor;
- a animação não puder ser congelada pelo relógio.

## estado (agente, 2026-09-23, branch `task/67-structural-regression`)

implementado, rebaseado na `main` @ bf67799 (task-82 fase b, 85, 87, 86
aberta; o último passo só trouxe o journal) e revisto a pedido do
orquestrador; `./tool/tasks/fullcheck.sh` verde na
árvore final, perna urnaui incluída. nenhum dos "pare e reporte"
disparou:

- nome estável de layer existe sem mudar a API do compositor:
  `Compositor::layers`, `Layer::nodes` e `Compositor::bindings` já eram
  públicos. o engine não foi tocado.
- a animação é congelada pelo relógio que já existia: `stage_page` anda
  em passos fixos de 1/60 s e nunca lê o relógio da parede. as 78
  páginas assentam em 2 passos, nos dois temas.
- o hash estável não toca `SceneNode` nem o dirty tracking.
- o job de CI não precisa de GPU quando nada mudou.

### o que foi feito

`cargo run -p showcase --example regress -- check|bless`, em
`tool/bench/regress/src/`:

- `scene.rs`: nome de cada layer, bounds do que ela desenha, ligações
  por layer; `hash.rs`: a codificação estável do hash.
- `baseline.rs`: o formato em disco, leitura e escrita.
- `compare.rs`: as duas primeiras perguntas (significado, desenho), sem
  pixel.
- `pixels.rs`: a terceira, só dentro da região das layers que mudaram,
  e a contagem por rect.
- `report.rs`: o relatório por `SemId` (modelo e `build`);
  `report/text.rs` o texto, `report/json.rs` o JSON do `check --json`,
  `report/describe.rs` uma mudança semântica em poucas palavras.
- `run.rs`: os comandos, o renderer aberto sob demanda e contado.
- `tests.rs`: de ponta a ponta sobre páginas reais, em diretórios
  temporários.

### depois da revisão do orquestrador

1. rebase na `main` sem conflito. `regress check` na árvore rebaseada:
   `checked 157 baselines in 838.6 ms: 157 unchanged, 0 changed; no
   frame drawn, no GPU opened`. nenhuma baseline mudou, então não houve
   bless de rebase. o que a 82b mexeu é o ritmo, não o desenho em
   repouso: `ShowcaseView::wake` pede ao `ToastManager` e ao workbench um
   `Wake` em vez de um `bool`, a meia-fase do caret recomeça na tecla, o
   shell alcança o tempo antes de um evento de fora (`app/pace.rs`) e os
   eventos do worker git entram por `apply_git`. o `stage_page` monta a
   página no estado de lançamento, sem foco, sem toast e sem worker git
   conectado, e assenta em 2 passos; nada disso aparece nesse frame. as
   imagens do todo desenhadas depois do rebase têm os mesmos bytes das
   de antes (sha1 `12e343d0...` core, `aaf7cf23...` hoff).
2. `report.rs` e `scene.rs` divididos por responsabilidade, sem mudar
   comportamento: o JSON de um check sobre as 157 baselines é o mesmo
   fora os tempos, e os 31 testes são os mesmos (três mudaram de módulo
   com o que testam). registro: contando só o código de produção, como a
   convenção manda (bloco `#[cfg(test)]` fora da conta), eles tinham 517
   e 535 linhas, abaixo das 639; os 745 e 730 eram o arquivo inteiro.
   agora o maior arquivo de produção do exemplo é `run.rs`, com 428.
3. o baseline do todo em core e hoff a 1280x800 guarda a imagem (87 KB e
   95 KB). por que só esses: o todo é a página que projeta e liga os
   controles (o Toggle Group é alias dele), então a imagem é o que troca
   "todo controle ligado na layer que mudou" pelo controle exato e a
   região; nenhuma outra página projeta (o Git liga três regiões do
   workbench sem projetar, as outras não ligam nada), e ali a imagem só
   estreitaria a layer a uma região. core é o tema de lançamento e hoff o
   vidro escuro, o par que os testes do headless desenham; cada imagem
   custa uns 90 KB a cada bless e são os pixels de uma GPU só.
4. `docs/ref/validate-visuals-by-pixel.md` ganha a seção do fluxo
   check/bless no lugar de "until then, check the snapshot by hand"; o
   README ganha a linha do `regress -- check`; o `arc.toml` ganha
   `showcase = ["measure", "regress"]`.

### formato do baseline

um diretório por (página, tema, viewport) em
`tool/bench/regress/baselines/<página>@<tema>@<w>x<h>[@<s>x]/`
(a escala só quando não é 1):

- `scene.json`: `format` (`regress.baseline.v1`), `hash`
  (`regress.layer.v1`), página, tema, largura, altura, escala, `settle`
  (passos, se assentou), `semantics` (revision e número de nós da
  projeção, ou `null`), `layers` (nome, z, hash em 16 dígitos hex,
  número de nós, bounds lógicos) e `bindings` (id, nome da layer, rect,
  clip).
- `semantics.json`: a projeção na forma enxuta do `showcase --render
  --semantics`, só para página que projeta.
- `frame.png`: só com `bless --png`; um baseline que guarda imagem
  continua guardando no próximo bless, até `--no-png`.

commitados: as 78 páginas em core e em hoff a 1280x800 e o todo em core a
390x844, 157 baselines, 166 KB de JSON; imagem só no todo em core e hoff
a 1280x800 (182 KB).

### nome e hash de layer, e por que são estáveis

- nome: as raízes dos `SemId` ligados na layer, em ordem de bind, unidas
  por `+` (`todo`; no Git, `changes+stacks+diff`); sem ligação, o
  z-order que o app deu (`z0`, `z10`, `z1000`); nome repetido ganha
  `#2` em ordem de composição (no Git, `z1000#2`, a layer de overlay
  do workspace). item novo sob `todo` não renomeia, layer criada em
  outro z também não. a lista completa dos ids foi descartada porque
  renomeia a layer a cada item, e a posição entre as layers porque
  renomeia tudo que está acima de uma layer nova.
- hash: FNV-1a 64 sobre uma codificação escrita no doc de `hash.rs`
  (tag de versão, z, visibilidade, opacidade, scissor, efeitos, e cada
  nó em ordem de push com uma tag e os campos, `f32` pelos bits). um
  valor dourado calculado por um script à parte (python, `struct.pack`)
  prende a codificação. imagem entra pelo tamanho, não pelo id: o id é a
  ordem de carga do processo, e o Card sozinho ou depois de outras 70
  páginas dá o mesmo hash. `bless --all` duas vezes, em processos
  diferentes e em ordens diferentes, gera arquivos idênticos.

### aceite

na árvore final, contra as baselines commitadas; as mudanças de token e
de label foram feitas numa cópia de trabalho e revertidas (`git status`
limpo depois de cada uma).

1. sem mudança, `check` em todas as baselines termina verde sem abrir
   GPU. o resumo conta os frames desenhados, e o teste
   `a_page_checked_against_its_own_bless_holds_without_opening_a_gpu`
   exige `gpu.frames == 0` e nenhum renderer aberto:

   ```text
   ok       typography@hoff@1280x800 (stage 3.6 ms, hash 0.1 ms, compare 0.0 ms)
   checked 157 baselines in 627.2 ms: 157 unchanged, 0 changed; no frame drawn, no GPU opened
   ```

2. cor de um token: `text_mid` do hoff em `rust/engine/src/styling/theme/hoff.rs`,
   `n2(0.70)` para `n2(0.60)`. 28 baselines hoff mudam, nenhuma core; o
   todo em hoff tem imagem e nomeia os controles com região, o único
   frame desenhado:

   ```text
   CHANGED  todo@hoff@1280x800 (stage 3.8 ms, hash 0.1 ms, compare 0.0 ms, render 19.1 ms, decode 2.5 ms, diff 11.6 ms)
     todo          semantics unchanged; visual changed, layer todo, 3597 px at 506,230 273x100
     todo.item[1]  semantics unchanged; visual changed, layer todo, 1272 px at 506,230 273x13
     todo.item[2]  semantics unchanged; visual changed, layer todo, 964 px at 507,274 204x12
     todo.item[3]  semantics unchanged; visual changed, layer todo, 1361 px at 506,317 226x13
     6 other controls unchanged
     layer todo: changed (f038f7f55fdc9bd6 -> 18a4a7dec0cfac78), 3597 px at 506,230 273x100
     pixels: 3597 changed of 572539 compared at 506,230 273x100
   ...
   checked 157 baselines in 730.0 ms: 129 unchanged, 28 changed; 1 frames drawn, GPU opened in 21.2 ms
   ```

   uma página sem imagem diz a layer:

   ```text
   CHANGED  button@hoff@1280x800 (stage 3.7 ms, hash 0.3 ms, compare 0.0 ms)
     layer z0: changed (4f2d8232e76eca3d -> 27f396cf34e7130d)
     pixels: visual changed, no baseline image to diff (bless --png keeps one)
   ```

3. uma label do todo: `"Measure text, never estimate it"` para `"Measure
   text, then estimate it"` em `apps/uigallery/src/views/apps/todo/mod.rs:68`.
   5 baselines mudam (as três do todo, o Toggle Group
   nos dois temas); as duas com imagem:

   ```text
   CHANGED  todo@core@1280x800 (stage 3.9 ms, hash 0.1 ms, compare 0.2 ms, render 24.5 ms, decode 2.3 ms, diff 11.5 ms)
     todo          semantics unchanged; visual changed, layer todo, 727 px at 600,277 111x10
     todo.item[2]  semantics changed: label "Measure text, never estimate it" -> "Measure text, then estimate it"; visual changed, layer todo, 727 px at 600,277 111x10
     8 other controls unchanged
     layer todo: changed (541caa8afab91996 -> de9200db8cf27cee), 727 px at 600,277 111x10
     pixels: 727 changed of 572539 compared at 600,277 111x10
   CHANGED  todo@hoff@1280x800 (stage 3.8 ms, hash 0.1 ms, compare 0.1 ms, render 10.9 ms, decode 2.1 ms, diff 11.4 ms)
     todo          semantics unchanged; visual changed, layer todo, 732 px at 600,274 111x10
     todo.item[2]  semantics changed: label "Measure text, never estimate it" -> "Measure text, then estimate it"; visual changed, layer todo, 732 px at 600,274 111x10
     8 other controls unchanged
     layer todo: changed (f038f7f55fdc9bd6 -> 1674e8c6fe44670e), 732 px at 600,274 111x10
     pixels: 732 changed of 572539 compared at 600,274 111x10
   ...
   checked 157 baselines in 784.2 ms: 152 unchanged, 5 changed; 2 frames drawn, GPU opened in 22.7 ms
   ```

   sem imagem (o todo a 390x844, o Toggle Group), `todo.item[2]` diz a
   mesma mudança de label, e cada controle da layer sai como "visual
   changed, layer todo (the layer moved; its pixels were not compared)".

### medições

máquina apple m4, build de debug (release não foi medido: o briefing
proíbe `cargo build --release`). medido antes do rebase; depois dele o
check sem mudança seguiu entre 0,61 e 0,84 s com a máquina dividida com
outros builds.

check sem mudança, 157 baselines, cinco execuções:

| o quê | mínimo | mediana | p95 | máximo |
| --- | --- | --- | --- | --- |
| total | 606 ms | 607 ms | | 628 ms |
| stage + projeção, por página | 3,52 ms | 3,61 ms | 3,92 ms | 5,17 ms |
| nome + hash das layers, por página | 0,04 ms | 0,12 ms | 0,26 ms | 0,47 ms |
| comparação, por página | | 1 us | 4 us | 19 us |

a primeira página de um processo custa de 9 a 26 ms a mais (13 a 29 ms
no total, o aquecimento do processo). as mais lentas: Builder (4,7 ms) e
Icons (4,7 ms).

quando uma layer muda, as 78 páginas hoff com imagem num diretório
local e `text_mid` alterado (28 mudam), três execuções:

| o quê | mínimo | mediana | p95 | máximo |
| --- | --- | --- | --- | --- |
| abrir o renderer (uma vez) | 16,6 ms | 16,9 ms | | 21,0 ms |
| desenhar e ler de volta | 3,9 ms | 5,7 ms | 12,5 ms | 19,7 ms |
| decodificar o png | 1,6 ms | 2,0 ms | 2,7 ms | 3,3 ms |
| comparar na região | 0,8 ms | 8,1 ms | 13,5 ms | 13,8 ms |

pixels comparados por página: de 37 mil a 684 mil (mediana 408 mil) de
1,02 milhão. as 78 baselines em 1,17 s contra uns 0,3 s sem mudança.
nessa mesma rodada o Button hoff mudou o hash com 0 de 673 mil pixels
diferentes: um nó cuja cor mudou não aparece. o check falha e diz 0 px.

## handoff

```text
Goal: task-67, regressão estrutural antes de pixel.
Status: implementado, rebaseado na main @ bf67799 e revisto; gate verde
(2160 testes passados, perna urnaui incluída); regress check verde na
árvore final.

Implementado:
- `cargo run -p showcase --example regress -- check|bless`, com
  `--page` (repetível), `--theme`, `--width`, `--height`, `--scale`,
  `--dir`; `bless --all`, `--png`, `--no-png`; `check --json <file>`.
  saída 0 tudo segurou, 1 algo mudou, 2 pedido malformado, 3 erro.
- check: significado (revision; `Snapshot::diff` só quando difere),
  desenho (hash estável por layer casada por nome, rect de cada
  ligação); nada mudou, termina sem abrir GPU. layer mudou e o baseline
  tem imagem: frame pelo headless, pixels só na região das layers que
  mudaram, cruzados com as ligações.
- relatório por SemId (texto e JSON): semântica mudou ou não, visual
  mudou ou não, layer, pixels, região; layers com os pixels fora de
  qualquer controle; tempos por etapa; frames desenhados.
- bless imprime o check do baseline que substitui ("accept", "same",
  "new") e escreve.
- 157 baselines commitadas, imagem no todo core e hoff a 1280x800; job
  `structural regression (macos)` no CI.

Arquivos principais:
- tool/bench/regress/src/{main,run,scene,hash,baseline,compare,pixels,report,tests}.rs
  e report/{text,json,describe}.rs (novos)
- tool/bench/regress/baselines/** (novos)
- apps/uigallery/src/shell/window/headless.rs (aditivo)
- apps/uigallery/Cargo.toml, Cargo.lock
- .github/workflows/ci.yml (o job novo)
- docs/adr/workflows/0045-regression-by-meaning-before-pixels.md (novo)
- docs/ref/validate-visuals-by-pixel.md, README.md, docs/ATLAS.toml
- este arquivo

Interfaces adicionadas/alteradas:
- `StagedPage::compositor(&self) -> &Compositor`, `steps() -> usize`,
  `settled() -> bool`, só leitura. o laço de settle é o mesmo, escrito
  com um contador. nenhuma interface do engine mudou.
- apps/uigallery/Cargo.toml: `[[example]] regress` com `test = true`
  (fora da letra do briefing, que só previa dependência de dev: sem isso
  o cargo compila os testes de um exemplo e não os roda, e os 31 do
  comparador ficariam fora do gate); `image` e `tempfile` em
  dev-dependencies (decodificar o png do baseline; diretórios
  temporários nos testes).

Testes (antes -> depois):
- gate inteiro na árvore rebaseada: 2160 passados, 0 falhas, 12
  ignorados, nenhum pulado por falta de GPU. 32 são desta task (31 do
  exemplo, 1 do headless); a main em bf67799 teria 2128 por essa conta
  (não rodei o gate da main à parte). antes do rebase: 2096 -> 2128.
- `cargo test -p showcase --example regress`: 0 -> 31 (cena: nome
  estável, bounds com clip e texto, ligações, JSON; hash: codificação
  presa, muda com valor e não com ligação; baseline: rótulo, ida e
  volta, formato velho recusado; comparador: igual, só hash, só
  semântica, os dois, projeção de um lado só; pixels: região e só ela,
  centro do pixel, buraco, tamanhos diferentes; relatório: por
  controle, texto e JSON, linha única quando segurou, uma mudança
  descrita pelo nó; ponta a ponta: página contra o próprio bless sem
  abrir GPU, label que nomeia `todo.item[2]`, layer sem imagem, e com
  GPU um bloco sobre o segundo item nomeado com a região exata).
- `cargo test -p showcase --lib`: +1 (`a_page_stages_the_same_scene_every_time`).

Validação executada:
- ./tool/tasks/fullcheck.sh (com o ambiente de build do briefing, perna urnaui
  incluída): verde na árvore final.
- regress check na árvore rebaseada e na final: 157 unchanged, nenhum
  frame desenhado.
- os três aceites acima contra as baselines commitadas, com as mudanças
  revertidas.
- `bless --all` duas vezes em processos separados: arquivos idênticos;
  Git, Card e Avatar sozinhos: o mesmo scene.json; as imagens do todo
  antes e depois do rebase: os mesmos bytes.
- actionlint no ci.yml; o `arc_sync_guard` com a linha nova do arc.toml.

Decisões:
- o veredito é o hash e a semântica; pixel explica, nunca decide. sem
  GPU o veredito é o mesmo, e um hash que muda com 0 px diferentes
  falha (o relatório diz 0 px; bless aceita).
- imagem só no baseline do todo em core e hoff a 1280x800 (decidido pelo
  orquestrador; o porquê está acima e no ADR).
- viewport padrão 1280x800 escala 1, o desktop dos testes do headless.

Limitações:
- regressão por significado só vale onde há projeção: o todo e o
  Toggle Group (alias do todo). o Git liga três ids sem projetar; as
  outras páginas comparam só layers, ligações e pixels.
- o hash vê a cena, não o raster: shader, arquivo de fonte ou
  rasterizador mudam pixels sem mover hash (isso é a task-77).
- imagem entra pelo tamanho: pixels novos numa imagem do mesmo tamanho
  não mudam o hash.
- sem imagem no baseline, o relatório nomeia todos os controles da
  layer que mudou, sem região.
- bounds de texto são o box medido mais um quarto do tamanho da fonte;
  a região comparada é um superconjunto (só custa comparar mais).
- camadas sobrepostas: um pixel mudado que cai na região de duas layers
  que mudaram conta nas duas.

Dependências pendentes: nenhuma.

Riscos de integração:
- toda mudança que mexe no desenho de uma página precisa de `regress
  bless` na mesma mudança, ou o job de CI fica vermelho. o bless das
  duas páginas com imagem precisa de GPU (sem adapter ele recusa e diz
  que `--no-png` tira a imagem).
- os hashes são bits exatos de `f32` de uma cena montada na CPU. poucas
  páginas passam coordenadas pelo libm do sistema (seno e cosseno no
  donut, no spinner, no skeleton e em Effects); outro macOS no runner
  pode mudar o último bit. a primeira execução do job no PR confirma;
  se divergir, as opções são tirar essas páginas das baselines,
  abençoar no runner, ou quantizar essas coordenadas no hash.
- as imagens do todo são pixels do Metal desta máquina: num runner que
  desenha diferente, uma mudança no todo traz o ruído da outra GPU na
  contagem de pixels. o veredito não muda (é o hash), só o detalhe.

Commits (depois do rebase):
- 9862f55 task-67: baseline semântico e de layers por página
- 8a5c89f task-67: check sem render quando nada mudou
- f824a95 task-67: diff de pixel por layer e cruzamento com SemId
- 89e10c0 task-67: relatório por controle e bless
- 92297e6 task-67: job de CI
- 7a75708 task-67: o tempo do hash sai à parte do stage
- fc4d637 task-67: ADR, medição e handoff
- d3f146e task-67: report e scene divididos por responsabilidade
- ca41424 task-67: o baseline do todo guarda a imagem, em core e hoff no desktop
- a595c7f task-67: o manual, o README e o arc.toml falam do regress
- (o ADR e este arquivo depois da revisão, no commit seguinte)
```

### proposta para o orquestrador

linha do índice de ADRs (`docs/adr/README.md`):

```text
| 2026-09-23 | [regression-by-meaning-before-pixels](regression-by-meaning-before-pixels.md) | proposed | regressão pergunta na ordem: o significado mudou (revision, `Snapshot::diff`), o desenho mudou (hash estável por layer, FNV-1a 64 sobre codificação documentada, `f32` pelos bits; nome de layer pelas raízes dos `SemId` ligados, ou pelo z-order), e só então os pixels, só na região das layers que mudaram, cruzados com as ligações para nomear o controle; nada mudou, nenhuma GPU aberta; pixel explica e não decide o veredito; imagem só no baseline do todo; `regress check|bless` no showcase e job de CI no macOS (task-67) |
```

## fechamento (orquestrador, 2026-09-23)

integrada na `main` em `49e2822` (onze commits). validação: diff inteiro
lido nas duas rodadas; devolvida uma vez (rebase sobre a 82b com check,
arquivos grandes, as imagens do Todo, os docs); a divisão de arquivos não
era obrigatória pela regra exata (o orquestrador tinha contado o arquivo
inteiro, não a produção), e não fez mal. gate verde no diretório da task:
2160 passaram, 0 falharam, 12 ignorados. `regress check` sobre a árvore
final: 157 baselines iguais, nenhum frame, nenhuma GPU. reproduzido pelo
orquestrador numa cópia de trabalho: trocar a label de um item do Todo
dá `todo.item[2] semantics changed: label ...; visual changed, 727 px at
600,277 111x10`. o ADR passou de proposto a aceito. o primeiro job de CI
no macOS do GitHub confirma se os hashes batem fora desta máquina.
