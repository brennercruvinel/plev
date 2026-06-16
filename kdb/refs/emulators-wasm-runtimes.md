---
project: phi
audience: [ai-agents, contributors]
status: reference
last-updated: 2026-03-11
domain: wasm
---

# reference analysis: emuladores e WASM runtimes

**data:** 2026-03-11
**contexto:** pesquisa para suporte futuro a fortran, inform 6/7 e outras linguagens rodando no browser via φ.

---

## escopo

como emuladores e runtimes de linguagens funcionam dentro de WASM, referencia para arquitetura futura de plugins/linguagens no φ. foco em: rendering loop, interface host-guest, gerenciamento de memoria e frame timing.

---

## repositorios analisados

### wasmboy, 1.476 stars, pre-1.0

**repo:** `torch2424/wasmboy`
**licenca:** gpl-3.0
**linguagem principal:** assemblyscript (compila para webassembly)
**criado:** 2018-01-24

**o que e:** emulador de game boy / game boy color escrito inteiramente em assemblyscript, que compila para WASM. demos construidos com preact e svelte. roda tanto no browser quanto em node.js (modo headless).

**arquitetura:**
- **core** (`core/`): emulacao de CPU, GPU, apu, memoria, interrupts e timers, tudo em assemblyscript. compila para um unico modulo WASM.
- **lib** (`lib/`): camada javascript que orquestra o core WASM. dividida em modulos: graphics, audio, controller, memory, plugins, worker.
- **rendering loop:** usa `requestAnimationFrame` (polyfill `raf` para node). o loop chama `WasmBoyGraphics.renderFrame()` + `WasmBoyController.updateController()` a cada frame. suporta frame skip configuravel.
- **graphics:** renderiza em canvas 2d. o worker de emulacao produz um `Uint8ClampedArray` com os pixels (160x144 do game boy). o thread principal copia via `ImageData.set()` e faz `putImageData()`. CSS `image-rendering: pixelated` para upscale sem blur.
- **web workers:** emulacao roda em worker separado. comunicacao via `postMessage` com `SharedArrayBuffer` quando disponivel (fallback para transferable buffers). o worker envia frame buffers prontos para o thread principal.
- **plugin system:** hooks para `canvas`, `graphics`, e outros pontos de extensao. permite intercecao e transformacao dos dados de frame.
- **pause/resume:** detecta `visibilitychange` no document para auto-pause quando tab perde foco.

**relevancia para φ:**
- modelo de separacao emulacao (worker) vs renderizacao (main thread) e diretamente aplicavel.
- o padrao de enviar frame buffers via postmessage/sharedarraybuffer e relevante para qualquer linguagem que produza output grafico.
- frame skip e um conceito util se o runtime guest for mais lento que 60fps.
- plugin hooks mostram como extensibilizar sem acoplar.

**insight principal:** a emulacao roda desacoplada do rendering, o WASM core "ticking" em worker produz frames, e o main thread consome no ritmo do requestanimationframe. isso evita que emulacao lenta trave o UI e que emulacao rapida desperdice GPU renders.

**limitacao:** canvas 2d (nao webgpu/webgl). nao resolve acesso a GPU do guest, o guest produz pixels em memoria linear e o host renderiza. para φ, o guest precisaria produzir scene nodes, nao pixels brutos.

---

### waforth, 581 stars, v0.20.1

**repo:** `remko/waforth`
**licenca:** MIT
**linguagem principal:** webassembly text format (.wat) puro
**criado:** 2018-05-15

**o que e:** interpretador e compilador dinamico de forth, escrito inteiramente em webassembly (formato texto .wat). o compilador gera modulos WASM on-the-fly. 14kb o core (7kb gzip), mais 15kb para wrapper JS.

**arquitetura:**
- **core** (`src/waforth.wat`): um unico arquivo wat contendo interpretador + compilador. usa subroutine threading (nao direct threading, pois WASM nao permite jumps desestruturados).
- **compilador JIT:** quando em modo compile para uma word, gera instrucoes WASM em formato binario diretamente na memoria. a word compilada e empacotada como modulo WASM separado e enviada ao loader.
- **loader** (javascript): usa a webassembly JS API para carregar modulos compilados dinamicamente. gerencia uma function table compartilhada, cada nova word recebe um slot nessa tabela.
- **shell** (`src/web/waforth.ts`): classe typescript que encapsula o modulo WASM. fornece i/o primitivos (read/write character) e expoe `interpret()` para executar codigo forth.
- **host bindings:** `forth.bind("name", callback)` vincula funcoes JS a words forth. `bindAsync` para operacoes assincronas (ex: fetch) com continuations via execution tokens.
- **code word:** permite escrever webassembly inline dentro de forth, acesso direto ao stack WASM operand.
- **standalone shell:** usa wasmtime como engine nativo, tambem funciona fora do browser.
- **compilacao AOT:** `waforthc` compila forth para executavel nativo via WASM como representacao intermediaria.

**relevancia para φ:**
- modelo exemplar de linguagem compilando para WASM dentro de WASM. diretamente aplicavel ao cenario fortran/inform.
- o padrao de host bindings (bind/bindasync) e o exato mecanismo que φ precisaria: a linguagem guest chama funcoes do host (renderizacao, i/o, file system).
- a function table compartilhada e o mecanismo de comunicacao, cada modulo compilado registra funcoes nela.
- 14kb para um runtime completo demonstra que e viavel embeder runtimes leves.

**insight principal:** WASM nao tem JIT nativo, entao waforth contorna isso compilando cada word como um modulo WASM separado e carregando dinamicamente via JS API. esse padrao (compile -> load module -> register in table) e o unico caminho para compilacao dinamica em WASM ate a proposta de JIT compilation ser aprovada.

**limitacao:** o carregamento dinamico de modulos requer javascript (webassembly.instantiate e async). em ambiente puramente WASM (sem JS host) nao e possivel. para φ rodando via WASM, o host φ (rust compilado para WASM) precisaria de uma bridge JS para carregar modulos dinamicos.

---

### chasm, 186 stars, v1.4.0

**repo:** `CharlieTap/chasm`
**licenca:** apache-2.0
**linguagem principal:** kotlin (classificada como webassembly pelo github por causa dos .wasm de teste)
**criado:** 2024-01-16

**o que e:** runtime webassembly construido em kotlin multiplatform. roda em android, jvm, ios, linux, macos e windows. suporta wasm 3.0 (exceto memory64 e SIMD vetorial). plugin gradle gera interface kotlin tipada a partir de modulos .wasm.

**arquitetura:**
- **modulos gradle**: `decoder` (parse do binario), `ast` (representacao interna), `executor` (interpretador), `compiler` (nao documentado), `chasm` (API publica).
- **embedding API:** `module(bytes)` -> `store()` -> `instance(store, module)` -> `invoke(store, instance, "funcName")`. API simples sem boilerplate.
- **host functions:** `HostFunction { params -> ... }` com `FunctionType` descrevendo inputs/outputs. o host define funcoes kotlin que o WASM guest chama em runtime.
- **imports/exports:** sistema de importacao generico, funcoes, globals, memories, tables, tags. exports de um modulo podem ser imports de outro (mecanismo padrao WASM).
- **WASI preview 1:** suportado para syscalls (clock, filesystem, etc).
- **gradle plugin:** gera classes kotlin type-safe a partir de exports WASM. usa chasm para jvm/native, e engines embarcados (v8, spidermonkey, jsc) para targets JS.
- **proposals suportadas:** tail call, extended const, typed function refs, gc, multiple memories, exception handling, extended name sections, wide arithmetic.

**relevancia para φ:**
- chasm demonstra como construir um WASM runtime cross-platform de verdade. se φ quiser executar WASM modules nativamente (fora do browser), este e o modelo.
- o padrao de hostfunction e diretamente analogo ao que φ precisaria: funcoes do engine expostas para o WASM guest.
- o gradle plugin (gerar API tipada a partir de .wasm) e uma ideia aplicavel: um build step que gera rust bindings a partir dos exports de um modulo WASM.
- suporte a gc proposal e relevante para linguagens com garbage collection (como inform 7).

**insight principal:** o pattern `store + module + instance + invoke` e o padrao universal de embedding WASM. qualquer runtime φ precisaria dessas quatro abstracoes. chasm mostra que e viavel implementar isso em ~6k linhas de kotlin, um runtime WASM nao precisa ser massivo.

**limitacao:** nao e um runtime de alta performance (interpretador, nao JIT). para φ no browser, o WASM runtime do proprio browser (v8/spidermonkey) seria preferivel. chasm e mais relevante para o cenario nativo (desktop/mobile) onde nao ha runtime WASM built-in.

---

### wizard engine, 487 stars, sem releases formais

**repo:** `titzer/wizard-engine`
**licenca:** apache-2.0 (arquivo rt/license)
**linguagem principal:** virgil (linguagem propria do autor)
**criado:** 2019-12-11

**o que e:** engine webassembly de pesquisa ("research engine"), projetado para ensino e experimentacao. suporta wasm 3.0 completo incluindo proposals avanadas (gc, stack-switching, exception handling, SIMD, memory64). compila para ~1mb de binario nativo.

**arquitetura:**
- **module:** representacao decodificada em memoria de um modulo WASM. suporta relaxed section order com indice segregado por tipo de membro.
- **instance:** estado completo de um modulo instanciado, memorias, tabelas, imports vinculados. estrutura primaria para execucao.
- **binparser:** parser push-based (state machine), pode receber bytes incrementalmente (ex: streaming de rede). diferencial importante para cenarios web.
- **codevalidator:** validacao single-pass com abstract interpretation. produz "control transfer information" usado pelo interpretador rapido.
- **dois interpretadores:**
  1. v3 interpreter: simples, legivel, roda em todos os targets.
  2. fast interpreter: assembly x86-64 escrito a mao, ~40x mais rapido que o simples. interpreta bytecode in-place (sem rewrite).
- **single-pass compiler (spc):** compilador de passagem unica para codigo nativo.
- **self-hosting:** wizard compila para WASM e roda uma copia de si mesmo, proof que o engine e completo o suficiente para auto-hospedar.
- **monitors/instrumentacao:** sistema de monitors (built-in e custom) para analise de execucao. suporte a whamm! (DSL de instrumentacao WASM).
- **targets:** x86-darwin, x86-linux, x86-64-linux, jar (jvm), wasm.

**relevancia para φ:**
- modelo de referencia para instrumentacao e introspeccao de WASM modules. se φ quiser debugger ou profiler para linguagens guest, wizard mostra como.
- o parser push-based e ideal para streaming, carregar modulos WASM progressivamente enquanto faz download.
- o conceito de "control transfer information" (metadata pre-computada para branches) e relevante para performance de interpretadores.
- stack-switching (wasmfx) e relevante para coroutines/fibers em linguagens guest.

**insight principal:** wizard prioriza "simplicity and functionality first", a arquitetura soma requisitos presentes e futuros antes de otimizar. escrito em virgil (linguagem gc), proposals como wasm gc reutilizam o collector da linguagem host, mantendo o engine pequeno. essa filosofia de design e relevante para φ: nao otimizar prematuramente o runtime, mas garantir que a arquitetura comporte extensoes futuras.

**limitacao:** escrito em virgil (linguagem pouco conhecida, criada pelo mesmo autor). nao e possivel integrar diretamente no φ. valor e puramente de referencia arquitetural. sem releases formais no github.

---

### pywasm, 509 stars, v2.2.2

**repo:** `libraries/pywasm`
**licenca:** MIT
**linguagem principal:** python puro (sem dependencias externas)
**criado:** 2018-12-14

**o que e:** interpretador webassembly escrito em python puro, sem bibliotecas terceiras. suporta wasm 2.0 e WASI preview 1. requer python >= 3.12.

**arquitetura:**
- **modulos** (`pywasm/`): `core.py` (engine principal), `opcode.py` (definicoes de opcodes), `arith.py` (operacoes aritmeticas), `leb128.py` (codificacao de inteiros), `wasi.py` (syscalls WASI), `log.py` (logging).
- **API:** `Runtime()` -> `instance_from_file(path)` -> `invocate(instance, "func", args)`. minimalista e direta.
- **host functions:** suportadas via importacao, funcoes python podem ser chamadas pelo WASM guest (exemplo `fibonacci_env.py`).
- **WASI:** implementacao de WASI preview 1 para filesystem, clock, stdout. exemplos incluem http requests, directory listing, stdout capture.
- **performance:** interpretador puro, sem JIT. ~10x mais rapido no pypy que no cpython (design JIT-friendly). nao compete com runtimes nativos em velocidade.

**relevancia para φ:**
- demonstra a simplicidade minima de um runtime WASM: decoder + executor + host bindings em ~8 arquivos.
- o padrao `Runtime -> instance -> invocate` e o mesmo de chasm, reforando que e universal.
- util como referencia para entender internals de WASM (opcodes, leb128, memoria linear) sem a complexidade de engines de producao.
- WASI support mostra o contrato minimo para dar syscalls a um guest.

**insight principal:** um runtime WASM funcional cabe em poucos milhares de linhas de python puro. a complexidade nao esta no runtime em si, mas nas proposals avanadas (gc, SIMD, threads). para um mvp de runtime em φ, o subconjunto core e tratavel.

**limitacao:** performance inutilizavel para producao (interpretador python puro). nao suporta SIMD, threads, gc proposal. valor e exclusivamente educacional e de prototipagem.

---

## padroes cross-cutting

### rendering loop em emuladores WASM

- **padrao dominante:** requestanimationframe no main thread, emulacao em web worker. o worker "ticka" a CPU do guest e produz frame buffers. o main thread consome os buffers no ritmo do vsync.
- **frame skip:** quando o guest nao acompanha 60fps, pula frames de renderizacao mas continua tickando a emulacao.
- **canvas vs GPU:** wasmboy usa canvas 2d com `putImageData`. para φ, o guest produziria scene nodes (nao pixels), e o compositor φ renderizaria via webgpu.
- **visibility change:** emuladores pausam automaticamente quando a tab perde foco (event `visibilitychange`). φ ja faz isso no lifecycle nativo, precisa equivalente para guest runtimes.

### interface host-guest em runtimes

- **padrao universal:** `bind("name", hostFn)` no host, `call "name"` no guest. waforth, chasm e pywasm usam variacoes disso.
- **tipagem:** o contrato e via `FunctionType` (params + results em tipos WASM: i32, i64, f32, f64, externref). chasm gera wrappers type-safe. waforth usa stack manipulation manual.
- **async:** waforth resolve com `bindAsync` + execution tokens. WASM nao tem async nativo, o host precisa suspender a execucao e resumir com callback.
- **function table compartilhada:** waforth registra cada word compilada numa tabela de funcoes. `call_indirect` e o mecanismo para dispatch dinamico em WASM.
- **WASI como caso especial:** syscalls sao apenas host functions com nomes padronizados (`wasi_snapshot_preview1.*`). o guest importa, o host implementa.

### gerenciamento de memoria

- **memoria linear:** o guest tem acesso a uma memoria linear (arraybuffer). o host le/escreve nela para troca de dados (frame buffers, strings, structs).
- **sharedarraybuffer:** wasmboy usa quando disponivel para zero-copy entre worker e main thread. fallback para transferable buffers (muda ownership, nao copia).
- **gc:** chasm e wizard suportam a wasm gc proposal (structs e arrays gerenciados pelo engine). relevante para linguagens com gc (inform 7, potencialmente fortran moderno).
- **grow-only:** memorias WASM crescem mas nao encolhem (como `GpuVec` do φ, coincidencia arquitetural util).

### frame timing e sincronizacao

- **requestanimationframe:** unico mecanismo confiavel para vsync no browser. emuladores o usam no main thread e desacoplam a emulacao.
- **performance.now():** para medicao de tempo dentro de WASM, o host precisa expor uma funcao de clock (o WASM nao tem acesso a relogio nativo).
- **batch execution:** wasmboy executa n ciclos de CPU por batch, nao um por frame. o batch size e calibrado para manter ~60fps. analogia: o runtime φ executaria n instrucoes do guest por frame do compositor.

---

## implicacoes para φ

### arquitetura recomendada para runtime de linguagens

1. **modelo worker (browser):** a linguagem guest roda em web worker. produz comandos de cena (nao pixels) que sao transferidos para o main thread onde o compositor φ renderiza via webgpu. isso desacopla a velocidade do guest da taxa de refresh.

2. **interface via host bindings:** definir um conjunto de funcoes φ que o guest pode importar:
   - `φ_push_rect(layer, x, y, w, h, r, g, b, a)` -> scenenode::rect
   - `φ_push_text(layer, x, y, text_ptr, text_len, size)` -> scenenode::text
   - `φ_begin_frame()`, `φ_end_frame()`
   - `φ_get_input()` -> eventos de input
   - `φ_log(ptr, len)` -> console

3. **memoria compartilhada:** o guest escreve dados (strings, structs) na sua memoria linear. o host le via offsets. para dados grandes (textures), usar sharedarraybuffer quando disponivel.

4. **padrao store-module-instance:** seguir o padrao universal (chasm, pywasm, wizard):
   ```
   store = φWasmStore::new()
   module = store.load(wasm_bytes)
   instance = store.instantiate(module, imports)
   store.invoke(instance, "main", args)
   ```

5. **WASI para i/o de linguagens:** fortran e inform precisam de filesystem e stdout. implementar WASI preview 1 como host functions resolve isso sem inventar API propria.

6. **compilacao dinamica (futuro):** para linguagens que compilam dinamicamente (como forth), o padrao waforth (compile -> new module -> load -> register in table) e o unico caminho ate a proposta de JIT compilation em WASM ser aprovada.

### prioridades de implementacao

- **fase 1:** host bindings minimos (rect, text, input). um modulo WASM guest que chama essas funcoes para produzir uma cena φ. proof of concept.
- **fase 2:** WASI preview 1 para suportar fortran compilado com WASI-SDK. o compilador fortran (flang/lfortran) ja suporta target WASM.
- **fase 3:** inform 6/7 via glulx VM compilada para WASM (ja existem projetos como `glulx-wasm`). o guest rodaria a VM que interpreta o jogo, e produziria output via host bindings φ.
- **fase 4:** compilacao dinamica e instrumentacao (inspirado em waforth e wizard monitors).

### riscos identificados

- **WASM JIT proposal:** ainda nao aprovada. linguagens que precisam de compilacao dinamica dependem do workaround waforth (carregar modulos via JS API), que requer bridge JS mesmo no target WASM nativo do φ.
- **performance de interpretadores:** um runtime WASM interpretado (como pywasm) e ordens de magnitude mais lento que compilado. para targets nativos do φ, usar wasmer/wasmtime como backend (nao implementar interpretador proprio).
- **sharedarraybuffer:** requer headers coop/coep no servidor. sem ele, a transferencia de frame buffers entre worker e main thread copia dados.
- **wasm gc:** necessario para linguagens com gc. a proposal e recente e nem todos os browsers suportam completamente.
