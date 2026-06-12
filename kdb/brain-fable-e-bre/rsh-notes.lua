return {
  id  = "rsh-notes",
  typ = "reference",
  sts = "viva",
  dom = "research-raw",
  dat = "2026-06-11",
  ttl = "notas cruas: taffy, makepad, winit, touch, narrate, android, wgpu",
  lnk = { "shc-absorcao", "monster-formato", "org-docs" },
  txt = [[
notas de estudo do brenner, perto da fonte. sao insumos, nao decisoes.

taffy (por que 0.9, nao custom): 89us para 1000 nodes em release (bem
abaixo do <1ms); zero deps, rust puro; battle-tested (zed, bevy, servo,
slint, lapce); api simples; custom seria 300-500 loc basico e 3000+ com
edge cases. nossa superficie: Direction row|column; Align start|center|
end|stretch; Justify start|center|end|spacebetween|spacearound|
spaceevenly; LayoutStyle (direction, align, justify, padding[4], gap,
w/h/min/max, grow/shrink, width_percent/height_percent); ComputedBounds
absoluto; LayoutEngine possui a arvore, compute() devolve bounds.

makepad (insights; avaliar item a item, nao engolir):
1. turtle layout: layout como subproduto do desenho, sem fase separada;
   simples no caso comum, perde skip de subarvore
2. fenwick tree no portallist: o(log n) de scroll pra indice; 100k+ itens
   sem escanear alturas. forte candidato pra nossa virtualizacao
3. dsl com hot reload: editar ui sem recompilar (vm interpreta); o narrate
   pode chegar la
4. dockitem como arvore recursiva: splitter{a,b} binaria, tabs agrupa
   folhas, drag-drop reestrutura, serializa. relevante pro ide
5. drawstep incremental: draw() -> Result<(), WidgetRef>; err = mais um
   frame. rendering pausavel
6. componentmap: template separado de instancia, widgets sob demanda
7. stack integrada (do shader ao editor) permite otimizacao cross-layer
- a ide deles roda no browser via wasm (tree, tabs, editor com highlight,
  log, preview 3d, splitters): prova de teto
- theming por variaveis globais da dsl; file tree como vec flat pre-order
  + set de expandidos + depth pra indentacao (memoria sequencial)

winit ime/scale: WindowEvent::Ime {enabled, preedit(string, cursor),
commit, disabled}; set_ime_allowed(true) mostra teclado virtual;
set_ime_cursor_area posiciona sugestoes; altura de teclado NAO exposta
(heuristica 40 por cento da tela mobile; android content_rect muda quando
abre, da pra refinar por diff). GAP CONFIRMADO: keyboard_height sem
produtor em producao no workspace. scale_factor: f64, muda em runtime
(ScaleFactorChanged), nunca cachear.

touch (ja entregue em src/input): constantes touch_slop 10px, tap_max
300ms, long_press 500ms, double_tap_timeout 300ms, double_tap_slop 100px,
swipe_min_vel 200px/s, swipe_min_dist 50px. transicoes: down em idle ->
possibletap; move alem do slop -> dragging; tick alem de long_press ->
longpressing; lift dentro de tap_max -> waitingforsecondtap (emite tap);
segundo dedo -> pinching; segundo tap no timeout -> doubletap -> idle;
lift rapido com distancia -> drag(ended) + swipe.

narrate dsl (avaliar com ultraplan; candidato a frontend da dsl total):
gramatica hibrida estruturada-verbal: elementos sao keywords substantivos,
modifiers pares chave-valor, chaves para filhos, verbos (show, on, when,
each, bind). disambiguacao: sets disjuntos modifier/elemento; {expr} vs
{children} por estado do parser; flag vs valor por tabela estatica;
virgulas opcionais. flags: flex, center, centered, bold, italic, wrap;
resto exige valor; valores so literais ou {expr} (sem bare ident). codegen:
row -> div().flex().row(); show "Count: {count}" -> .child(format!(...))
com escape {{ e depth de chaves; output com paths qualificados sem leak.
estado real: crates/narrate_macro 2137 linhas, 99 testes verdes, usado por
src/narrate_runtime e hot_reload.

android emulador (era phi, precioso): cargo-ndk cross-compila o cdylib
aarch64; gradle empacota com MainActivity extends GameActivity (java);
android-activity 0.6.0 + games-activity aar 2.0.2 (TEM que casar; 3.0.5 da
NoSuchMethodError). comandos: cargo ndk -t arm64-v8a -o android/app/src/
main/jniLibs/ build --features android-game-activity; cd android &&
./gradlew installDebug; adb shell am start -n <app>/.MainActivity.
ARMADILHA: swiftshader (hw.gpu.enabled=no) trava PRA SEMPRE em
create_render_pipeline; fix: hw.gpu.enabled=yes + hw.gpu.mode=host no avd
(adapter vira a gpu host via gfxstream, init ~700ms). pollster em resumed
funciona com gpu host. cargo-apk nao serve (nativeactivity hardcoded, sem
workspaces). theme.appcompat obrigatorio no styles.xml. lib_name do
manifest tem que casar com o cdylib (o "φ" stale ja foi corrigido pra
plev).

wgpu gotchas: TextureView::clone() e safe no wgpu 28 (ref-counted); blur
compartilha uniform buffer entre passes h e v via write_buffer; texturas
do pool precisam RENDER_ATTACHMENT | TEXTURE_BINDING.

key findings da pesquisa de mercado (resume.txt): flutter 3.29 removeu
skia do ios (impeller unico, ~100KB/arch); slint render_by_line via spi
(ram minima = uma linha) e <300KiB runtime, primeiro port rp2040; lvgl
oficial: flash >64KB (>180KB rec), ram ~2KB estatica (nao os 32KB/128KB
de marketing); zag.js 5085 estrelas e o caso canonico de statecharts;
tauri <600KB; zed/gpui 84941 estrelas; dioxus 36267; lynx (bytedance
2025) claims proprios, exige poc antes de adotar; stars arredondados
sao volateis, revalidar via api antes de decisao critica.
]],
}
