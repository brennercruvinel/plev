---
project: plev
audience: [ai-agents, contributors]
status: reference
last-updated: 2026-03-11
domain: animation
---

# reference analysis: animation e motion

## escopo

analise factual de sete bibliotecas rust de animacao e movimento relevantes para task-27 (animation system) do plev: players de animacao vetorial com runtime completo, spring physics, tweening com keyframes, transicoes CSS-like, easing functions, e traits de interpolacao generica. foco em arquitetura interna, API design, compatibilidade WASM/multi-plataforma, e licoes aplicaveis ao design do sistema de animacao do plev.

dados coletados em marco de 2026 via github, crates.io, docs.rs e documentacao oficial.

---

## repositorios analisados

### dotlottie-rs (lottiefiles/dotlottie-rs), 235 stars, v0.1.54

**o que e:** player universal de animacoes lottie/.lottie construido em rust. renderiza via thorvg (c++, suporta software/opengl/webgpu). bindings FFI (cbindgen) para nativos e wasm-bindgen para web. alimenta os players oficiais para android, ios, web, flutter e react native. licenca MIT.

**arquitetura:**
- nucleo em rust, renderizacao delegada a thorvg (linkado estaticamente como dependencia c++). rust nao toca pixels diretamente, orquestra playback, state machines e theming.
- dois caminhos de binding: c API (cbindgen) para plataformas nativas, wasm-bindgen para browser.
- state machines declarativas para interatividade: transicoes entre estados de animacao controladas por inputs programaticos.
- formato .lottie e um ZIP contendo JSON (animacao) + assets (imagens). o runtime descompacta, parseia e alimenta thorvg.
- tweening entre frames adicionado em v0.1.39 (mar 2025): cubic bezier easing com control points p1(x1,y1) e p2(x2,y2) para interpolacao suave entre frames arbitrarios.
- build system com makefile cross-platform (android, apple, WASM, linux). linguagens: rust 89.9%, makefile 7%.

**relevancia para plev:**
- modelo de "orquestrador rust + renderer nativo" e analogamente o que plev faz (rust + wgpu). a separacao clara entre logica de animacao e rendering e um pattern validado.
- state machines para transicoes interativas e relevante para task-27: plev poderia adotar pattern similar para gerenciar estados de UI (idle -> hover -> pressed -> active).
- o tweening cubic bezier entre frames e exatamente o que task-27 precisa implementar internamente para `CubicBezier(f32,f32,f32,f32)` no enum easing.

**insight principal:** dotlottie-rs demonstra que um runtime de animacao robusto pode ser construido com rust orquestrando e uma engine de rendering executando. o valor do rust nao esta nos pixels, esta no controle de estado, timing e coordenacao cross-platform. plev ja tem essa arquitetura, task-27 adiciona a camada temporal.

**limitacao:** thorvg como dependencia c++ linkada estaticamente e inviavel para plev (que e pure rust + wgpu). o formato lottie (JSON-based, orientado a motion design) nao se aplica a UI transitions. relevancia e puramente arquitetural/conceitual.

---

### rive-rs (rive-app/rive-rs), 117 stars, sem releases publicadas

**o que e:** runtime rust para animacoes interativas criadas no editor rive. renderiza via vello (2d vector renderer em rust/wgpu) com trabalho em andamento para suportar o rive renderer proprietario como backend alternativo. licenca MIT.

**arquitetura:**
- linguagens: rust 71.9%, c++ 28.1% (runtime c++ do rive compilado junto).
- rendering via vello, vector graphics 2d com WGSL compute shaders. backend alternativo (rive renderer) em desenvolvimento para resolver limitacoes do vello (inconsistencias em mesh triangles, strokes sempre round, clips incorretos com alta contagem).
- state machines: conceito central do rive. animacoes sao grafos de estados com transicoes condicionais controladas por inputs (boolean, number, trigger). o runtime avanca a state machine a cada tick.
- requer c compiler e git submodules inicializados antes de compilar, dependencia pesada no build.
- sem publicacao no crates.io, uso via git dependency.

**relevancia para plev:**
- o conceito de state machine de animacao do rive e o mais sofisticado entre as libs analisadas. para task-27, o pattern "estado atual + transicao condicional + blending entre estados" e valioso como referencia arquitetural.
- vello como renderer 2d via wgpu e tecnicamente adjacente ao plev. analisar como rive-rs integra com vello pode informar futuras decisoes sobre vector rendering (task-31).
- a separacao entre "definicao de animacao" (editor) e "runtime de animacao" (rive-rs) e um pattern maduro. plev pode eventualmente consumir arquivos de animacao, mas task-27 foca em animacao programatica.

**insight principal:** state machines de animacao com blending condicional sao o padrao da industria para animacoes interativas complexas (jogos, UI rica). task-27 comeca com tweens simples, mas a arquitetura deve permitir evolucao para state machines se necessario (nao fechar portas).

**limitacao:** nao publicado no crates.io. dependencia pesada em c++ compilado. vello ainda tem inconsistencias de rendering. comercialmente atrelado ao editor rive, nao e uma lib generica de animacao. para plev, valor e puramente conceitual.

---

### natura (bugthesystem/natura), 71 stars, v0.1.0

**o que e:** biblioteca de spring animation baseada em damped harmonic oscillator. port direto do algoritmo de ryan juckett (c++, 2008/2012). framework-agnostico, funciona em 2d e 3d. licenca unlicense (dominio publico).

**arquitetura:**
- nucleo: damped simple harmonic oscillator com tres modos controlados pelo damping ratio:
  - under-damped (ratio < 1.0): rapido com oscilacao (bouncy)
  - critically damped (ratio = 1.0): mais rapido sem oscilacao
  - over-damped (ratio > 1.0): mais lento, sem oscilacao
- dois parametros de configuracao: angular frequency (velocidade) e damping ratio (comportamento).
- frame-rate independent: recebe delta time como parametro, calcula estado do spring por frame.
- zero dependencias em runtime (apenas `approx` em dev-dependencies para testes).
- plugin bevy disponivel (`bevy_natura`) que usa `Time` resource do bevy para dt automatico.
- suporte a animation groups (controle batch), pause/resume individual, event listeners para completion.
- rust 100%, edition 2021.

**relevancia para plev (alta, task-27 fase d):**
- task-27 fase d especifica `create_spring(initial, stiffness, damping)` como stretch goal. natura implementa exatamente isso.
- o modelo matematico (damped harmonic oscillator) e o padrao para spring animations em UI (ios uispringanimation, android springforce, react-spring usam o mesmo modelo).
- API minima: configura angular_frequency + damping_ratio, alimenta dt por frame, recebe posicao atualizada. sem overhead, sem alocacoes.
- framework-agnostico: encaixa no loop de rendering do plev sem friccao. basta passar o `dt` do `AnimationTick` proposto no task-27.
- licenca unlicense permite literalmente copiar o algoritmo para dentro do plev se preferir nao adicionar dependencia.

**insight principal:** o algoritmo de spring (damped harmonic oscillator) cabe em ~100 linhas de rust puro. a equacao diferencial tem solucao analitica para cada modo de damping, nao e integracao numerica (euler/verlet), e formula fechada por frame. isso significa: zero acumulo de erro, determinismo independente de framerate, custo computacional negligivel por spring.

**limitacao:** v0.1.0, crate nao parece publicado no crates.io (docs.rs retorna 404). uso via git dependency ou vendoring. sem suporte no_std documentado. ultimo commit jan 2025.

---

### keyframe (hannesmann/keyframe), 138 stars, v1.1.1, ~475k downloads

**o que e:** biblioteca de tweening e keyframe animation. easing functions penner + bezier customizavel + macro `keyframes![]` para sequencias declarativas. suporte no_std. licenca MIT.

**arquitetura:**
- funcao core: `ease(function, from, to, time)` onde `time` e f32 em [0,1], `from`/`to` implementam `CanTween`.
- `CanTween` trait: requer `fn tween(&self, other: &Self, scalar: f64) -> Self`. implementado para f64 e tipos mint (vector2, vector3, vector4, point2, point3). `#[derive(CanTween)]` disponivel para structs customizados.
- `EasingFunction` trait: `fn y(&self, x: f64) -> f64`, curva 2d normalizavel. qualquer struct que implemente pode ser usado como easing.
- `AnimationSequence<T>`: timeline com multiplos keyframes, tracking de tempo via `advance_by(dt)`, lerp automatico entre keyframes adjacentes.
- `keyframes![...]` macro: cria animationsequence a partir de lista declarativa de (valor, timing, easing).
- `no_std` suportado: desabilitar feature `alloc` remove keyframe e animationsequence (que usam vec), mantendo `ease()` e easing functions. ideal para embedded.
- feature `mint_types`: interop com ecossistema mint (glam tem feature mint para conversao).
- 475k+ downloads, 7 versoes publicadas. estavel desde v1.0.

**relevancia para plev (alta, task-27 fase b e c):**
- o design de `ease(function, from, to, t)` e exatamente o que task-27 especifica em `ease(t, easing) -> f32`.
- o `CanTween` trait e analogo ao `Interpolate` trait que task-27 propoe para f32, [f32;4], (f32,f32). mesmo conceito, mesma necessidade.
- `AnimationSequence` com `advance_by(dt)` demonstra o pattern de timeline que plev pode adotar: sequencia de valores interpolados avancada por delta time a cada frame.
- bezier customizavel (CSS `cubic-bezier` equivalent) ja implementado, task-27 pode referenciar ou ate usar diretamente.
- no_std + mint interop = dependencia segura e leve para plev.

**insight principal:** keyframe valida a abordagem de task-27: o trait `CanTween` (= `Interpolate` do plev) + `ease()` puro + timeline com `advance_by(dt)` e suficiente para animacoes de UI. a complexidade adicional (state machines, springs, physics) e ortogonal e pode ser composta por cima.

**limitacao:** usa f64 internamente (cantween retorna f64). plev opera em f32 (vertex buffers, uniforms, shaders). conversao f64->f32 nao e problema de performance mas e atrito ergonomico. nao tem spring physics. nao tem state management, e puramente funcional (calcula valor, nao gerencia estado).

---

### mina (focustense/mina), 21 stars, v0.1.1, ~11k downloads

**o que e:** biblioteca de animacao framework-independent inspirada em CSS transitions e @keyframes. macro `timeline!` e `animator!` para sintaxe declarativa. licenca MIT.

**arquitetura:**
- duas apis principais:
  1. **timeline**: sequencia de keyframes com porcentagens (estilo CSS @keyframes). define valores em 0%, 50%, 100% com easing entre cada par. avancado via elapsed time.
  2. **stateanimator**: transicoes automaticas entre estados (estilo CSS transitions). ao mudar de estado, blenda automaticamente do valor atual para o valor do novo estado. analogo a `.transition { property: all 0.3s ease }`.
- modulos: `mina_core` (logica), `mina_macros` (proc-macros para sintaxe), `mina` (re-export).
- suporte a bevy, iced e nannou via examples (nao tight coupling).
- framework-agnostico: nao tem event loop proprio. o usuario avanca o tempo e le os valores.
- merge de timelines heterogeneas: animar propriedades de tipos diferentes (f32 + color + position) em uma unica timeline.
- easings inclusos, custom functions suportadas.
- dependencia em `enum-map` para state animators (roadmap menciona remocao futura).
- 11k downloads, 2 versoes publicadas.

**relevancia para plev (media-alta, task-27 design reference):**
- o stateanimator e a resposta mais direta para "como animar transicoes de estado em UI": define estado idle, hover, pressed como enum, associa valores para cada, e o animator interpola automaticamente na transicao. isso e exatamente o que messagedock example do plev faz manualmente.
- o pattern "framework-agnostico, usuario avanca tempo" e identico ao que task-27 propoe: `AnimationTick` fornece dt, tween avanca internamente.
- sintaxe CSS-like (`timeline!` com porcentagens, easing keywords) e boa referencia de ergonomia, mas plev provavelmente nao precisa de proc-macro dedicada para animacao (signals + tween sao suficientes).

**insight principal:** o stateanimator de mina resolve um problema real que task-27 nao aborda explicitamente: gerenciamento de transicoes bidirecional entre estados. quando hover entra, anima para valores de hover; quando sai, anima de volta. o tween<t> proposto em task-27 faz isso via `.set_target()`, mas sem conceito formal de "estado". considerar se plev eventualmente precisa de um pattern mais estruturado.

**limitacao:** comunidade muito pequena (21 stars, 11k downloads). dependencia em enum-map para states. documentacao limitada (readme bom, docs.rs basico). v0.1.1, API nao estavel.

---

### easer (orhanbalci/rust-easing), 18 stars, v0.3.0, ~909k downloads

**o que e:** implementacao minimal das easing functions de robert penner. crate chamado `easer` no crates.io (repo chamado `rust-easing`). licenca MIT.

**arquitetura:**
- 30 easing functions: 10 familias (quad, cubic, quartic, quintic, sine, circular, exponential, elastic, back, bounce) x 3 direcoes (in, out, inout).
- assinatura classica penner: `ease(t, b, c, d)`, t=current time, b=start value, c=change, d=duration. valores nao normalizados (diferente do padrao moderno t in [0,1]).
- dependencia: `num-traits` (para generics numericos).
- zero alocacoes, puramente funcional.
- 909k+ downloads (mais popular crate de easing por volume), 5 versoes.
- ultimo release: v0.3.0. repo com 99 commits, ultimo release tag jun 2018.

**nota:** existe tambem o crate `easing` (joliv/easing, v0.0.5) que usa iterators para easing e tem abordagem diferente (16 funcoes, sem documentacao). e o crate `easings` que normaliza para t in [0,1]. o ecossistema de easing em rust e fragmentado.

**relevancia para plev (media, task-27 fase b referencia):**
- as 30 funcoes de penner sao o catalogo padrao da industria. task-27 fase b especifica apenas 4 (linear, easein, easeout, easeinout cubic) + cubicbezier. o catalogo completo de penner pode ser adicionado futuramente.
- a assinatura penner classica (t,b,c,d) e menos ergonomica que a abordagem moderna (t normalizado em [0,1], resultado em [0,1]). task-27 deve usar a abordagem moderna.
- com ~909k downloads, easer valida que funcoes penner sao utility code amplamente reutilizado. mas a implementacao e trivial (~5 linhas por funcao) e nao justifica dependencia externa para plev.

**insight principal:** easing functions sao commodity. a implementacao e matematica pura (polinomios, trigonometria) e cabe em <200 linhas para as 30 funcoes. para plev, implementar internamente (inline, f32, t normalizado) e preferivel a depender de crate externo, evita conversao f64/f32, controla API, zero overhead.

**limitacao:** assinatura penner nao-normalizada e arcaica. repo sem atividade desde 2018. dependencia em num-traits e desnecessaria para f32 puro.

---

### interpolation (pistondevelopers/interpolation), ~900k downloads, v0.3.0

**o que e:** biblioteca de interpolacao generica do ecossistema piston. traits `Lerp` e `Spatial` + funcoes lerp, quad_bez, cub_bez + enum `EaseFunction` com 30 variantes penner. licenca MIT.

**arquitetura:**
- `Lerp` trait: `fn lerp(&self, other: &Self, scalar: &Self) -> Self`. implementado para f32, f64.
- `Spatial` trait (nao documentado explicitamente): usado por funcoes de interpolacao como constraint generico.
- `EaseFunction` enum: 30 variantes identicas ao catalogo penner (quadraticin/out/inout, cubicin/out/inout, ..., bouncein/out/inout).
- funcoes: `lerp(a, b, t)`, `quad_bez(a, b, c, t)` (bezier quadratico), `cub_bez(a, b, c, d, t)` (bezier cubico).
- zero dependencias. 100% documentado.
- mantido por pistondevelopers (sven nilsen). 10 versoes, 900k+ downloads.
- estavel: v0.3.0, sem breaking changes recentes.

**relevancia para plev (media, task-27 design reference):**
- o `Lerp` trait e a abordagem mais simples possivel para interpolacao generica. o `Interpolate` trait que task-27 propoe pode seguir este design exato.
- `EaseFunction` enum com 30 variantes e referencia direta para o `Easing` enum do task-27. mesma abordagem (enum + match + formula matematica).
- `cub_bez(a, b, c, d, t)` e a implementacao de bezier cubico que task-27 precisa para `CubicBezier(f32,f32,f32,f32)`.
- crate maduro (900k downloads) e bem testado. pode ser usado como dependencia se desejado, mas as funcoes sao simples o suficiente para implementar internamente.

**insight principal:** o pattern `trait Lerp` + `enum EaseFunction` + `fn ease(function, t) -> f32` e o denominador comum de toda biblioteca de animacao em rust. keyframe usa, mina usa, interpolation define. task-27 deve implementar exatamente este pattern, e o consenso do ecossistema.

**limitacao:** ecossistema piston esta essencialmente inativo (ultimo release do piston em 2023). o crate interpolation funciona standalone mas nao recebe atualizacoes. API minimalista, sem timelines, sequencias ou state management.

---

## padroes cross-cutting

### 1. separacao entre easing, interpolacao e timeline

todas as bibliotecas separam tres camadas:
- **easing function**: `f(t) -> t'` onde t in [0,1]. pura matematica, sem estado.
- **interpolacao**: `lerp(a, b, t') -> valor`. generica sobre tipo t.
- **timeline/tween**: gerencia tempo, avanca por dt, compoe easing + interpolacao.

essa separacao e universal: keyframe, mina, interpolation, dotlottie-rs, rive-rs, todos seguem. task-27 deve respeitar essa separacao.

### 2. delta time como input, nao global

todas as bibliotecas framework-agnosticas (keyframe, mina, natura, interpolation) recebem dt como parametro explicito. nenhuma assume timer global. task-27 pode usar signal global para dt, mas internamente o tween deve receber dt como parametro do tick.

### 3. trait de interpolacao e o ponto de extensao

`CanTween` (keyframe), `Lerp` (interpolation), `Lerp` (mina), todos definem um trait para tipos interpolaveis. esse e o ponto onde o usuario estende o sistema para tipos customizados (cores, posicoes, tamanhos). task-27 deve expor `Interpolate` como trait publico.

### 4. springs sao ortogonais a tweens

natura (springs) e keyframe (tweens) resolvem problemas diferentes:
- **tween**: vai de a para b em tempo fixo com curva definida. determinismo temporal.
- **spring**: converge para target com dinamica fisica. tempo de chegada e indeterminado.

ambos produzem um valor por frame. a interface de consumo e identica (`.get() -> T`). a implementacao interna e completamente diferente. task-27 deve permitir que springs e tweens coexistam com mesma interface.

### 5. f32 vs f64

keyframe e easer usam f64 internamente. interpolation suporta ambos via generics. para plev (GPU pipeline f32), a conversao e atrito desnecessario. implementacao interna em f32 e preferivel.

### 6. cubic bezier e o padrao para easing customizado

CSS `cubic-bezier(x1,y1,x2,y2)` e suportado por keyframe (beziercurve), dotlottie-rs (tweening v0.1.39), e e o padrao web para easing customizado. task-27 ja especifica `CubicBezier(f32,f32,f32,f32)`, correto.

---

## implicacoes para plev

### decisoes de design para task-27

**1. nao adotar dependencia externa para easing/interpolacao.**
as funcoes sao triviais (<200 LOC para 30 easings + lerp + cubic bezier). implementar internamente em f32, inline, zero-alloc. evita conversao f64, controla API, sem dependencias transitivas. o ecossistema confirma que isso e commodity code.

**2. implementar `Interpolate` trait seguindo o consenso.**
```rust
pub trait Interpolate {
    fn interpolate(&self, target: &Self, t: f32) -> Self;
}
```
implementar para f32, [f32;4] (cor rgba), (f32,f32) (posicao). derivacao automatica pode vir depois.

**3. tween<t> com advance(dt) e a abordagem correta.**
keyframe (animationsequence::advance_by), mina (stateanimator com elapsed), natura (spring com dt), todos usam o mesmo pattern. tween recebe dt, calcula t normalizado, aplica easing, interpola. task-27 ja propoe isso.

**4. springs como modulo separado, mesma interface.**
se fase d for implementada, seguir o modelo de natura: damped harmonic oscillator com angular_frequency + damping_ratio. a solucao analitica cabe em ~100 LOC (port direto do algoritmo de ryan juckett). mesma interface `.get() -> T` do tween. pode inclusive copiar o codigo do natura (unlicense).

**5. nao fechar portas para state machines.**
mina's stateanimator e rive-rs' state machines mostram que animacoes de UI complexas eventualmente precisam de conceito de "estado" alem de "target value". task-27 comeca com tween + set_target(), mas a arquitetura deve permitir compor algo como:
```rust
animator.transition_to(State::Hover); // blenda de current para Hover
animator.transition_to(State::Idle);  // blenda de Hover para Idle
```
isso nao precisa estar no scope de task-27, mas nao deve ser impedido pelo design.

### monitorar

**keyframe**, se plev precisar de sequencias de keyframes complexas (multi-step animations), keyframe e a referencia mais madura. a 475k downloads e estavel desde v1.0.

**natura**, se task-27 fase d implementar springs, o algoritmo de natura (ryan juckett) e a referencia exata. unlicense permite vendoring direto.

### nao adotar

**dotlottie-rs**, dependencia c++ (thorvg), formato lottie nao se aplica. valor puramente conceitual.

**rive-rs**, nao publicado no crates.io, c++ linkado, atrelado ao editor rive. conceito de state machines e valioso mas nao a implementacao.

**mina**, comunidade muito pequena (21 stars), API instavel (v0.1.1), dependencia em enum-map. conceito de stateanimator e valioso como referencia, nao como dependencia.

**easer / easing / interpolation**, easing functions e interpolacao sao triviais de implementar. adicionar dependencia para <200 LOC de matematica pura nao se justifica para um engine que visa zero dependencias desnecessarias.
