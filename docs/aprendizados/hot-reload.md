---
id: aprendizado-hot-reload
title: "sessão hot reload: o que funcionou e o que mordeu"
project: plev
type: aprendizado
status: referencia
date: 2026-09-22
date_basis: cabecalho
topics: [learnings, hot-reload, engine, dev-loop, session]
relations: []
source: docs/log/learnings/hot-reload.md
source_revision: ec0a224c
revised: 2026-10-30
review: pendente
---

# sessão hot reload: o que funcionou e o que mordeu

registro da sessão que ligou o hot reload de verdade no plev (adr
hot-reload-on-by-default-in-gpu-context, task-56). escrito pra quem for
mexer no dev loop, humano ou agente, e pra não repetir a manhã.

## o caminho que funcionou

1. **auditar o que existe antes de escrever.** os três ADRs de 2024
   diziam "implementado"; ninguém tinha rodado. meia hora de leitura
   achou seis defeitos (feature desligada, watcher num `App` que nenhum
   app usa, diretório de shader inexistente, formato srgb errado no
   reload, poll dormindo, layer limpa sem redesenhar). corrigir isso
   valia mais que qualquer mecanismo novo.
2. **o ponto de integração é o que todo app compartilha.** showcase,
   ide e urnaui têm cada um o próprio `ApplicationHandler`; o que eles
   compartilham é `GpuContext`, `EffectProcessor` e `present_frame`. o
   watcher foi pra dentro do `GpuContext` e o poll pra dentro do
   `present_frame`. zero linha nos apps.
3. **acordar o loop de qualquer thread é `request_redraw`.** `Window` é
   `Send + Sync`; o watcher chama `window.request_redraw()` e o frame
   acontece agora, sem proxy, sem estender `EventLoopProxy` pra native.
   channel polling em `about_to_wait` (o adr-3 de 2024) só roda enquanto
   há input.
4. **reload de shader exige `mark_all_dirty`, não `invalidate`.**
   `invalidate` força um frame; layer com hash de cena igual não
   redesenha e mantém a textura desenhada pelo shader velho.
5. **`cfg(hot_reload)` de um build.rs em vez de `cfg(feature)`.** a
   feature fica ligada por padrão e o build.rs (`PROFILE`,
   `CARGO_CFG_TARGET_OS`) decide onde o código existe. release, wasm e
   mobile não compilam nada disso e o dev não passa flag.
6. **literal por valor, no destino.** a ideia do rsx-hotreload do dioxus
   (`collect.rs`: apaga a parte recarregável, compara o resto) sem o
   macro: skeleton do token stream com literais em branco (hash) +
   lista de literais, por arquivo, no start. skeleton igual = só
   literal mudou. o delta entra em `TextNodeKey`, `TextMeasurer`,
   `push_to_layer` e no clear. o `TextMeasurer` é obrigatório: sem ele
   o texto novo desenha na caixa velha.
7. **clonar a referência antes de escrever.** `git clone --depth 1
   --sparse` do dioxus no scratchpad, ler `packages/rsx-hotreload/
   collect.rs` e `diff.rs`, portar a ideia. quarenta linhas de leitura
   pouparam uma tarde de invenção.
8. **`-p $CARGO_PKG_NAME` no rebuild.** `cargo run` exporta
   `CARGO_PKG_NAME`, `CARGO`, `CARGO_MANIFEST_PATH` pro binário. um
   `cargo build --bin showcase` na raiz do workspace unifica features de
   todos os membros e recompila engine, comps e ide (10s); com `-p` só a
   crate tocada (2s).
9. **validar por captura de janela com pid antes e depois.** o critério
   de hot reload é "mesmo processo, estado intacto". `pgrep -x showcase`
   antes e depois, `screencapture -l<windowid>` e pixel na captura
   (how-to validate-visuals-by-pixel). "parece que trocou" não vale.

## o que mordeu

- **rebuild + exec não é hot reload.** foi entregue como se fosse. a
  definição que vale é a do HMR: processo vivo, estado vivo, só o que
  mudou troca. restart automático é útil e tem nome: restart.
- **propor toolchain de terceiro pra uma engine autoral.** subsecond
  precisa do `dx` e de um linker próprio no loop; dylib com
  save/restore de estado é restart disfarçado. os dois foram propostos
  e os dois estavam errados pra este projeto. o design de 2024 já tinha
  a resposta (a camada que recarrega é dado, não código compilado).
- **`pkill -x showcase` derruba a janela do usuário também.** matar por
  pid próprio, nunca por nome.
- **capturas de tela pegam o app errado.** `screencapture -x` sem
  `-l<windowid>` gravou a janela do claude em vez do showcase. o
  how-to já dizia isso.
- **cliques "fantasma" eram o usuário.** sidebar dobrada e tema trocado
  na janela relançada pareciam input espúrio do exec; um log de
  `MouseInput` mostrou cliques reais espaçados de 1s. logar antes de
  teorizar.
- **sed e editores salvam por arquivo oculto.** `.!69353!quad.wgsl`
  chegou no watcher e virou erro de leitura. filtrar nome que começa
  com ponto e dedup no batch.
- **clippy no código que nunca compilou.** ligar uma feature morta
  desde 2024 trouxe `uninlined_format_args` em dez lugares e
  `type_complexity` no novo. `cargo clippy --fix` resolve o primeiro
  lote; o gate é a única prova de que o cfg existe.
- **um comentário de linha num teste comenta o resto da linha.**
  `fn size(  ) // spaced` engoliu `-> f32 { 16.0 }` e o teste de
  "whitespace não muda nada" virou teste de "código mudou". bloco
  `/* */` quando o fragmento continua.

## regras que ficaram

- dev-only é `#[cfg(hot_reload)]`, nunca `feature = "hot-reload"`
  (rust-conventions.md, crate boundary).
- o chokepoint de um override de dev mantém o `cfg` numa linha só; o
  caminho de produção lê sem ele.
- `docs/log/steps/gap-1-hot-reload.md` virou
  `checked/task-56-hot-reload-done.md`.
