---
id: task-74/phase-2-noto-sans
title: "phase 2: embedded Noto Sans"
project: plev
type: section
parent: task-74
status: reference
date: 2026-09-25
date_basis: text
recorded: 2026-09-23
topics: [text, fonts]
relations: [adr-0071]
evidence: [3882aedd, a0f7a0b6, fc304299, caad36f2]
source: docs/log/steps/task-74-text-and-editing.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# phase 2: embedded Noto Sans

Embeds Noto Sans subsets for Arabic, Hebrew, Devanagari, simplified Chinese, Japanese and Korean, in Regular and Bold, by brenner's decision, so that the embedded path (wasm, android and iOS) stops showing tofu in these scripts. cosmic-text's fallback finds the faces in the last step, without changing the library, and Han follows the load order SC, JP, KR. The desktop keeps using the system fonts, and emoji is left for phase 5.

## fase 2: a sonda

data: 2026-09-25. branch `task/74b-noto` sobre `5dad7ec1`. decisão do
brenner: "Noto, amigo. Noto, sans". escritas: árabe, hebraico, devanágari,
chinês simplificado, japonês e coreano; emoji fica na fase 5. as linhas
citadas da biblioteca são do fonte travado no Cargo.lock: cosmic-text
0.18.2 e fontdb 0.23.0, no registro do cargo.

### (a) como o cosmic-text escolhe a face de fallback

por palavra. `shape_run` (cosmic-text `src/shape.rs:273`) junta as
escritas do trecho na ordem em que aparecem, sem `Common`, `Inherited`,
`Latin` e `Unknown` (`shape.rs:288-297`), abre um `FontFallbackIter`
(`shape.rs:306`), molda com a primeira face e, enquanto sobra cluster sem
glifo, molda o trecho inteiro com a próxima face e copia só os clusters
que ela cobre (`shape.rs:326-400`). a ordem das faces, em
`FontFallbackIter::next_item` (`src/font/fallback/mod.rs`):

1. a família pedida pelo estilo (`mod.rs:305`), só com peso exato
   (`default_font_match_key`, `mod.rs:279`);
2. a lista por escrita do `Fallback` (`mod.rs:419`), só faces de peso
   exato (o filtro `font_weight_diff == 0`, `mod.rs:302`);
3. a lista comum do `Fallback` (`mod.rs:446`), também de peso exato;
4. todas as faces do banco, menos as proibidas (`mod.rs:462-463`), sem
   filtro de peso, na ordem de `FontMatchKey`: derivada dos campos
   `not_emoji`, `font_weight_diff`, `font_stretch_diff`,
   `font_style_diff`, `font_weight`, `font_stretch`, `id`
   (`src/font/system.rs:18-27`, ordenada em `system.rs:370`, com a face
   da consulta levada para a frente em `system.rs:380-394`). o `id` do
   fontdb é chave de slotmap (`fontdb src/lib.rs:92-95`), crescente na
   ordem de carga num banco novo.

qual `Fallback` cada caminho usa: o motor constrói o caminho embutido com
`FontSystem::new_with_locale_and_db("en-US", Database::new())`
(`rust/engine/src/drawing/text/system.rs:91-95`, `measure.rs:121-125`), que é
`PlatformFallback` do alvo (`system.rs:266-267` do cosmic-text). o alvo
escolhe a lista em `fallback/mod.rs:12-26`: wasm e android caem em
`other.rs`, listas vazias (`other.rs:30-42`); ios é `unix` e cai em
`unix.rs`, que nomeia "Noto Sans Arabic", "Noto Sans Devanagari" e "Noto
Sans Hebrew" (`unix.rs:76`, `:88`, `:98`) e, para han, kana e hangul,
"Noto Sans CJK SC/JP/KR" (`unix.rs:56-68`, `:95-102`). o desktop usa
`FontSystem::new()` com `macos.rs`, que nomeia fontes do sistema
(`macos.rs:61-109`).

o que faz o embutido achar as faces Noto sem as fontes do sistema: o
passo 4. em wasm e android as listas são vazias, então toda escrita sem
glifo na face pedida chega ao passo 4, que percorre todas as faces do
banco e pega a primeira que cobre o cluster; como as faces Noto são as
únicas com glifo dessas escritas, é nelas que o cluster cai, em qualquer
peso (o passo 4 não filtra peso, e a ordem por `font_weight_diff` faz 500
cair na Regular e 600 na Bold). no ios, árabe, hebraico e devanágari
batem no passo 2 pelo nome, em 400 e 700; os CJK não batem (a família do
subset é "Noto Sans SC", não "Noto Sans CJK SC") e vão ao passo 4. o
resultado é o mesmo nos três alvos: a única face que cobre, ou, para o
han, a primeira na ordem de carga (item b). nenhuma mudança na
biblioteca, nenhum `Fallback` próprio.

um `Fallback` próprio do embutido (kana para JP, hangul para KR, han pelo
locale) não compila sem pacote: o método `script_fallback` recebe
`unicode_script::Script` (`fallback/mod.rs:9`, `:76`), o cosmic-text não
reexporta esse tipo, e o engine não depende de `unicode-script`. o
pacote já está no Cargo.lock (0.5.8), mas pô-lo no `Cargo.toml` do engine
está fora do que esta rodada pode tocar. fica como pendência (ver
handoff).

### (b) a ordem das faces CJK para o han unificado

no embutido, o han não acha nome em lista nenhuma (vazias em wasm e
android; no ios os nomes "Noto Sans CJK *" não batem) e cai no passo 4,
onde as faces Noto Regular empatam em tudo menos no `id`. então a ordem é
a de carga em `register_embedded_fonts`: SC, depois JP, depois KR. SC vem
primeiro porque é o que o próprio cosmic-text faz com `en-US`: as tabelas
de unificação han dele mandam qualquer locale que não seja `ja`, `ko`,
`zh-HK` ou `zh-TW` para o chinês simplificado (`unix.rs:56-68`,
`macos.rs:45-57`), e o locale embutido é `en-US`. o subset KR não tem
hanja, então para o han ele nunca entra.

consequência medida: uma palavra chinesa sai inteira em SC; uma palavra
japonesa sai com kanji e pontuação CJK em SC e kana em JP, porque SC é
avaliada antes e cobre o que tem. é o mesmo que o cosmic-text faz no
desktop com `en-US` quando a palavra começa por kanji ("世界は" dá
Arial Unicode MS e Hiragino Sans aqui, abaixo). o conserto é o
`Fallback` próprio do item (a).

### (c) o que o desktop passa a mostrar

a fonte do sistema, como antes. `macos.rs` nomeia uma face do sistema
por escrita (Geeza Pro, Arial, Devanagari Sangam MN, Hiragino Sans, Apple
SD Gothic Neo; PingFang SC para han, que o fontdb não acha neste macOS e
cai na lista comum, Arial Unicode MS), e o passo 2 vem antes do 4. em 500
e 600 os passos 2 e 3 não acham peso exato e o passo 4 decide pela
distância de peso e depois pelo `id`; as faces do sistema carregam antes
das embutidas (`id` menor), então empatadas elas ganham. as faces por
peso, antes desta fase, neste mac (Inclusive Sans, `probe::shape_report`,
`Faces::Desktop`):

| linha | 400 | 500 | 600 | 700 |
|---|---|---|---|---|
| السلام عليكم... | Geeza Pro | Damascus | Damascus | Geeza Pro |
| שלום עולם | Arial | Lucida Grande | Lucida Grande | Arial |
| नमस्ते दुनिया | Devanagari Sangam MN | Kohinoor Devanagari | Kohinoor Devanagari | Devanagari Sangam MN |
| 你好，世界 | Arial Unicode MS | Hiragino Sans e Apple SD Gothic Neo | Hiragino Sans GB e Apple SD Gothic Neo | Songti SC e Apple SD Gothic Neo |
| こんにちは、世界 | Hiragino Sans e Arial Unicode MS | Hiragino Sans e Apple SD Gothic Neo | idem | idem |
| 안녕하세요, 세계 | Apple SD Gothic Neo | idem | idem | idem |

o chinês em 700 sai em Songti SC, uma serifada; é o desktop de hoje, e
esta fase não mexe nele (fonte de sistema por configuração está fora do
escopo). o regress: nenhuma página do showcase tem texto dessas escritas
(só testes de `rope`, `sem` e do editor de `comps` têm), então nenhuma
das 176 baselines passa por esse caminho; a rodada confirma com o
`regress check` no handoff.

### (d) se o `text_audit` mede o caminho embutido nesta máquina

o shaping, sim: `probe::shape_report` com `Faces::Embedded` já monta um
`FontSystem` só com as faces do repositório (`probe.rs:336-347`). o
pixel, não: `probe::render` passa pelo `headless::Renderer`
(`probe.rs:207`), que cria o `TextSystem` dele
(`rust/engine/src/drawing/headless/mod.rs:157`) e o guarda privado, e
`TextSystem::new` escolhe as fontes do sistema por `cfg` no desktop
(`system.rs:84-95`). falta uma chave em tempo de execução. o que esta
rodada faz, só em `src/text`: um construtor único do `FontSystem` em
`fonts.rs`, usado pelo `TextSystem`, pelo `TextMeasurer` e pela sonda,
com uma chave por thread que a sonda liga em volta do `Renderer::new`
(`probe::render_with(Faces::Embedded, ...)`). fora do `cfg` do desktop a
chave não existe; nada fora de `src/text` muda.

## handoff da fase 2: Noto Sans embutida

o critério "corpus renderizado sem tofu nas escritas decididas", com gate
verde, fechou para árabe, hebraico, devanágari, chinês simplificado,
japonês e coreano. a ADR é [`0071-embedded-fonts-by-script.md`](../../adr/Rendering/0071-embedded-fonts-by-script/0071-embedded-fonts-by-script.md)
(proposed).

### commits

- `3882aedd` task-74: phase 2 probe, how the embedded path finds a face per script
- `a0f7a0b6` task-74: tool/tasks/subsetter.py and the Noto subsets for six scripts, with the OFL
- `fc304299` task-74: register the Noto faces, one font-system constructor, and the probe's embedded render
- `caad36f2` task-74: the text audit holds the six scripts to zero tofu and draws the embedded path
- o commit da ADR e deste handoff

### a sonda, curta

(a) o embutido acha as faces Noto no último passo do fallback do
cosmic-text (todas as faces, por distância de peso e depois ordem de
carga), sem nome de família e sem mudar a biblioteca; 500 cai na Regular,
600 na Bold. (b) o han vai na ordem de carga: SC, JP, KR, porque com
`en-US` o próprio cosmic-text escolhe o chinês simplificado; palavra
japonesa sai com kanji de SC e kana de JP. (c) o desktop não muda: as
listas do macOS acham a fonte do sistema antes; as faces por escrita e
peso (400 a 700) saíram iguais antes e depois nas oito linhas medidas.
(d) o pixel do embutido agora roda nesta máquina, por
`probe::render_with(Faces::Embedded, ...)`.

### arquivos novos

| arquivo | bytes |
|---|---|
| `rust/engine/assets/fonts/NotoSansArabic-Regular.ttf` | 217.320 |
| `rust/engine/assets/fonts/NotoSansArabic-Bold.ttf` | 243.820 |
| `rust/engine/assets/fonts/NotoSansHebrew-Regular.ttf` | 25.204 |
| `rust/engine/assets/fonts/NotoSansHebrew-Bold.ttf` | 25.204 |
| `rust/engine/assets/fonts/NotoSansDevanagari-Regular.ttf` | 225.236 |
| `rust/engine/assets/fonts/NotoSansDevanagari-Bold.ttf` | 231.752 |
| `rust/engine/assets/fonts/NotoSansSC-Regular.otf` | 946.616 |
| `rust/engine/assets/fonts/NotoSansSC-Bold.otf` | 956.892 |
| `rust/engine/assets/fonts/NotoSansJP-Regular.otf` | 794.888 |
| `rust/engine/assets/fonts/NotoSansJP-Bold.otf` | 813.168 |
| `rust/engine/assets/fonts/NotoSansKR-Regular.otf` | 350.204 |
| `rust/engine/assets/fonts/NotoSansKR-Bold.otf` | 361.128 |
| `rust/engine/assets/fonts/LICENSE-Noto-OFL.txt` | 4.823 |
| `tool/tasks/subsetter.py` | 9.926 |
| [`0071-embedded-fonts-by-script.md`](../../adr/Rendering/0071-embedded-fonts-by-script/0071-embedded-fonts-by-script.md) | a ADR |

fontes embutidas: antes 1.744.532 bytes (834.203 com gzip -9), depois
6.935.964 bytes (4.873.679 com gzip -9). nenhum arquivo passa de 1 MB.
`tool/tasks/subsetter.py --src target/fonts-noto --check` refaz tudo num
diretório temporário e compara byte a byte: 13 arquivos iguais.

### o audit, notdef antes e depois (caminho embutido, 16 px, 400)

| linha | Inclusive Sans | Inter |
|---|---|---|
| árabe | 24 para 0 | 24 para 0 |
| hebraico | 8 para 0 | 8 para 0 |
| hebraico com niqqud | 7 para 0 | 7 para 0 |
| devanágari नमस्ते दुनिया | 12 para 0 | 12 para 0 |
| devanágari क्षत्रिय | 8 para 0 | 8 para 0 |
| chinês | 5 para 0 | 5 para 0 |
| japonês こんにちは、世界 | 8 para 0 | 8 para 0 |
| japonês テキスト | 4 para 0 | 4 para 0 |
| coreano | 7 para 0 | 7 para 0 |
| bidi abc אבג def | 3 para 0 | 3 para 0 |
| bidi hello مرحبا 123 world | 5 para 0 | 5 para 0 |

o teste novo `decided_scripts_have_no_tofu` segura esses zeros nas duas
famílias e em 400, 500, 600 e 700. o golden (`text_audit.txt`) mudou só no
cabeçalho (os bytes) e nas linhas dessas escritas; latim, marcas
combinantes, emoji e a tabela de kerning ficaram iguais. o devanágari agora
forma os conjuntos (क्षत्रिय, 8 caracteres, 4 glifos) e o árabe mede 167,4
px em vez de 251,7 de tofu. emoji continua tofu no embutido (fase 5).

pixel: as quatro folhas saem em
`/Users/nn/Dev/hoff/plevdev/target/agents/one/text-audit-74b/`; a do
embutido é `Inclusive-Sans-embedded.png` (e `Inter-embedded.png`). as 38
linhas do embutido ficam "ok" (tinta dentro da largura medida), e toda
linha das escritas decididas tem tinta; o teste afirma as duas coisas.

### gate e regress

`./tool/tasks/fullcheck.sh` verde com o ambiente do briefing: `cargo test --workspace`
3079 passed, 0 failed, 27 ignored (a base tinha um teste a menos: este
trabalho soma `decided_scripts_have_no_tofu` e não tira nenhum); clippy,
fmt e wasm verdes; perna urnaui 85 passed, 0 failed, 1 ignored.
`regress -- check`: 176 unchanged, 0 changed. fora do gate:
`cargo check -p engine --target aarch64-apple-ios-sim` com `-D warnings`
verde; `cargo ndk -t arm64-v8a check -p uigallery --lib` compila, com dois
avisos de código morto que já existiam e ficam fora do escopo
(`rust/engine/src/runtime/window/mod.rs:57`, `apps/uigallery/src/shell/mod.rs:407`).
disco: 13 GB livres antes do gate, 11 GB no fim.

### linha para o índice de ADR

```
| 2026-09-25 | [embedded-fonts-by-script](embedded-fonts-by-script.md) | proposed | Noto Sans embutida por escrita (decisão do brenner): árabe, hebraico, devanágari, SC, JP e KR em Regular e Bold, subsets reprodutíveis por tool/tasks/subsetter.py (sha256 das fontes, conjuntos pelos codecs da stdlib); de 1,7 MB para 6,9 MB embutidos; o fallback acha as faces no último passo do cosmic-text, han na ordem SC, JP, KR; o desktop segue com as fontes do sistema; emoji fica para a fase 5 |
```

### pendências

- palavra japonesa sai com kanji de SC. o conserto é um `Fallback` próprio
  do embutido (kana para JP, hangul para KR, han pelo locale), que precisa
  de `unicode-script` como dependência direta do engine: o pacote já está
  no Cargo.lock (0.5.8), mas é mudança no `Cargo.toml`, decisão do
  orquestrador.
- o desktop desenha essas escritas com fontes do sistema, diferentes das
  do embutido (e o chinês em 700 sai em Songti SC, serifada). preferir Noto
  no desktop é uma decisão à parte.
- a cobertura é um conjunto: hanzi fora do GB2312 nível 1, kanji fora do
  JIS nível 1 e os jamo de compatibilidade do coreano (ㅋ, U+3131 a U+318E)
  não estão; alargar é editar `tool/tasks/subsetter.py` e rodar de novo.
- o wasm cresce uns 5,2 MB crus (uns 4 MB com gzip); não medi o wasm de
  release (o briefing proíbe `--release`).
- herdadas da fase 1 e ainda abertas: o travessão no aviso de fallback de
  `atlas.rs` e as três cópias da montagem de `Attrs` (agora a sonda e o
  medidor dividem o construtor do `FontSystem`, mas não o de `Attrs`).
- os outros itens do critério de aceitação (bidi no cursor, IME, emoji,
  um milhão de linhas) são das fases 3, 4, 5 e 7.

## fase 2 integrada (orquestrador, 2026-09-25)

fecha o item "corpus renderizado sem tofu nas escritas decididas" do
critério de aceite. a decisão foi do brenner ("Noto, amigo. Noto, sans"):
árabe, hebraico, devanágari, chinês simplificado, japonês e coreano em
Noto Sans, Regular e Bold, subsets de `tool/tasks/subsetter.py`.

validação do orquestrador: diff lido (as faces entram por último no
banco, emprestadas do binário sem cópia; `TextSystem`, `TextMeasurer` e a
sonda passam pelo mesmo construtor); o PNG do caminho embutido aberto: as
seis escritas desenham, o árabe ligado, o devanágari com os conjuntos, o
hebraico com niqqud; o emoji continua tofu, que é da fase 5. o desktop
não muda (as 176 baselines iguais).

os itens que continuam abertos no critério: testes de movimento bidi
(fase 3), roteiro de IME executado (fase 4, pede o brenner), emoji
colorido com pixel medido (fase 5), benchmark de um milhão de linhas
(fase 7). as fases 6, 8 e 9 não têm item no critério.

o agente gastou uns 250 mil tokens.
