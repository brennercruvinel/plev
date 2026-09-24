---
project: plev
audience: [ai-agents, contributors]
status: ongoing
last-updated: 2026-09-23
domain: generative-runtime
task: task-76
wave: trilha
depends: []
depends-optional: [task-58]
branch: task/76-platform
---

# task-76: integração de plataforma

## objetivo

o plev responde de forma consistente às bordas que separam renderer de
application runtime: menus, tray, notificações, diálogos de arquivo, drag
and drop, clipboard, IME, múltiplas janelas, diálogos nativos, impressão,
câmera, microfone, deep links, sleep e wake, perda de device de GPU e
accessibility. e cada integração que é uma ação vira capacidade no
contrato: o item de menu nativo e o botão da toolbar são duas
representações da mesma capacidade.

## ligação com a tese

seção 22 (integração de plataforma continua existindo; é parte da
diferença entre renderer e application runtime), seção 6 (uma capacidade,
várias representações).

## estado atual no código

- clipboard: `crates/engine/src/clipboard.rs` (arboard no desktop,
  `LocalClipboard` em teste, wasm e mobile).
- diálogo de arquivo nativo (rfd) e drop de arquivo: só no urnaui
  (`crates/urnaui/src/app.rs`, `crates/urnaui/src/view/explorer.rs`).
- IME básico: `crates/engine/src/platform/ime.rs`.
- lifecycle (estados de app, aviso de memória):
  `crates/engine/src/platform/lifecycle.rs`; safe areas em
  `crates/engine/src/platform/mod.rs`.
- sem menus nativos, tray, notificações do sistema, múltiplas janelas,
  diálogos nativos no engine, impressão, câmera, microfone, deep links,
  sleep e wake, ou recuperação de device de GPU perdido.

## escopo

ordem recomendada (robustez primeiro, depois o que os apps atuais pedem):

1. perda de device de GPU: callback de device lost do wgpu, recriação de
   surface, pipelines, atlas de glifo, texturas; teste forçando a perda.
2. múltiplas janelas no engine (hoje cada app tem uma).
3. diálogos de arquivo e drag and drop: mover do urnaui para
   `engine::platform`, sem segunda implementação; o urnaui passa a usar o do
   engine.
4. menus nativos: cada item é um `SemId` com ação no contrato; o menu é
   outra representação da capacidade, e o clique no menu vai pelo
   dispatcher como `Actor::Human`.
5. notificações do sistema e tray.
6. deep links (esquema de URL do app) que viram `Request` pelo dispatcher,
   com policy.
7. sleep e wake: pausar animação e trabalho de fundo, retomar sem frame
   velho.
8. diálogos nativos (alerta, confirmação), incluindo a confirmação humana
   da task-58 como opção.
9. impressão (render headless da task-66 para o sistema de impressão).
10. câmera e microfone (captura para textura e buffer de áudio), com
    permissão do sistema.

cada item: API no engine, a capacidade no contrato quando é ação, teste, e
dependência nova justificada (por exemplo menus e tray em crate
especializado) e aprovada antes.

## fora de escopo

bring-up de linux e windows como alvo; cada item entra nessas plataformas
quando o alvo existir.

## dependências

independente; os itens que viram capacidade usam 58.

## superfície sob responsabilidade

`crates/engine/src/platform/**`, `crates/engine/src/window` na parte de
múltiplas janelas e device lost (coordenado com a 60 se ela estiver
rodando), o urnaui na troca para o diálogo do engine, ADR por item.

## interfaces que consome

`wgpu`, `winit`, `sem` (58) para capacidades.

## interfaces que pode alterar

`engine::platform` por adição; `engine::window` com aprovação.

## critérios de aceitação

por item: funcionando no macOS (alvo principal), variante mobile ou web
registrada (feita ou declarada fora), teste automático onde é possível,
roteiro manual onde não é. device lost sobrevive a uma perda forçada sem
reiniciar o app.

## testes esperados

device lost forçado; múltiplas janelas com estado separado; menu como
capacidade (request pelo dispatcher); deep link com policy.

## riscos e perguntas abertas

- cada item pode puxar dependência grande; o custo de binário é medido.
- permissões de câmera e microfone mudam por sistema.

## commits esperados

um grupo por item, `task-76:` no início.

## pare e reporte se

- um item exigir mudar o shell de todos os apps;
- dependência nova (sempre aprovação).

## handoff da fase 1: perda de device de GPU (item 1)

branch `task/76a-device-loss`, base `e478054a` (main). commits, em ordem:

- `93dc4eab` task-76: the device loss probe
- `b9ae6aa1` task-76: the device lost signal
- `6047a4de` task-76: the recreation after a lost device
- `b89caed6` task-76: the frame recovers a lost device, and the tests that
  force one
- (este) task-76: ADR and the phase 1 handoff

### a sonda

teste descartável sobre `engine::headless::Renderer` (ficou como `#[ignore]`
no commit 1, substituído pelos testes reais no commit 4): frame com texto,
rounded rect e imagem, `Device::destroy()`, poll, desenhar de novo, ler.
observado no metal desta máquina (wgpu 28.0.0, wgpu-core 28.0.1):

- `destroy` sozinho não avisa ninguém: o callback de perda chega no primeiro
  `poll` (`Poll` ou `wait`), com `(DeviceLostReason::Destroyed, "")`. no
  wgpu-core, `destroy` só marca `valid = false`; `maintain` (o poll, e o
  submit) entrega o closure quando a fila está vazia. perda por driver é
  outra: `Device::lose` dispara o callback na hora, com `Unknown` e a
  mensagem do driver, na thread que bateu no erro hal.
- desenhar no device morto: "1 draw, sem pânico". o wgpu engole todo erro
  do tipo `DeviceLost` (`ErrorType::DeviceLost => return` antes do handler
  fatal), então cada `create_*` devolve recurso inválido, o submit some, e
  o compositor marca as layers limpas em cima de nada. buraco negro
  silencioso: a janela fica com a última imagem.
- o primeiro uso de recurso morto que o wgpu ainda valida é fatal: o
  readback headless deu pânico em `Buffer::map_async` ("headless_readback
  is invalid") pelo handler padrão.
- um renderer novo não conseguiu redesenhar a imagem: o `prepare` drenava
  o `pending` do store global, a única cópia dos pixels.
- `wgpu::Instance::new` com `Backends::empty()` não dá pânico e
  `request_adapter` responde "No suitable graphics adapter found": é como o
  teste simula a GPU ausente.
- o callback existe e é entregue no metal: não precisou do plano B
  (`SurfaceError`, `PollError`).

(b) o que é recurso do device (morre) e o que é CPU (sobrevive):

- `GpuContext`: morrem `device`, `queue`, `surface_config` (só a
  configuração; a `Surface` em si pertence ao `Instance` e sobrevive),
  `projection_buffer` e `projection_bind_group`, todos os
  `*_bind_group_layout`, todos os pipelines (quad, rect_sdf,
  shadow_analytic, text, image, backdrop, composite), `composite_sampler`,
  `image_atlas` (textura, bind group, sampler), `timestamps` (query sets e
  buffers). sobrevivem `config`, `logical_size`, os watchers de hot reload.
- `TextSystem`: morrem `atlas_texture`, `atlas_view`, `atlas_sampler`,
  `atlas_bind_group`; ficam inválidos `glyph_cache` e `allocator` (apontam
  pra textura morta; não há bitmap de glifo em cache de CPU, o glifo volta
  do swash). sobrevivem `font_system`, `shaping_cache`, `raster_scale`,
  `atlas_size`, `embedded_fonts`.
- `ImageAtlasGpu`: morrem textura, bind group, sampler. `ImageStore` (CPU,
  global): sobrevivem os handles e, agora, os pixels de cada upload.
- `TexturePool`: morre tudo (só textura e view).
- `EffectProcessor`: morre tudo (três pipelines, quatro layouts, sampler,
  buffer de uniform).
- compositor, por `Layer`: morrem `msaa_texture/view`, `texture/view`,
  `composite_bind_group`, `opacity_buffer/bind_group`, e os doze `GpuVec`
  (vb e ib de quad, sdf, shadow, image, backdrop, text). sobrevivem
  `nodes`, `sequence`, `text_groups`, os vetores de vértice e índice na
  CPU, `prev_hash`, `clip_rect`, `effects`, `opacity`.
- headless `Renderer`: morre `target`.

(c) a surface de janela sobrevive à recriação no wgpu 28: o
`wgpu_core::Surface` guarda a `Presentation { device: Arc<Device>, config,
acquired_texture }` e `configure_surface` substitui a presentation com o
device novo (recusa só se há uma textura adquirida, `PreviousOutputExists`).
precisa ser reconfigurada com o device novo, e o adapter novo tem que ser
pedido com `compatible_surface`. é o que `recover` faz.

### o sinal

`crates/engine/src/gpu/loss.rs`. `GpuContext::from_device` (o corpo em que
os dois construtores terminam) instala `Device::set_device_lost_callback`
em todo device. o callback sobe um `AtomicBool` num `Arc` e guarda
`LostDevice { reason, message }` num `Mutex` ao lado; o frame lê o flag sem
bloquear (`GpuContext::device_lost`, `lost_device`).
`poll_device_loss` faz um `device.poll(Poll)` antes de ler, porque um
`Destroyed` só chega com poll. `Destroyed` pedido pelo engine não conta: o
`Drop` do `GpuContext` marca `expected` antes do device cair. um
`reported` por perda faz o erro de recuperação falhada sair uma vez.
escolhi atomic e não canal: o frame precisa de "está perdido agora?", não
de uma fila de eventos, e o callback pode vir de qualquer thread.

### a ordem de recriação e o que sobrevive

`crates/engine/src/gpu/recover.rs`. `GpuContext::recover` (bloqueante,
nativo) e `recover_async` (os dois alvos):

1. adapter no `Instance` do próprio contexto (novo campo `instance`;
   `compatible_surface` quando há janela), depois device e queue com o
   mesmo `device_descriptor`. erro em qualquer um: `RecoverError::NoAdapter`
   ou `NoDevice`, nada muda, o contexto fica perdido, a próxima tentativa
   pode dar certo.
2. janela: `surface_configuration` (extraído de `new_with_config`) refaz
   formato, view format sRGB, present mode e alpha mode com o adapter novo,
   mesmo tamanho, e `surface.configure(&device_novo)`. headless: a config
   fica.
3. `from_device` reconstrói tudo (projeção, layouts, pipelines, sampler,
   query sets, atlas de imagens vazio) e instala o callback no device novo.
4. carregam: `config`, `logical_size` (reaplicado com `set_projection`),
   `recovery_backends`, os watchers de hot reload. `*self = fresh`.

`recover_frame_resources` (nativo) e `_async` cuidam do que o frame tem
fora do contexto, nessa ordem: `TextSystem::recreate_atlas` (mesmo tamanho,
vazia, cache e allocator zerados, `atlas_disturbed` pra toda layer
re-resolver; os glifos voltam no próximo frame pelo mesmo `rasterize_and_upload`,
então o ADR do raster não muda), `EffectProcessor::new` (o dono troca,
porque `effects/**` é proibido), `TexturePool::clear` (novo),
`Compositor::drop_gpu_resources` (novo, `layer/texture.rs` e
`layer_ops.rs`). `mark_all_dirty` não bastou: `ensure_texture` só recria
por mudança de tamanho e `GpuVec` só por capacidade, os dois escreveriam
no device morto. o compositor sai dirty inteiro.

imagens: o `ImageStore` virou um log append-only de uploads com os pixels
(mesmo orçamento do atlas, que nunca despeja) e cada `ImageAtlasGpu` guarda
um cursor de quanto já escreveu; um atlas que nasce vazio (device
recuperado, ou um segundo contexto) escreve o log inteiro no próximo
`prepare`. o app não reenvia nada. a exceção é recurso que o app criou no
`gpu.device` por conta própria: morreu, e `device_lost()` antes do frame é
como ele sabe.

### o gancho no frame

antes de `get_current_texture`, onde já estava o poll do shader:
`present_frame` (`window/frame.rs`), `App::render` (`window/render.rs`) e
`Renderer::draw` (`headless/mod.rs`) fazem `poll_device_loss()` e, se
perdido, `gpu::recover_or_skip`: recupera e desenha o frame no device novo,
ou pula (`None`; `0` draws no headless) com um `log::error!` por perda, e o
próximo frame pedido tenta de novo. `SurfaceError::Lost | Outdated` num
device perdido faz `compositor.invalidate()` (pede um frame, que recupera
antes de adquirir) em vez de reconfigurar a swapchain morta. render on
demand fica: nada acontece fora de um frame pedido. `Renderer::read`
responde `HeadlessError::DeviceLost` enquanto o device está morto (era o
pânico). no wasm o frame é pulado e a perda logada uma vez:
`recover_frame_resources_async` existe pra um shell que a dirija, mas
`present_frame` é síncrono e o request de device no browser é promise.

### os testes e os pixels

`crates/engine/tests/device_loss.rs`, headless 240x140 a escala 2
(480x280), pulam sem adapter:

- `the_frame_after_a_forced_loss_matches_the_frame_before`: frame A com
  texto, rounded rect com borda e imagem 8x8 verde numa layer retida, rect
  vermelho na layer padrão (36 mil pixels fora do fundo); `destroy`; poll
  entrega `Destroyed`; frame B igual a A byte a byte (a imagem em 400,60 é
  20,200,40), `device_lost()` falso, `lost_device()` None; frame C igual.
- `a_loss_in_the_middle_of_a_sequence_leaves_no_stale_frame`: quatro
  frames, o rect da layer padrão muda de cor por frame, a outra layer é
  retida; `destroy` antes do frame 2 sem poll no meio; cada frame igual a
  um renderer de controle que nunca perdeu o device, e mostra o rect do
  próprio frame e a imagem retida.
- `a_recovery_without_an_adapter_leaves_the_renderer_alive_and_says_so_once`:
  `set_recovery_backends(Backends::empty())`, `destroy`; dois `draw`
  devolvem 0 sem pânico, `read` dá `DeviceLost(Destroyed)`, um único
  `error` no log ("not recovered"); `Backends::PRIMARY` de volta e o frame
  seguinte é igual ao A, ainda um único error.

unitários: `gpu/tests.rs` (vivo, destruído com razão, destroy esperado não
conta, recuperação em lugar com projeção mantida e buffer mapeável no
device novo, sem adapter é erro e fica perdido até ter adapter),
`image.rs` (o log guarda tudo pra um atlas novo), `texture_pool.rs`
(`clear`), `compositor/tests.rs` (`drop_gpu_resources` esquece tudo e suja
tudo). os testes de hoje (headless, text_raster_pixels, text_drawn_extent,
text_audit, hot reload, stages) sem mudança de resultado.

### gate e regress

`./script/gate` verde no worktree (com a perna wasm e a perna urnaui, que
achou `../urna`): antes `cargo test --workspace` 2454 passed, 0 failed,
16 ignored em 114 binários; depois 2465 passed, 0 failed, 16 ignored em
115 binários (+11: os testes novos). clippy `-D warnings` e fmt limpos.
`cargo run -p showcase --example regress -- check`: 159 baselines, 159
iguais, 0 mudaram, sem frame desenhado. contagem de linhas:
`gpu/context.rs` 507 de produção (era 461), `gpu/recover.rs` 231,
`gpu/loss.rs` 118, tudo abaixo de 639.

### ADR

`docs/ADR/gpu-device-loss-recovers-in-place.md` (proposed): o sinal, a
ordem de recriação, o que sobrevive e o que não, o que o app faz (nada; a
exceção é recurso próprio no device). linha pro índice, que é do
orquestrador (`docs/ADR/README.md` é proibido aqui):

| 2026-09-24 | [gpu-device-loss-recovers-in-place](gpu-device-loss-recovers-in-place.md) | proposed | `GpuContext` instala o callback de device lost em todo device e guarda o flag; `recover` refaz adapter, device, surface config, pipelines e atlas pelo mesmo `from_device`; o frame recupera antes do acquire ou pula com um erro por perda; o store de imagens vira log de uploads com os pixels; provado headless com `Device::destroy` e pixels iguais antes e depois (task-76 fase 1) |

### interfaces adicionadas (só crescimento)

`GpuContext::{device_lost, lost_device, poll_device_loss, recover,
recover_async, set_recovery_backends}`, `gpu::{LostDevice, RecoverError,
recover_frame_resources, recover_frame_resources_async}`,
`TextSystem::recreate_atlas`, `TexturePool::clear`,
`Compositor::drop_gpu_resources`, `Renderer::{gpu, gpu_mut}`,
`HeadlessError::DeviceLost`, `ImageStore::upload_count` (no lugar de
`has_pending`, que só o próprio módulo usava). nenhuma assinatura mudou;
`Renderer::draw` continua `u32`. showcase, ide, urnaui e wasm compilam sem
mudança.

### limitações e decisões, sem esconder

- a perda real (driver, sleep, GPU trocada) não foi provocada nesta
  máquina: o que está provado é o caminho `Destroyed`. o caminho `Unknown`
  usa o mesmo flag e a mesma recuperação; o que difere é só quando o
  callback chega (na hora, sem poll).
- no wasm não há recuperação, só o sinal e o erro uma vez por perda.
- a recuperação bloqueia o frame pelo tempo de subir adapter e device
  (o mesmo que um contexto headless leva pra nascer).
- `Renderer::draw` devolve `0` no frame pulado, não `None`: mudar pra
  `Option`/`Result` quebraria consumidores fora do escopo (ide lab, rec,
  showcase measure usam o valor como statement e `Result` é `must_use`).
- o `HeadlessError` ganhou uma variante; ninguém faz match exaustivo nele.
- a cópia dos pixels no `ImageStore` dobra a memória de imagens na CPU
  (o atlas é grow-only, então é o orçamento que a textura já tinha).
- `recreate_surface` (resume do Android) continua com o próprio request
  de adapter e sem passar por `surface_configuration`; não mexi por ser
  outro caso (surface nova, device vivo).
- `SurfaceError::Lost` num device perdido adia a recuperação pro próximo
  frame pedido, via `compositor.invalidate()`; os shells que não olham
  `needs_render()` no `about_to_wait` (nenhum dos atuais) não pediriam
  esse frame sozinhos.
- a cadeia `recover_or_skip` é `pub(crate)`; os apps entram por
  `present_frame`, que já é o caminho de todos.

### o que os itens 2 e 7 precisam disto

- item 2 (múltiplas janelas): cada janela é um `GpuContext` com o próprio
  `Instance`, callback e flag; `recover` é por contexto e a surface de cada
  uma pertence ao seu instance. o `ImageStore` é global e agora não drena:
  o log com cursor por atlas foi feito pra dois atlases sobre um store, e
  é o que a segunda janela precisa. o que falta lá: um `TextSystem` e um
  `EffectProcessor` por contexto (já são), e decidir se a perda de um
  device de janela é a perda de todos (no metal e no vulkan, um adapter
  perdido derruba todos os devices dele; recuperar cada um no seu frame
  funciona sem coordenação).
- item 7 (sleep e wake): a perda por sleep é o caso de uso deste item; o
  wake só precisa pedir um frame (`invalidate` + `request_redraw`) e a
  recuperação acontece antes do acquire. o que o item 7 acrescenta é
  pausar animação e trabalho de fundo, e o "sem frame velho" já está
  provado pelo segundo teste. se o driver entregar a perda como
  `SurfaceError::Lost` sem callback, o caminho do `Lost` já pede o frame
  que recupera.

## handoff do item 3: diálogos de arquivo e drag and drop

branch `task/76b-file-dialogs`, base `e3dc7e1f` (main). commits, em ordem:

- (este) task-76: the file dialog and drop probe

### a sonda

(a) o que o urnaui faz hoje com `rfd` e com os eventos de drop, por
linha:

- `crates/urnaui/Cargo.toml:45`: `rfd = "0.15"`, features padrão
  (`xdg-portal` e `async-std`), na seção
  `[target.'cfg(not(target_arch = "wasm32"))']`. o `rfd` 0.15.4 só tem
  backend para linux/bsd, macos, windows e wasm (`backend.rs:7-45`): em
  android e ios `FileDialog::pick_file` não compila (nenhum
  `FilePickerDialogImpl`). o urnaui só chama `rfd` sob
  `not(any(wasm32, android, ios))`, mas declara a dependência para todo
  não-wasm; no engine ela entra na seção do `arboard`, que já exclui
  android e ios.
- `crates/urnaui/src/view/explorer.rs:400-407`, `Action::PickFile` no
  desktop: `rfd::FileDialog::new().set_title("Open a .urna corpus")
  .add_filter("urna corpus", &["urna", "nest"]).pick_file()`; `Some(path)`
  vira `open_database(path)`, `None` (cancelado) não faz nada. síncrono,
  modal, sem parent, sem diretório inicial.
- `crates/urnaui/src/view/explorer.rs:445-476`, `export_blob` no desktop:
  `rfd::FileDialog::new().set_title("Export media blob")
  .set_file_name(&name).save_file()`; `Some(dest)` vira
  `UrnaCommand::ExportBlob { index, dest }` e um toast; `None` devolve
  `true` (tratado, nada a fazer). `explorer.rs:478-486`: em wasm, android e
  ios a exportação só toasta "blob export needs the desktop app".
- `crates/urnaui/src/view/explorer.rs:393-399` e `app.rs:482-488`: em
  wasm `PickFile` marca `pick_requested`; o shell dispara
  `web_picker::trigger` no `about_to_wait`, e o `<input type=file>`
  devolve `UserEvent::FileLoaded { name, bytes }` (bytes, nunca caminho),
  que vira `view.file_picked`. é a entrada de arquivo do alvo web e não
  passa por `rfd`.
- `crates/urnaui/src/app.rs:362-377` (só não-wasm):
  `WindowEvent::DroppedFile(path)` vira `view.file_dropped(path)` mais
  invalidate; `HoveredFile(_)` vira `view.set_file_hover(true)`, o caminho
  do arquivo pairando é descartado; `HoveredFileCancelled` vira
  `set_file_hover(false)`.
- `crates/urnaui/src/view/mod.rs:217-233`: `file_dropped(path)` desliga o
  hover, chama `explorer.open_database(path)` e toasta "opening dropped
  file…"; `set_file_hover(bool)` repassa ao explorer.
- `crates/urnaui/src/view/explorer.rs:67,121,186-190,869` e
  `open.rs:32-33,414-435`: `file_hover: bool` chega à tela Open como
  `OpenContext.file_hover` e desenha um contorno de acento em volta do
  conteúdo com a dica "release to open". `open.rs:276-279`: a dica de
  status muda por alvo ("drop a .urna file anywhere" no desktop, "pick a
  .urna file (drag-and-drop is not available on the web canvas)" na web).
- o que a view recebe: só o `PathBuf`, um por evento. posição, nenhuma.
  um drop de vários arquivos chega como N `DroppedFile`, e o urnaui
  chama `open_database` N vezes (o último ganha).

(b) o que compila no wasm no `rfd` 0.15.4
(`~/.cargo/registry/src/*/rfd-0.15.4/src/`):

- `lib.rs:114-119`: `FileDialog` (síncrono) é
  `#[cfg(not(target_arch = "wasm32"))]`; `AsyncFileDialog` existe em todo
  alvo.
- `backend.rs:29-30` e `backend/wasm.rs`: no wasm `AsyncFileDialog::
  pick_file` e `pick_files` abrem um overlay DOM próprio (`rfd-overlay`,
  com o `style.css` do crate) com um `<input type=file>` e devolvem um
  `FileHandle` que embrulha `web_sys::File`: `file_name()` e `read()`
  assíncrono em bytes, sem caminho. filtros viram o atributo `accept`.
  `set_directory` não existe no wasm.
- `backend/wasm/file_dialog.rs`: `save_file` resolve na hora com um
  handle "writable"; `FileHandle::write(bytes)` mostra um link de
  download. `file_dialog.rs:258-271`: `pick_folder` e `pick_folders` são
  `not(wasm32)`, "does not exist in WASM32".
- dependências do `rfd` no wasm (`Cargo.toml` dele): `js-sys`,
  `wasm-bindgen`, `wasm-bindgen-futures` e `web-sys` com doze features
  (Document, Element, HtmlInputElement, HtmlButtonElement,
  HtmlAnchorElement, Window, File, FileList, FileReader, Blob,
  BlobPropertyBag, Url). o engine já tem os quatro crates na seção wasm
  (`crates/engine/Cargo.toml:97-102`), então o `rfd` compilaria para wasm
  fora da seção de desktop, ao custo das features extras no `web-sys` e
  de um overlay DOM que não é do engine.
- conclusão: o trait do engine é síncrono e devolve caminho, porque as
  duas chamadas do urnaui querem um `PathBuf` (abrir por caminho, exportar
  para um caminho). no wasm o `rfd` só dá bytes e só assíncrono, então a
  resposta do engine no wasm é `Unsupported`, e o `web_picker.rs` do
  urnaui continua sendo a entrada de bytes da web (é outra capacidade,
  fora dos arquivos permitidos aqui). o `rfd` fica na seção de desktop:
  `[target.'cfg(all(not(target_arch = "wasm32"), not(target_os = "android"),
  not(target_os = "ios")))']`, a mesma do `arboard`.

(c) o molde do `clipboard.rs` (`crates/engine/src/clipboard.rs`):

- `ClipboardProvider` (linhas 9-14): o trait, compilado em todo alvo.
- `LocalClipboard` (16-36): a implementação em memória, compilada em todo
  alvo; é o falso dos testes e o fallback de wasm e mobile.
- `SystemClipboard` (40-75): `arboard`, sob
  `#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]`;
  `new()` degrada para `None` quando o `arboard` falha (CI headless) e as
  operações viram no-op com um `log::warn!`.
- `default_clipboard() -> Box<dyn ClipboardProvider>` (78-89): o `cfg`
  escolhe `SystemClipboard` no desktop e `LocalClipboard` no resto.
- `crates/engine/Cargo.toml:82-85`: `arboard` só na seção
  `cfg(all(not(wasm32), not(android), not(ios)))`.
- por teste: quem consome guarda `Box<dyn ClipboardProvider>` ou o tipo
  concreto; teste usa `LocalClipboard`. o urnaui constrói
  `SystemClipboard::new()` direto (`explorer.rs:80,124`, `app.rs:333`).
- o módulo de diálogos segue o mesmo desenho: `FileDialogs` (trait),
  `FakeDialogs` (em todo alvo; responde o que o teste carregou e registra
  o que foi pedido), `NativeDialogs` (`rfd`, sob o cfg do `arboard`),
  `NoDialogs` (wasm e mobile: `Unsupported`), `default_dialogs()`.

o que entra de dependência (e o motivo da parada): o `Cargo.lock` da raiz
não tem `rfd`. o urnaui é workspace próprio (`crates/urnaui/Cargo.lock`
tem `rfd 0.15.4`); a raiz não. entrando no engine, o lock da raiz ganha:

- `rfd 0.15.4` e `block2 0.6.2` (a raiz só tem `block2 0.5.1`; o 0.6.2
  está no cache local do registry). `dispatch2 0.3.1`, `objc2 0.6.4`,
  `objc2-app-kit 0.3.2`, `objc2-core-foundation 0.3.2`,
  `objc2-foundation 0.3.2`, `raw-window-handle 0.6.2`, `log` já estão;
  `objc2-app-kit` ganha features (NSOpenPanel, NSSavePanel, NSAlert,
  NSPanel e outras). é tudo que compila no macOS.
- com as features padrão do `rfd` (`xdg-portal`, `async-std`), o lock
  ganha também `ashpd 0.11`, `urlencoding 2.1` e a árvore do `ashpd`
  (zbus 5 e o que vem com ele: async-fs, async-net, enumflags2,
  serde_repr, wayland-backend, wayland-client, wayland-protocols, rand,
  url e as dezenas de crates abaixo), como entradas só de linux e bsd,
  nunca compiladas aqui. o `.crate` do `ashpd` não está no cache local
  (o macOS nunca precisou baixar), só o índice.
- com `default-features = false`, entram só `rfd` e `block2 0.6.2`. no
  linux o `rfd` fica sem backend até o bring-up escolher `xdg-portal`
  (daemon de portal em runtime) ou `gtk3` (headers C), o que é decisão
  daquele bring-up, hoje fora de escopo.
- no wasm, o lock registra as dependências wasm do `rfd` de qualquer
  forma (todas já presentes); elas só compilam se o `rfd` estiver na
  seção wasm, e não estará.

o que o winit 0.30.13 entrega no drop
(`~/.cargo/registry/src/*/winit-0.30.13/src/platform_impl/macos/window_delegate.rs:367-430`):

- `draggingEntered:` enfileira um `HoveredFile(path)` por arquivo do
  pasteboard; `performDragOperation:` um `DroppedFile(path)` por arquivo;
  `draggingExited:` um `HoveredFileCancelled`. nenhum traz posição, e o
  `draggingUpdated:` não está implementado, então não há `CursorMoved`
  enquanto um arquivo é arrastado (o mouse pertence à sessão de drag, não
  à janela). o `event.rs:180-192` documenta o mesmo para todo backend
  (x11 e windows são os outros dois com drop). a posição lógica pedida
  pelo briefing não existe no winit 0.30: o evento do engine a carrega
  como `Option`, `None` até o winit expor a `draggingLocation`.
- os N `DroppedFile` de um drop múltiplo são enfileirados juntos, antes
  de qualquer redraw: o `App` do engine, que bufferiza eventos até o
  `RedrawRequested`, pode juntá-los num `Dropped` só; um shell que
  entrega evento a evento à view (urnaui, showcase) recebe um por
  arquivo, como hoje.

o shell do showcase: `crates/showcase/src/app.rs:341-478` não trata
`DroppedFile`, `HoveredFile` nem `HoveredFileCancelled`, e o arquivo é
proibido nesta task. a página Files recebe o drop pelos testes, pelo
headless e pelo dispatcher; para o drop chegar numa janela de verdade o
shell precisa de um braço a mais no `match`, que repassa o evento do
engine a `ShowcaseView::handle_file_drop`.
