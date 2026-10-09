---
type: reference
tags: [learnings, comps, design-system, tokens, refactor, session]
date: 2026-09-19
status: reference
---

# sessão comps: o que funcionou e o que mordeu

registro da sessão que criou o crate `comps` e tirou os literais dos
widgets (adr comps-one-design-system-crate, inventário em
docs/catalog.md). escrito pra quem for fazer a próxima migração desse
tamanho, humano ou agente.

## o caminho que funcionou

1. **catálogo antes de código.** meia hora de grep produziu a tabela do
   docs/catalog.md: onde cada componente vivia, quem duplicava quem, quais
   números estavam soltos. a tabela virou o plano; cada linha virou um
   move, um merge ou um delete. sem ela o refactor teria virado uma
   sequência de descobertas no meio do caminho.
2. **decisão de fronteira primeiro, com o usuário.** "corte limpo" (os
   widgets saem do engine) contra "camada por cima" (comps re-exporta o
   engine) muda todo o trabalho. uma pergunta, uma resposta, e o resto é
   mecânico.
3. **tokens no engine antes de mover qualquer widget.** os scales
   (`shape`, `control`, `size`, `duration`, `layout`, `shadows`) entraram
   com testes contra os valores do spec (os testes do `ide/theme.rs`
   migraram pra `tests_scales.rs`). quando os widgets moveram, cada
   literal já tinha um nome pra apontar.
4. **mover com `git mv`, reescrever caminho com script, compilar em
   ciclos curtos.** um python que troca `crate::compositor` por
   `engine::compositor` e `super::{Rect, ...}` por `crate::core::{...}`
   fez 40 arquivos numa passada; depois o compilador guiou. cada
   categoria (action, form, nav, content, feedback) fechou com
   `cargo check -p comps` antes da próxima.
5. **`&Theme` no `handle_event` quando a geometria vem do tema.** o
   primeiro instinto foi cachear rects do último render; o padrão do repo
   é "uma geometria pra evento e pixel", então select, tabs, slider, tree,
   list, modal, menu, toast e text field passaram a receber o tema no
   evento. custou 172 call sites nos consumidores, resolvidos por script
   com o compilador listando as linhas.
6. **matriz tema x viewport como rede.** `tests/matrix.rs` renderiza todo
   widget sob 13 temas em 390 px e 1440 px e exige nós finitos. pegou
   zero divisões por zero porque os tokens já evitavam; vale como
   contrato pros próximos widgets.
7. **novo widget nasce com teste no mesmo commit** (`tests/components.rs`):
   avatar, badge, sidebar (rail, drawer, scroll da banda), breadcrumb
   (colapso), table (drop de coluna por prioridade), shell (breakpoint,
   safe area, drawer exclusivo).

## armadilhas

- **disco cheio derruba o `cargo test` com erro de query cache.** o
  volume de dados estava em 100 % (119 MB livres). `cargo clean` neste e em
  três outros repos liberou 18 GB. sintoma: `failed to write query cache
  ... No space left on device`. checar `df -h /System/Volumes/Data`
  antes de uma sessão longa.
- **rust do Homebrew não tem `rustup` nem o target wasm32.** a perna
  `cargo check --target wasm32-unknown-unknown -p showcase` do gate não
  roda local; `script/gate` já pula com aviso e o CI verifica. não
  confundir com "wasm quebrou".
- **`cargo fmt` reformata arquivos que o agente acabou de escrever**; o
  `Edit` seguinte falha se o texto antigo mudou de indentação. rodar fmt
  no fim de cada lote, não no meio.
- **`grep --include` não funciona no zsh deste repo** (glob sem match
  aborta o comando). usar `grep -r ... dir | grep pattern` ou `find`.
- **`echo ====` quebra no zsh** (expansão de `=`). aspas.
- **`git rm` recusa arquivo com conteúdo staged diferente** depois de um
  `git mv`; `-f` quando a remoção é intencional.
- **`allow(dead_code)` crate-wide escondia código nunca ligado.** a seção
  App do showcase tinha `handle_text`/`handle_enter`/`tick` com o allow e
  ninguém conseguia digitar no campo. a varredura final ligou tudo e tirou
  o allow; a convenção agora proíbe o allow de crate.
- **`TextMeasurer::elide_path` do ide só derrubava um segmento do path**
  (o loop `for keep in (1..n-1)` gera um único `…`). o teste com largura
  derivada de medição pegou; o novo tenta lead + tail com o máximo de
  segmentos.
- **screencapture no macOS capturou o wallpaper**, não a janela (permissão
  de captura). a validação de pixel fica pelos testes headless ou pelo
  `text_probe` (docs/how-to/validate-visuals-by-pixel.md).

## números pra calibrar a próxima

- 164 arquivos, +5 890 / -9 622 linhas na primeira rodada.
- ide: -2 160 linhas de tema e componentes locais.
- testes: 855 -> 1 414 no workspace, 64 -> 80 no urnaui.
- tempo de rebuild completo depois do `cargo clean`: ~2 min em debug.
