---
id: task-74/phase-7-large-documents
title: "phase 7: large documents"
project: plev
type: section
parent: task-74
status: reference
date: 2026-09-25
date_basis: text
recorded: 2026-09-23
topics: [text, text-editing, performance]
relations: []
evidence: [e7427780, 71e0a6f2, 8b4c58e9, 727c378e]
source: docs/log/steps/task-74-text-and-editing.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# phase 7: large documents

Measures the editor with one million lines in release (Apple M4, 16 GB, macOS 26.6.2) and fixes what the number showed: line positions move to f64 (in f32 it went wrong from line 100 thousand on), `Document::load` stops copying the text, and lines with a caret or selection gain a shaping cache. With one million lines, opening takes 21 ms, the frame takes 0,36 ms per wheel event, and the peak on opening drops from 121 to 65 MiB, with no change to the public API. It closes the benchmark item of the criterion; the f32 scrolling of `ScrollState` remains pending.

## fase 7: a sonda

documentos grandes no editor: o que ele faz hoje com N linhas, onde entram
o layout virtualizado e o cache de shaping por linha, e o que o rope já dá.
os números vêm do benchmark desta fase (`rust/engine/tests/editor.rs`,
comando no fim da seção), em release, num Apple M4 com 16 GB, macOS 26.6.2.

### (a) o que o editor faz por frame com N linhas

- já é virtualizado na emissão. `render` percorre só `visible_lines()`
  (`rust/engine/src/widgets/editor/view.rs:288`, laço em `:299-339`), que é a
  conta pura de `visible_line_range` (`view.rs:468-483`): as linhas que
  cortam a viewport mais `overscan_lines` (8) de cada lado. a altura total
  vem da contagem de linhas, sem medir nenhuma
  (`view.rs:98`, `len_lines() * line_height`).
- por linha visível: o texto sai do rope (`view.rs:134-140`,
  `rope.line(n)`, O(log n)); o número da margem é medido com
  `TextMeasurer::measure_styled` (`view.rs:304-305`), que tem cache LRU de
  2048 entradas por conteúdo (`rust/engine/src/drawing/text/measure.rs:24`,
  `:327-346`); o texto vira um nó `SceneNode::Text` com o conteúdo como
  chave.
- quem molda o texto desenhado é o `TextSystem`, na hora de montar o frame:
  `ensure_shaped` (`rust/engine/src/drawing/text/system.rs:253-289`) molda uma
  vez por chave, num mapa por conteúdo, e `finish_frame` (`system.rs:346-352`)
  joga fora o que o frame não usou. isso já é o shaping por linha com cache
  do desenho, limitado às linhas da tela.
- o que o editor molda de novo a cada frame, sem cache: a linha de cada
  caret na tela (`view.rs:343-360`, `caret_x` em `:145-147`) e cada linha
  visível com seleção (`view.rs:319-329`, `selection_boxes_on_line` em
  `:423-441`). as duas passam por `VisualLine::new`
  (`rust/engine/src/widgets/form/text_input/visual.rs:51-52`), que chama
  `TextMeasurer::clusters_styled` (`measure.rs:400-414`): esse molda toda
  vez, é o único caminho de shaping do measure sem cache.
- a geometria das linhas é f32 contra o topo do documento:
  `line_top` (`view.rs:129-131`) faz `line as f32 * lh - offset`,
  `hit_test_point` (`view.rs:150-163`) faz `y - bounds.y + offset`, e o
  `scroll_to_cursor` (`view.rs:166-178`) também. com a linha de 19,6 px do
  tema, um milhão de linhas dá 19,6 milhões de px, acima dos 16,7 milhões
  que o f32 guarda com passo de 1 px. medido com os testes novos
  (`editor/tests_large.rs`), antes da correção: o passo entre duas linhas
  sai 19,5 na linha 99.990 e 20 na linha 499.963, e um clique a 31,1 px do
  topo, no fim do documento, cai na linha 999.972 em vez da 999.971.

custo medido antes de mudar (média / p99, ms; `scene` para no
`Compositor::resolve_scene`, sem GPU; `frame` passa pelo
`headless::Renderer::draw`, que molda, rasteriza e submete como a janela):

| linhas | até | abrir | roda, 3 linhas por frame | arrasto da barra | roda, tudo selecionado | editar no meio | heap mantido (MiB) | pico ao abrir (MiB) |
|---|---|---|---|---|---|---|---|---|
| 10 mil | scene | 0,3 | 0,021 / 0,026 | 0,055 / 0,060 | 0,589 / 0,629 | 0,028 / 0,033 | 0,6 | 1,1 |
| 100 mil | scene | 2,0 | 0,021 / 0,025 | 0,088 / 0,099 | 0,590 / 0,636 | 0,030 / 0,034 | 6,2 | 11,8 |
| 1 milhão | scene | 22,8 | 0,021 / 0,025 | 0,092 / 0,101 | 0,590 / 0,628 | 0,029 / 0,034 | 64,2 | 121,3 |
| 10 mil | frame | 1,0 | 0,364 / 0,585 | 0,701 / 0,907 | 0,932 / 1,063 | 0,262 / 0,341 | 0,6 | 1,5 |
| 100 mil | frame | 3,1 | 0,357 / 0,573 | 1,007 / 1,119 | 0,919 / 1,001 | 0,280 / 0,393 | 6,2 | 11,8 |
| 1 milhão | frame | 23,3 | 0,357 / 0,544 | 1,194 / 2,884 | 0,946 / 2,026 | 0,311 / 1,071 | 64,2 | 121,3 |

o frame não depende de N: rolar e editar custam o mesmo com 10 mil e com
um milhão de linhas. só abrir cresce com o documento, e o pico ao abrir é
quase o dobro do que fica, porque `Document::load` copia o texto inteiro
antes de montar o rope (`rust/editor/src/document/mod.rs:51-62`, o
`text.into()`). com tudo selecionado, a cena custa 28 vezes a da roda
simples: é o `clusters_styled` moldando de novo cada linha da tela a cada
frame.

### (b) onde entram o layout virtualizado e o cache por linha

- layout virtualizado: a emissão já é. o que falta é a geometria aguentar
  um milhão de linhas. a posição de cada linha passa a ser calculada em
  f64 contra o deslocamento da rolagem e só o resultado, pequeno, volta a
  f32 (`line_top`, `hit_test_point`, `scroll_to_cursor` e a faixa
  visível). o `ScrollState` guarda o deslocamento em f32
  (`rust/engine/src/control/input/scroll.rs:42-56`) e é do engine, fora dos
  arquivos desta fase: perto do fim de um milhão de linhas ele anda em
  passos de 2 px. isso vira pendência, não mudança de API.
- cache de shaping por linha: no `clusters_styled` do measure, ao lado
  do cache de medida que já existe. chave: o texto da linha e o estilo
  (os mesmos campos do `MeasureKey`, com a largura exata). invalidação
  numa edição: a chave é o conteúdo, então só a linha cujo texto mudou
  deixa de bater; uma quebra de linha que empurra um milhão de linhas
  para baixo não invalida nenhuma, porque a chave não carrega o índice.
  limite de memória: orçamento em bytes (texto mais clusters de cada
  entrada), com despejo da menos usada. fica no engine e não no editor
  porque o que se repete é o shaping dentro do `clusters_styled`, e o
  editor não consegue guardar o `VisualLine` entre frames (ele empresta o
  texto e mora em `form/`, fora dos arquivos permitidos). o `TextField`
  ganha o mesmo cache sem mudar nada. o cache do desenho
  (`TextSystem`) continua como está.
- abrir: `Document::load` deixa de copiar o texto quando não há `\r`.

### (c) o que o rope já dá

o rope é o ropey 1.6, uma árvore B com contagem de linhas nos nós:
`line(n)`, `line_to_byte` e `byte_to_line` são O(log n) e `len_lines` é
O(1). uma edição é uma `Transaction` (`rust/editor/src/document/transaction.rs:91-105`):
para cada trecho, `byte_to_char`, `remove` e `insert`, todos O(log n). o
inverso guarda só o texto removido (`transaction.rs:110-124`), e o
histórico guarda a transação e o inverso, sem cópia do documento. inserir
e apagar um caractere na linha 500 mil não custa nada linear; o benchmark
confirma (a coluna de edição é a mesma com 10 mil e com um milhão).

### como reproduzir

```sh
export CARGO_TARGET_DIR=/Users/nn/Dev/hoff/plevdev/target/agents/one
cargo test --offline --release -p engine --test editor_large_bench -- --ignored --nocapture
```

## fase 7: o benchmark

mesmo teste, mesma máquina (Apple M4, 16 GB, macOS 26.6.2), perfil
release, depois das três mudanças (linhas em f64, `Document::load` sem
cópia, cache de clusters por linha). o documento gerado tem linhas de
código de 44 a 65 bytes, todas diferentes; com um milhão de linhas são
57 MiB de texto. cada cenário: a roda anda 3 linhas por frame por 2000
frames; o arrasto da barra vai do topo ao fim em 300 frames, cada um numa
viewport nova; a roda com tudo selecionado são 1000 frames depois de
`select_all`; a edição insere e apaga um caractere na linha do meio, 200
vezes cada, medindo até o frame seguinte montado. média / p99 em ms.

| linhas | até | abrir | roda, 3 linhas por frame | arrasto da barra | roda, tudo selecionado | editar no meio | heap mantido (MiB) | pico ao abrir (MiB) |
|---|---|---|---|---|---|---|---|---|
| 10 mil | scene | 0,2 | 0,021 / 0,025 | 0,054 / 0,064 | 0,146 / 0,177 | 0,019 / 0,022 | 0,6 | 0,6 |
| 100 mil | scene | 1,6 | 0,021 / 0,025 | 0,086 / 0,099 | 0,146 / 0,159 | 0,020 / 0,024 | 6,2 | 6,2 |
| 1 milhão | scene | 17,5 | 0,021 / 0,029 | 0,095 / 0,126 | 0,148 / 0,185 | 0,020 / 0,040 | 64,2 | 64,2 |
| 10 mil | frame | 0,8 | 0,365 / 0,593 | 0,673 / 0,826 | 0,495 / 0,607 | 0,304 / 0,771 | 0,6 | 1,5 |
| 100 mil | frame | 3,1 | 0,368 / 0,625 | 1,019 / 1,227 | 0,481 / 0,558 | 0,258 / 0,317 | 6,2 | 7,1 |
| 1 milhão | frame | 21,0 | 0,360 / 0,587 | 1,042 / 1,241 | 0,482 / 0,568 | 0,268 / 0,343 | 64,2 | 65,0 |

antes e depois, com um milhão de linhas (os números de antes estão na
sonda):

| um milhão de linhas | antes | depois |
|---|---|---|
| abrir até o primeiro frame (ms) | 23,3 | 21,0 |
| pico de heap ao abrir (MiB) | 121,3 | 65,0 |
| heap mantido pelo documento e pelo editor (MiB) | 64,2 | 64,2 |
| roda, frame, média / p99 (ms) | 0,357 / 0,544 | 0,360 / 0,587 |
| arrasto da barra, frame (ms) | 1,194 / 2,884 | 1,042 / 1,241 |
| roda com tudo selecionado, cena (ms) | 0,590 / 0,628 | 0,148 / 0,185 |
| roda com tudo selecionado, frame (ms) | 0,946 / 2,026 | 0,482 / 0,568 |
| editar no meio até o frame seguinte (ms) | 0,311 / 1,071 | 0,268 / 0,343 |

leitura: o frame já não dependia de N antes e continua sem depender; o
pior frame medido, com um milhão de linhas, fica abaixo de 1,3 ms no
p99, longe dos 16,7 ms de 60 Hz. o que mudou foi o pico ao abrir (a
cópia do texto saiu), o custo de redesenhar linhas com seleção (o shaping
dos clusters deixou de se repetir a cada frame) e, fora da tabela, a
posição das linhas: antes, a partir da linha 100 mil, o passo entre duas
linhas saía errado em até 0,4 px e o clique perto do fim caía na linha
vizinha.

cada tabela é a segunda de duas rodadas seguidas. entre rodadas, as
médias variam até uns 15%; o p99 do `frame` varia até o dobro, porque
passa pela submissão à GPU (a primeira rodada de antes deu 1,973 ms de
p99 na roda com 10 mil linhas; a segunda, 0,585). a memória é só o heap
da CPU, contado por um alocador no próprio teste; o atlas de glifos na GPU
não entra.

para reproduzir (o disco precisa de uns 2 GB para o perfil release):

```sh
export CARGO_TARGET_DIR=/Users/nn/Dev/hoff/plevdev/target/agents/one
cargo test --offline --release -p engine --test editor_large_bench -- --ignored --nocapture
```

## handoff da fase 7: documentos grandes

fecha o item "benchmark de um milhão de linhas com números" do critério
de aceite: o editor abre, rola até o fim e edita no meio um documento de
um milhão de linhas, com o frame abaixo de 1,3 ms no p99 em release, e o
que ele desenha bate com o documento.

### commits

- `e7427780` a sonda, com os números de antes e o benchmark
- `71e0a6f2` as linhas posicionadas em f64 e o `Document::load` sem cópia
- `8b4c58e9` o cache de clusters por linha, com orçamento em bytes
- `727c378e` o benchmark depois, e a tabela de antes e depois
- o commit desta seção: o handoff

### a sonda, curta

o editor já era virtualizado na emissão: por frame, só as linhas da tela
mais 8 de cada lado viram nós de cena, e a altura vem da contagem de
linhas. o texto desenhado já tinha shaping por linha com cache no
`TextSystem`. o rope (ropey) já ia à linha n e editava no meio em O(log
n). o que faltava: a posição das linhas era f32 e errava a partir da
linha 100 mil; a linha do caret e as linhas com seleção eram moldadas de
novo a cada frame (`clusters_styled` sem cache); e abrir copiava o texto
inteiro antes de montar o rope.

### o que mudou

- `rust/engine/src/widgets/editor/view.rs`: `line_y` calcula em f64 a distância
  do topo do documento até a linha; `line_top`, `hit_test_point` e
  `scroll_to_cursor` usam essa conta, e só a diferença pequena, na tela,
  volta a f32. `input.rs`: o encaixe da roda em linhas inteiras usa a
  mesma conta. para um documento pequeno com rolagem zero, o resultado é
  bit a bit o de antes (as 176 baselines iguais confirmam).
- `rust/editor/src/document/mod.rs`: `Document::load` monta o rope direto do
  `&str` e só copia quando há `\r` para normalizar.
- `rust/engine/src/drawing/text/cluster_cache.rs` (novo): o cache de
  `TextMeasurer::clusters_styled`. chave: o texto e o estilo, com a
  largura exata; sem índice de linha, então uma edição invalida só as
  linhas cujo texto mudou, e uma quebra de linha que empurra o resto do
  documento não invalida nada. limite: 4 MiB contados por entrada (chave
  e clusters), sai a menos usada; uma linha maior que o orçamento inteiro
  não entra. `measure.rs` só consulta e guarda (633 linhas).
- nenhuma API pública mudou; nada foi removido.

### números, um milhão de linhas, release, Apple M4

| | antes | depois |
|---|---|---|
| abrir até o primeiro frame (ms) | 23,3 | 21,0 |
| pico de heap ao abrir (MiB) | 121,3 | 65,0 |
| roda, frame, média / p99 (ms) | 0,357 / 0,544 | 0,360 / 0,587 |
| arrasto da barra, frame (ms) | 1,194 / 2,884 | 1,042 / 1,241 |
| roda com tudo selecionado, frame (ms) | 0,946 / 2,026 | 0,482 / 0,568 |
| editar no meio até o frame seguinte (ms) | 0,311 / 1,071 | 0,268 / 0,343 |

as tabelas completas (10 mil, 100 mil e um milhão, cena e frame) estão na
sonda e na seção do benchmark. comando:

```sh
export CARGO_TARGET_DIR=/Users/nn/Dev/hoff/plevdev/target/agents/one
cargo test --offline --release -p engine --test editor_large_bench -- --ignored --nocapture
```

### testes (8 novos, 1 benchmark ignorado)

- `rust/engine/src/widgets/editor/tests/large.rs`, com um milhão de linhas, em
  debug e no gate: abre e rola até o fim; rola por onze pontos do
  documento; clica perto do fim e cai na linha sob o ponteiro; insere,
  apaga e quebra linha no meio. cada frame confere que cada nó de texto é
  a linha do documento naquela posição, que as linhas estão a uma altura
  de linha uma da outra e que a margem numera as linhas certas. os quatro
  falhavam antes da correção de f64.
- `rust/engine/src/drawing/text/cluster_cache.rs`: acerto e chave, despejo da
  menos usada no orçamento, linha maior que o orçamento e troca contada
  uma vez. `tests_measure.rs`: `clusters_styled` molda uma vez por texto
  de linha e uma edição molda só a linha editada.
- `rust/engine/tests/editor.rs`: o benchmark, `#[ignore]`.
- os testes do editor pequeno, do bidi (fase 3) e do `TextField` passam
  sem mudança.

### gate e regress

`./tool/tasks/fullcheck.sh` verde com o ambiente do briefing: `cargo test
--workspace` 3144 passed, 0 failed, 28 ignored (eram 3136 e 27: mais os
8 testes e o benchmark); clippy, fmt e wasm verdes; perna urnaui 85
passed, 0 failed, 1 ignored. `regress -- check` depois de tocar
`tool/bench/regress/src/main.rs`: 176 unchanged, 0 changed.
fora do gate: `cargo check -p engine -p engine --target
aarch64-apple-ios-sim` com `-D warnings` verde; `cargo ndk -t arm64-v8a
check -p uigallery --lib` compila, com os dois avisos antigos de código
morto. disco: 9,3 GB livres.

### pendências

- o `ScrollState` guarda o deslocamento em f32 (engine, fora desta fase).
  com um milhão de linhas de 19,6 px, perto do fim ele anda em passos de
  2 px: um delta de trackpad menor que 1 px se perde, e o fim do
  documento fica a até 1 px da borda. as linhas entre si ficam certas
  (a conta delas é f64). resolver pede o deslocamento em f64 no
  `ScrollState` ou um deslocamento do editor em linha mais fração; os
  dois mudam API pública, então é decisão.
- com tudo selecionado a cena ainda custa 7 vezes a da roda simples
  (0,148 contra 0,021 ms): o shaping saiu, mas o `VisualLine` refaz as
  paradas do caret de cada linha a cada frame. guardar o `VisualLine`
  pede um construtor a partir dos clusters em
  `rust/engine/src/widgets/form/text_input/visual.rs`, fora dos arquivos desta
  fase.
- linha muito longa (um arquivo minificado de megabytes numa linha só)
  não foi medida: o editor molda a linha inteira, sem virtualização
  horizontal, e uma linha acima de 4 MiB de clusters não entra no cache.
- o editor não quebra linha: a virtualização depende da altura fixa por
  linha. quebra suave num documento grande pede um índice de alturas;
  medido sem quebra, como o briefing pede.
- abrir continua O(n) no texto (17,5 ms de cena para 57 MiB) e o chamador
  segura o `String` e o rope ao mesmo tempo enquanto abre; carregar em
  pedaços, direto do arquivo, não foi feito.
- o IME (fase 4) continua aberto e pede o brenner.

## fase 7 integrada (orquestrador, 2026-09-25)

fecha o item "benchmark de um milhão de linhas com números". o editor já
emitia só as linhas visíveis; a fase corrigiu a posição das linhas (f64,
errava a partir da linha 100 mil), pôs um cache de shaping por texto de
linha (4 MiB, o menos usado sai) e abriu o documento sem cópia. com um
milhão de linhas, em release no M4: abrir 21 ms (pico de 65 MiB, antes
121), rolagem pela roda 0,36 ms por frame, edição no meio até o frame
seguinte 0,27 ms; rolar e editar custam o mesmo com 10 mil e com um
milhão. tabela completa e comando acima.

validação do orquestrador: diff lido (a chave do cache cobre todo campo
do `TextStyle` e a largura; nenhuma API pública mudou; os quatro testes
de um milhão de linhas falhavam antes da correção em f64); 176 baselines
iguais.

continua aberto no critério: o roteiro de IME executado (fase 4), que
pede o brenner. pendência: a rolagem do engine em f32 anda em passos de
2 px perto do fim de um milhão de linhas (mudar é API pública).

o agente gastou uns 190 mil tokens.
