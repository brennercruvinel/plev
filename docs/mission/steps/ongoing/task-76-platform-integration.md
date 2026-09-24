---
project: plev
audience: [ai-agents, contributors]
status: ongoing
last-updated: 2026-09-24
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

- `6b8bd395` task-76: the file dialog and drop probe
- `b1a2e686` task-76: the file dialogs behind a trait, with a fake
- `c90d1e1d` task-76: the file drop as an event of the engine
- `e65564a3` task-76: the urnaui opens and exports through the engine's
  dialogs
- `fe5a9279` task-76: the Files page, the dialog and the drop as
  capabilities
- `77800d1b` task-76: the explorer's dialogs in their own module
- (este) task-76: ADR and the item 3 handoff

decisões do orquestrador depois da sonda: `rfd` sem features padrão,
inline em `crates/engine/Cargo.toml`, na seção do `arboard`, lock com
`--offline` (opção B); o shell do showcase ganha um braço só (opção 2);
`open_file` com `Actors { human, assistive, test: true; agent: false }`;
posição opcional, junção no `App` do engine, trait síncrono por caminho,
`web_picker.rs` intocado.

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

### o trait, o falso e o que cada alvo faz

`crates/engine/src/platform/files.rs` (350 linhas de produção).
`FileDialogs` tem `pick_file`, `pick_files`, `save_file` e `pick_folder`;
cada um recebe `&DialogRequest` (título, `FileFilter { name, extensions }`
por `filter(name, &[ext])`, `directory`, `file_name`) e devolve
`Picked<T> = Result<Option<T>, DialogError>`: `Ok(None)` é a pessoa
cancelando, `Err(DialogError::Unsupported(DialogKind))` é alvo sem
diálogo. tudo síncrono e modal na thread que pediu, como o
`rfd::FileDialog`.

- `NativeDialogs` (só sob o cfg do `arboard`): monta o `rfd::FileDialog`
  por chamada (`set_title`, `add_filter`, `set_directory`,
  `set_file_name`), sem parent, e chama `pick_file`, `pick_files`,
  `save_file` ou `pick_folder`. comportamento igual ao que o urnaui fazia.
- `FakeDialogs` (todo alvo): `will_pick`, `will_pick_all`, `will_cancel`,
  `will_refuse` carregam respostas em ordem; fila vazia é cancel (uma
  página que pergunta sem o teste esperar não muda nada); cada chamada
  fica em `Asked { kind, request }` (`asked()`, `take_asked()`,
  `pending()`). `impl FileDialogs for Arc<Mutex<D>>` deixa um teste ficar
  com um handle e dar o outro à página como `Box<dyn FileDialogs>`.
- `NoDialogs` (todo alvo; o padrão em wasm, android e ios): todo tipo
  responde `Unsupported`.
- `default_dialogs() -> Box<dyn FileDialogs>`: `NativeDialogs` no
  desktop, `NoDialogs` no resto, pelo mesmo cfg do `default_clipboard`.

`crates/engine/Cargo.toml`: `rfd = { version = "0.15", default-features =
false }` na seção `cfg(all(not(wasm32), not(android), not(ios)))`, ao lado
do `arboard`, declarado inline (a raiz `Cargo.toml` é proibida). o
`Cargo.lock` da raiz ganhou `rfd 0.15.4` e `block2 0.6.2` (o `block2
0.5.1` do winit fica), nada mais; `objc2-app-kit` ganhou as features dos
painéis. no linux o `rfd` fica sem backend até o bring-up de linux
escolher `xdg-portal` ou `gtk3`; linux não é alvo. o wasm compila sem o
`rfd` (a perna wasm do gate prova).

testes (7, em `files.rs`): o falso responde em ordem e registra título,
filtro, diretório e nome; fila vazia cancela; um diálogo de um caminho
pega o primeiro e lista vazia cancela; `will_refuse` é o mesmo erro do
alvo sem diálogo, com o texto; o handle compartilhado responde por um
lado e reporta pelo outro; `NoDialogs` recusa os quatro tipos; o builder
do `rfd` carrega o pedido inteiro (conferido pelo `Debug` dele, sem abrir
nada). nenhum teste constrói um painel.

### o evento de drop

`crates/engine/src/input/drop.rs` (110 de produção): `FileDrop::{Hovered
{ paths, position }, Dropped { paths, position }, Cancelled}`, posição em
pixels lógicos e `Option`. `FileDrop::from_window_event(&WindowEvent)`
é a única tradução: `HoveredFile(p)` vira `Hovered` com um caminho,
`DroppedFile(p)` vira `Dropped` com um caminho, `HoveredFileCancelled`
vira `Cancelled`, o resto `None`. `join(next)` funde dois passos da mesma
fase (anexa os caminhos, guarda a primeira posição conhecida) e devolve
`next` quando as fases diferem ou um é cancel. `paths()`, `position()`,
`is_drop()`.

`input/types.rs`: `InputEvent::FileDrop(FileDropEvent { view_id:
Option<ViewId>, drop })`. `input/handlers.rs`:
`InputState::handle_file_drop(drop)` faz hit test na posição do drop,
quando há, e enfileira; sem posição, `view_id` é `None` e o app decide o
que um drop na janela inteira quer dizer. o cursor não serve de
substituto: durante o arraste o mouse pertence à sessão de drag e o
cursor da janela está parado onde estava antes.

`window/events.rs` (o `App` do engine): `BufferedEvent::FileDrop`; os três
eventos do winit passam por `from_window_event`; `buffer_event` funde o
drop novo com o último bufferizado da mesma fase, então os N
`DroppedFile` de um `performDragOperation:` chegam como um `Dropped` com
todos os caminhos antes do frame; `process_buffered_events` chama
`handle_file_drop`.

testes: 4 em `drop.rs` (os três eventos traduzem e mais nada, leitura de
cada fase, junção com a posição, fases diferentes e cancel não juntam) e
1 em `input/tests.rs` (o drop chega à view sob a posição, com caminhos e
posição, por hover e por drop; `begin_frame` preserva; sem região ou sem
posição chega a nenhuma view; o cancel chega; o pointer e o hover não se
mexem). `crates/engine/examples/input/state.rs` ganhou o braço
`InputEvent::FileDrop` que a variante nova exige (único match exaustivo
no repositório).

### o que mudou no urnaui

- `crates/urnaui/Cargo.toml`: `rfd` sai (o comentário aponta pro engine).
  `crates/urnaui/Cargo.lock`: 370 linhas a menos, 35 pacotes fora
  (`ashpd`, `url`, `icu_*`, `zbus`... a árvore de linux que o `rfd` com
  features padrão trazia); `rfd` continua no lock pelo engine, sem
  features.
- `crates/urnaui/src/view/explorer.rs`: campo `dialogs: Box<dyn
  FileDialogs>` (`default_dialogs()`); `set_dialogs`, `opening` e
  `file_hover` sob `cfg(test)`; `Action::PickFile` chama `pick_file` fora
  do wasm (no wasm continua marcando `pick_requested` pro `web_picker`);
  `set_file_hover` perdeu o cfg (compila em todo alvo, a web nunca manda).
  992 linhas de produção (era 1019, congelado pelo guard de higiene).
- `crates/urnaui/src/view/explorer/dialogs.rs` (novo, 85 linhas):
  `pick_file` (título "Open a .urna corpus", filtro "urna corpus" com
  `urna` e `nest`; `Ok(Some)` abre, `Ok(None)` nada, `Err` toasta "the
  file picker needs the desktop app") e `export_blob`, uma versão só
  pra todo alvo (título "Export media blob", `file_name` do blob; `Err`
  toasta "blob export needs the desktop app", o texto que o mobile e a
  web já mostravam). as duas variantes cfg'd de antes viraram uma.
- `crates/urnaui/src/view/mod.rs`: `handle_file_drop(&FileDrop) -> bool`
  no lugar de `file_dropped` e `set_file_hover`: `Hovered` liga o hover
  da tela Open, `Cancelled` desliga, `Dropped` desliga e abre cada caminho
  em ordem com o toast "opening dropped file…" (o último ganha, como
  antes). compila em todo alvo.
- `crates/urnaui/src/app.rs`: os três braços cfg'd de drop saíram; o
  braço `other =>` entrega todo evento não tratado a
  `engine::input::FileDrop::from_window_event` e, se é drop, a
  `view.handle_file_drop` mais `invalidate`. `app.rs` não nomeia
  `DroppedFile`, `HoveredFile` nem `HoveredFileCancelled`; o engine
  decide o que é drop.
- `crates/urnaui/src/web_picker.rs` não mudou: é a entrada de bytes da
  web, outra capacidade, e o candidato a um `AsyncFileDialog` do engine
  numa fase futura.

como conferi sem abrir diálogo (3 testes novos, o falso no lugar do
nativo): `browse_opens_what_the_dialog_picked_and_nothing_when_cancelled`
(cancel não abre nada; um caminho inexistente abre e falha como um
digitado, voltando à tela Open com o erro; os dois pedidos registrados
com o título, o filtro e sem diretório; `NoDialogs` toasta e não abre),
`export_asks_a_save_dialog_named_after_the_blob` (cancel sem toast nem
exportação; um caminho manda `ExportBlob` ao worker, que sem arquivo
aberto responde a falha; refuse toasta; blob inexistente nem pergunta),
`a_file_drag_shows_the_hover_and_a_drop_opens_the_file` (a view: hover
liga, cancel desliga, drop desliga, abre e toasta).

### a página Files (PNG e projeção)

`crates/showcase/src/view/apps/files.rs` (394 linhas), `files/semantics.rs`
(289) e `files/tests.rs` (729, 8 testes), grupo Apps Demo, depois de
Silences, ícone `folder-open`.

modelo semântico: `files` (Region; `received`, a lista dos caminhos na
ordem, e `count`), `files.drop` (Region "drop files here"; `drop_files`
com param `List` não constrained, lendo `files:received`, todo ator: o
drop da pessoa e o de um agente pela porta são a mesma capacidade),
`files.open` (Button "open file", focusable; `open_file` lendo
`files:received`, policy `Actors { human: true, assistive: true, test:
true, agent: false }`) e `files.status` (`message`). as duas ações leem a
lista porque as duas mudam a lista. foco em `files.open` quando o botão
tem o teclado.

execução (`Operable`): `open_file` chama
`dialogs.pick_file(DialogRequest::new().title("Open a file"))`;
`Some(path)` recebe (outcome `path`, `count`), `None` completa com
`cancelled: true` e o status "the dialog was cancelled", `Err` falha com
o texto do erro e o status "no file dialog on this target".
`drop_files` recebe os caminhos do param (outcome `received`, `count`);
lista vazia falha "the drop carries no file".

entrada: clique no botão vira `open_file` por `Actor::Human` pelo
dispatcher da galeria (o do todo, emprestado como pra Silences).
`handle_file_drop`: `Hovered` acende a zona (borda de acento, "release
to receive"), `Cancelled` apaga, `Dropped` com posição fora da zona não
recebe, com posição dentro ou sem posição (o que o winit 0.30 manda)
vira `drop_files` por `Actor::Human` com os caminhos como `List`. Tab
foca e desfoca o botão, Enter pressiona (`open_file`), Escape solta.
`FilesPage::new()` usa `default_dialogs()` fora de teste e `FakeDialogs`
sob `cfg(test)`: nenhum teste do crate abre diálogo nativo, por
construção.

registro como a Silences: `Pane::Files` (`pages/mod.rs`), a linha em
`pages/table.rs`, o campo em `panes.rs`, `dispatch.rs` (`content_height`,
`page_event`, `page_file_drop` novo, `page_render`, `semantics`, `act`,
`assist_focus`), `input.rs` (`ShowcaseView::handle_file_drop` novo,
`handle_edit_key`, `handle_enter`, `close_top_overlay`). shell,
`crates/showcase/src/app.rs:476-485` (aprovado, opção 2): o braço
`other =>` do `match` entrega todo evento não tratado a
`FileDrop::from_window_event` e `view.handle_file_drop`, com `changed()`.

testes (`files/tests.rs`): a projeção (roles, labels, policy do
`open_file` igual a `OPEN_FILE_ACTORS`, param e reads do `drop_files`,
status, sem foco); o drop pela pessoa (hover acende uma vez, cancel
apaga; sem posição recebe dois caminhos e o recibo 1 é `Human`,
`files.drop`, `drop_files`, param com os dois, `Completed` com `received
2`, `count 2`; fora da zona não recebe e nenhum recibo nasce; dentro
recebe; inert é `Rejected(Inert)`; vazio é `Failed`; o falso nunca é
perguntado; a zona é o binding de `files.drop`); o botão (clique pega
"/home/me/notes.md" e o recibo traz `path`; cancel completa com
`cancelled`; Tab foca, Enter com refuse é `Failed` com "pick_file: no
native file dialog on this target" e o status diz; Escape solta; três
pedidos `PickFile` com "Open a file" e sem filtro; `NoDialogs` falha
igual); o agente em processo (`open_file` é `Rejected(NotAuthorized)` e
o falso não é perguntado; `drop_files` completa como do agente; o mesmo
pedido com a observação velha é `Rejected(Stale)`; `Actor::Test` pode e
o falso responde); o agente pela porta real (`AgentPort` num diretório
temporário, thread do agente com `wire::Client`: observa `files` com
bounds e vê os quatro nós e as caixas da zona e do botão; `open_file` é
`requested, rejected`, um recibo e não um erro; `drop_files` com param
é `requested, accepted, executing, completed`; a lista mostra o caminho;
o dispatcher da galeria guarda os dois recibos como do agente "claude");
duas viewports (900 e 358 de largura: card na largura do conteúdo, zona,
botão e status dentro dele, botão abaixo da zona, status ao lado, o card
cresce com três arquivos e a altura é a do binding); as rotas da galeria
(a página Todo não recebe drop; na Files o drop chega pela view, `act`
assistivo completa, `assist_focus`, Tab, Escape, Enter cancela no falso,
os quatro bindings, em repouso a página não pede frame); o golden.

golden: `crates/uir/fixtures/showcase-files.snapshot.json` (4 nós,
revision `d7a661b0bdabefd9`), mantido por
`files::tests::the_page_projection_is_the_uir_golden`; `UIR_BLESS=1
cargo test -p showcase --lib files::tests::the_page_projection` regrava.
os testes do `uir` continuam verdes.

baselines: `crates/showcase/fixtures/regress/files@core@1280x800` e
`files@hoff@1280x800` (`semantics.json`, `scene.json`, `frame.png`; 4
layers, 4 controles bound, 4 nós, settled em 2 passos), abençoadas com
`cargo run -p showcase --example regress -- bless --page Files --theme
core --png` e o mesmo com `hoff`.

render headless: `cargo run -p showcase -- --render Files --theme core
--out files-core.png --semantics files-core.json` e o mesmo com `hoff`
deram dois PNG 1200x800 (o card centrado na largura legível, a zona com
"drop files here", o botão "open file" com o ícone de pasta, o status
"no files yet: drop some on the window, or open one" ao lado); o
`frame.png` de cada baseline é a mesma cena a 1280x800 e fica na árvore.
a projeção do render é o golden byte a byte (só o `\n` final que o teste
acrescenta).

### gate e regress

`df -h /` antes de cada gate: 15 GB livres no primeiro, 12 GB no
segundo. o primeiro `./script/gate` ficou vermelho no
`file_hygiene_guard`: `crates/urnaui/src/view/explorer.rs` está
congelado em 1019 linhas de produção ("pode encolher, nunca crescer") e
os diálogos o levaram a 1056; o commit `77800d1b` moveu `pick_file` e
`export_blob` pra `view/explorer/dialogs.rs` e o arquivo ficou em 992.
o segundo gate, verde no worktree, com a perna wasm e a perna urnaui
(`../urna` presente): antes `cargo test --workspace` 2465 passed, 0
failed, 16 ignored em 115 binários; depois 2485 passed, 0 failed, 16
ignored em 115 binários (+20: 7 do `files.rs` do engine, 5 do drop, 8 da
página). perna urnaui: antes 82 passed, 1 ignored; depois 85 passed, 0
failed, 1 ignored em 6 binários (+3). clippy `-D warnings`, fmt e a
checagem wasm limpos nas duas pernas.

`cargo run -p showcase --example regress -- check`: 161 baselines (eram
159), 161 iguais, 0 mudaram, sem frame desenhado.

linhas de produção (regra do guard): engine `platform/files.rs` 350,
`input/drop.rs` 110, `input/handlers.rs` 165, `input/types.rs` 256,
`window/events.rs` 227; showcase `apps/files.rs` 394,
`files/semantics.rs` 289, `view/dispatch.rs` 415, `view/input.rs` 394;
urnaui `view/explorer.rs` 992 (era 1019, congelado, encolheu),
`view/explorer/dialogs.rs` 85, `app.rs` 562. tudo abaixo de 639, salvo o
congelado.

### ADR

`docs/ADR/file-dialogs-and-drops-are-capabilities.md` (proposed): o
trait e o falso, o `rfd` sem features no desktop, o evento de drop com
posição opcional e a junção, o drop como request, a regra "diálogo
nativo só abre por pessoa" na policy, o que o wasm faz, o que os itens 4
e 8 reaproveitam. linha pro índice, que é do orquestrador
(`docs/ADR/README.md` é proibido aqui):

| 2026-09-24 | [file-dialogs-and-drops-are-capabilities](file-dialogs-and-drops-are-capabilities.md) | proposed | `engine::platform::files` (trait síncrono por caminho, `rfd` no desktop sem features padrão, `FakeDialogs` pra teste, `NoDialogs` em wasm e mobile) e `engine::input::FileDrop` (uma tradução dos três eventos do winit, posição opcional, junção dos N por arquivo no `App`); o urnaui usa os dois sem segunda implementação; a página Files do showcase projeta `open_file` só pra pessoa e `drop_files` pra todo ator, pelo dispatcher; diálogo nativo só abre por pessoa (task-76 item 3) |

### interfaces adicionadas (só crescimento)

`engine::platform::files::{FileDialogs, DialogRequest, FileFilter,
DialogKind, DialogError, Picked, FakeDialogs, Answer, Asked, NoDialogs,
NativeDialogs (desktop), default_dialogs}` e `impl FileDialogs for
Arc<Mutex<D>>`; `engine::input::{FileDrop, FileDropEvent}`,
`InputEvent::FileDrop`, `InputState::handle_file_drop`;
`ShowcaseView::handle_file_drop`; `UrnauiView::handle_file_drop` (no
lugar de `file_dropped` e `set_file_hover`, cujo único consumidor era o
`app.rs` do próprio urnaui). `InputEvent` ganhou uma variante; o exemplo
`input` do engine acompanhou e não há outro match exaustivo. nenhuma
assinatura existente mudou.

### arquivos tocados fora da lista permitida, e por quê

- `crates/engine/examples/input/state.rs`: o braço `InputEvent::FileDrop`
  (a variante nova quebrava o match exaustivo; clippy `--all-targets`
  compila o exemplo).
- `crates/urnaui/src/view/mod.rs`: `file_dropped` e `set_file_hover`
  viraram `handle_file_drop` (a tradução do drop do urnaui morava aí
  também, não só em `app.rs`); um teste novo.
- `crates/urnaui/src/view/explorer/dialogs.rs` (novo): o guard de higiene
  congela `explorer.rs`; sem o módulo filho o gate fica vermelho.
- `crates/showcase/src/app.rs`: o braço aprovado (opção 2).
- `crates/urnaui/Cargo.lock`: consequência do `Cargo.toml`.

### limitações e decisões, sem esconder

- posição: `None` em todo backend do winit 0.30; a zona da página só
  recusa um drop fora dela quando há posição, o que na janela hoje nunca
  acontece (provado com `Some` nos testes). quando o winit expuser a
  `draggingLocation`, `from_window_event` preenche e nada mais muda.
- drop múltiplo: o `App` do engine entrega um `Dropped` com todos os
  caminhos; urnaui e showcase, que entregam evento a evento, recebem um
  por arquivo (o urnaui abre cada um e o último ganha, como antes; a
  Files recebe um `drop_files` por arquivo).
- wasm: `Unsupported`; o `web_picker.rs` do urnaui fica e é o candidato a
  um `AsyncFileDialog` do engine (bytes, assíncrono).
- mobile: o Browse do urnaui agora toasta "the file picker needs the
  desktop app" (antes o botão existia e não fazia nada em android e ios);
  a exportação já toastava o equivalente.
- linux: sem backend de diálogo até o bring-up escolher (features do
  `rfd` desligadas); o urnaui tinha `xdg-portal` por conta própria. só
  linux muda, e linux não builda hoje.
- `NativeDialogs` não passa `set_parent` (o urnaui nunca passou): o
  painel abre como modal da aplicação. o diálogo real não foi aberto
  nesta máquina (proibido); a prova é o builder por `Debug` e a leitura
  do `rfd`; a abertura de verdade fica pro orquestrador.
- `FilesPage::new()` só usa o falso sob `cfg(test)` do próprio crate: um
  teste de integração em `crates/showcase/tests/` que construísse a view
  e disparasse `open_file` como pessoa abriria o painel nativo. nenhum
  faz (o `stage_page` nunca dispara nada), e a policy barra o agente
  antes; fica registrado como risco de quem escrever esse teste.
- o `Cargo.lock` da raiz cresceu dois pacotes e `objc2-app-kit` ganhou
  features; o custo de binário não foi medido.
- `explorer.rs` continua congelado (992, era 1019); o split completo
  por responsabilidade é dívida anterior a esta task.

### o que os itens 4 e 8 reaproveitam daqui

- item 4 (menus nativos): o item "Open…" de um menu é outra
  representação de `open_file`: dispara um `Request` por `Actor::Human`
  no mesmo dispatcher, e a policy "só pessoa" já está declarada no nó,
  não no menu. o molde é o mesmo desta fase: trait no
  `engine::platform`, implementação real sob o cfg do `arboard`, um
  falso que registra o que foi pedido, `default_*()` por alvo. o `rfd`
  não faz menu; o crate de menu é a dependência a aprovar.
- item 8 (alertas e confirmações nativas): `rfd::MessageDialog` e
  `AsyncMessageDialog` já vêm na dependência do engine; um trait `Alerts`
  com `NativeAlerts`, `FakeAlerts` e `NoAlerts` segue `files.rs` linha
  por linha, com o mesmo `Picked`-like (`Ok(resposta)`, `Unsupported`).
  a regra "painel nativo só por pessoa" vale igual, e a confirmação
  humana da task-58 (`Confirm::ForAgents`) pode ser o `MessageDialog`
  com OkCancel como uma das opções.
- os dois reaproveitam a forma do drop: um evento do engine, uma
  tradução `from_window_event`, o shell repassa pelo braço `other =>`
  sem nomear nada do winit.

### comandos de validação

```text
export CARGO_TARGET_DIR=/Users/nn/Dev/hoff/plevdev/target/agents/task-76
export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_PROFILE_TEST_DEBUG=line-tables-only
cargo test --offline -p engine -p showcase
CARGO_TARGET_DIR=$CARGO_TARGET_DIR/urnaui cargo test --offline --manifest-path crates/urnaui/Cargo.toml
./script/gate
cargo run --offline -p showcase --example regress -- check
cargo run --offline -p showcase -- --render Files --theme core --out files.png --semantics files.json
UIR_BLESS=1 cargo test --offline -p showcase --lib files::tests::the_page_projection   (regrava o golden)
cargo run --offline -p showcase --example regress -- bless --page Files --theme core --png   (e --theme hoff)
```

## handoff do item 2: múltiplas janelas no engine

branch `task/76c-windows`, base `153d4c65` (main). commits, em ordem:

- `3b642d82` task-76: the windows probe
- `6da95f2a` task-76: the WindowHost, the Windows map, and the App over
  them with one window
- `bd97c817` task-76: the layers example opens a second window
- `6e46fd28` task-76: the showcase page in a window of its own
- (este) task-76: ADR and the item 2 handoff

os commits 2 e 3 do briefing (o host; o `Windows` e o `App`) saíram
fundidos em `6da95f2a`: o `App` do engine é o único consumidor do
`Windows` dentro do crate, e cada commit tem que compilar.

### a sonda

(a) o que é "da janela" (morre com ela) e o que é "do app", campo a
campo, como estava antes desta fase:

- `App` do engine (`crates/engine/src/window/mod.rs:49`): da janela
  `window`, `state: GpuState` (o `GpuContext` com a surface, o
  `TextSystem`, o `EffectProcessor`, o `TexturePool`: tudo no device
  daquela janela), `compositor` (as layers e a tabela de binding são o
  frame de uma janela), `input_state` (hit regions, hover, foco, cursor),
  `touch_input` e `touch_pointer`, `ime_state`, `safe_area`,
  `scale_factor`, `frame_clock` e `animation_tick`, `perf` e `perf_hud`
  (contam frames de uma janela e desenham nela), `event_buffer`, `a11y`
  (o `Bridge` do AccessKit se prende a uma janela). do app: `theme`,
  `frame_read` e `frame_write` (o sinal reativo de frame), `lifecycle`
  (Active/Suspended é do processo), `is_animating` (o pedido do app,
  aplicado a cada janela ao abrir), `event_loop_proxy`,
  `narrate_watcher`.
- shell do showcase (`crates/showcase/src/app.rs:82`): da janela
  `window`, `state: GpuState`, `compositor`, `cursor`, `scale_factor`.
  do app: `view: ShowcaseView` (o estado das páginas, o dispatcher da
  galeria, o tema, o chrome), `clock` e `caught_up` (um relógio do loop:
  o tick anda uma vez por iteração, a janela que redesenha só lê o tempo
  real desde o último), `modifiers` (estado do teclado), `proxy`,
  `access` (o bridge é da janela principal; a porta do agente é do app),
  `recording`, `source_click`. dentro da view, três coisas são da janela
  e não do app: `layers: Option<Layers>` (ids de layer nomeiam layers de
  um compositor), `page_scroll` (o scroll de uma janela) e
  `width/height/scale_factor` (o layout de uma janela). elas ficaram na
  view para a janela principal; a segunda janela ganhou as suas em
  `view::dispatch::Detached`.
- o ide (`crates/ide/src/main.rs:106`) e o urnaui
  (`crates/urnaui/src/app.rs:75`) repetem a forma do showcase; fora do
  escopo, não mudaram (nenhum usa o `App` do engine).
- o que o briefing supunha e não é: nenhum exemplo do engine usa
  `engine::window::App`. `counter`, `builder`, `charts`, `layers` e os
  outros têm o próprio `ApplicationHandler` com `GpuContext::new` e
  `TextSystem::new`; o `App` do engine só é usado por
  `lib.rs::run_event_loop`, pelo `android_main` e pelo `wasm_main` do
  engine, e `hot_reload/mod.rs` já o descrevia como "experimental, no
  app uses it". então "os exemplos compilam e rodam sem mudança de
  comportamento" é verdade por não passarem pelo `App`; o exemplo que
  mudou é o `layers`, de propósito.

(b) `crates/engine/tests/windows.rs`,
`two_contexts_draw_two_scenes_in_the_same_loop_and_share_only_the_image_log`
(ficou como teste real): dois `GpuContext::new_headless` (via
`headless::Renderer`) com dois compositores, um rect vermelho, o texto
"one" e uma imagem 8x8 em A, um rect azul e o texto "two" em B, três
rodadas alternadas no mesmo laço. cada frame mostra só o que foi
empurrado nele: o rect da sua cor, o texto do seu atlas de glifos, a
imagem só em A, e os dois frames diferem byte a byte. o que é
compartilhado por desenho: o `ImageStore` global (`gpu/image.rs:232`), o
log de uploads da fase 1: a imagem carregada antes de B existir aparece
em B no frame seguinte a ser empurrada lá, sem segundo `load`. os outros
globais do engine são CPU e por thread (`MEASURE_CTX` do texto, o
`RUNTIME` de sinais, `DEFAULT_TOLERANCE_BITS` do path, os mapas do hot
reload). nada em `gpu/**` precisou mudar; não parei.

(c) o winit 0.30.13 entrega todo `WindowEvent` em
`ApplicationHandler::window_event(event_loop, window_id, event)`, e no
macOS o delegate de cada janela enfileira com o próprio id
(`platform_impl/macos/window_delegate.rs:819`,
`maybe_queue_window_event(self.window().id(), event)`). o shell do
showcase ignorava o id (`_id`). se um segundo `create_window` fosse
chamado nele: `self.window = Some(window)` sobrescreve o `Arc` da
primeira; o `Drop` do `Window` do winit no macOS chama `close()`
(`window.rs:20`), a primeira janela fecha, o winit enfileira
`Destroyed` para ela, e qualquer evento que ainda chegasse com o id da
primeira seria roteado para a view como se fosse da segunda. dois
`window` não existiam: o segundo apagava o primeiro. `WindowId::dummy()`
é `WindowId(0)` no macOS (`window.rs:77`) e `From<u64>` existe: os testes
usam `dummy()` e `from(2)`.

### o host e o `Windows`

`crates/engine/src/window/host.rs` (551 linhas de produção).
`WindowHost` agrupa o que a sonda (a) disse ser da janela: `window`,
`gpu: Option<Gpu>` (o contexto e, construídos nele, `TextSystem`,
`EffectProcessor` e `TexturePool`), `target` (a textura de um host
headless), `compositor`, `input`, `touch` e `touch_pointer`, `ime`,
`safe_area`, `scale_factor`, `size` (físico, para o host saber o tamanho
lógico mesmo sem device), `clock` e `tick`, `perf` e `perf_hud`,
`events` (o buffer até o próximo frame), `animating`, `redraw_asked`,
`deadline`, `frames`, `a11y`. constrói por etapas para caber em todo
alvo: `bare()` (sem janela e sem device: cena e input que nunca
desenham, para teste sem GPU), `attach_window`, `attach_gpu`; `open` e
`open_with_config` fazem as três no nativo bloqueando no adapter como os
construtores sempre fizeram; no wasm o shell faz o spawn e prende o
resultado. `headless(w, h, scale, config)` é um host sem janela
desenhando numa textura, `read()` devolve os pixels
(`headless::read_texture`).

o que ele faz: `resize` (reconfigura a surface, reaplica a projeção
lógica, invalida, pede frame), `scale_factor_changed`, `handle_event`
(em `events.rs`: `Resized` e `ScaleFactorChanged` valem na hora; pointer,
teclado, roda, modificadores, touch, IME e drop são bufferizados e
invalidam; `CloseRequested` e `RedrawRequested` não são dele),
`process_events` (o buffer entra no `InputState`, o touch tica),
`request_redraw` (a janela, ou só a flag num host sem janela),
`needs_redraw` (animando, ou pedido, ou compositor sujo, ou HUD),
`set_animating`, `set_deadline` e `deadline` (o `Wake` por janela: o
instante em que esta janela precisa de frame sozinha), `begin_frame`
(tica o relógio, consome o pedido, limpa o texto do IME), `present`
(o `present_frame` de todo app para uma janela; `draw_frame` no target
para um host headless, com o poll e a recuperação de device antes),
`render` (em `render.rs`: a cauda própria do `App` do engine, com efeitos
de layer e HUD, para surface ou target), `device_lost` e
`recover_or_skip` (por contexto, da fase 1), `suspend` e `resume`
(android), `purge_caches`, `set_accessibility`.

`crates/engine/src/window/windows.rs` (311). `Windows` guarda
`Vec<(WindowId, WindowHost)>` na ordem de abertura (`first()` é a
principal), mais os hosts suspensos. `open(event_loop, attrs)` cria a
janela e o host (no nativo com o contexto; no wasm sem, até
`attach_gpu(id, gpu)`); `insert(id, host)` para um host que o chamador
construiu (um headless num teste). `route(id, event) -> Routed`: o host
dono toma input, IME, drop, resize e escala (`Handled`); `RedrawRequested`
processa o buffer e responde `Redraw` (o chamador constrói a cena e
apresenta); `CloseRequested` responde `CloseRequested` (só o chamador
sabe se fecha); id desconhecido responde `Unknown` (o `Destroyed` do
winit depois de um `close` cai aqui). `close(id)` tira o host do mapa e
devolve; soltar o host solta a janela e o contexto. `should_exit()`:
abriu alguma, não sobrou nenhuma e `exit_when_last_closes` (padrão
ligado; desligado, o loop segue sem janela). `request_pending_redraws`
é o render on demand por janela: cada host com motivo pede o próprio
frame, os outros ficam parados. `earliest_deadline` para o `WaitUntil`
do loop. `invalidate_all`, `set_animating`, `purge_caches`, `suspend` e
`resume(event_loop, attrs)` (android: um host guarda o device, ganha
janela nova e id novo).

a segunda janela reaproveita o `Instance`? não: cada `GpuContext` nasce
com o próprio `wgpu::Instance` (é o que `GpuContext::new_with_config` e
`new_headless` fazem, e o `recover` da fase 1 pergunta ao instance do
próprio contexto). reaproveitar pediria um construtor novo em `gpu/**`,
proibido aqui; o custo é uma enumeração de adapters por janela aberta,
o que todo construtor já pagava uma vez. fica registrado, não parei.

### o `App` do engine e os exemplos

`crates/engine/src/window/mod.rs` (159): o `App` fica com `windows:
Windows`, `theme`, `frame_read` e `frame_write`, `lifecycle`,
`is_animating`, o proxy e o `narrate_watcher`. `GpuState` e `state.rs`
saíram (o host tem o device). `AppEvent::GpuReady { window, gpu }`
nomeia a janela e leva só o contexto (o host constrói o resto).
`lifecycle.rs` (225): `resumed` abre a janela por `Windows::open`,
prende o `Bridge` (a janela nasce oculta e aparece depois, como antes),
no wasm faz o spawn do device; `suspended` é `windows.suspend()`;
`resumed` com suspensos é `windows.resume`; `handle_window_event` é um
`match` sobre `route`: `Redraw` desenha aquela janela (`render_window`:
o sinal de frame avança, `host.render(&theme)`, e o próximo frame só se
`needs_redraw`); `CloseRequested` fecha aquela e sai do app quando
`should_exit`. `about_to_wait` é `request_pending_redraws` (mais o
narrate reload, que invalida todas). `render_passes.rs` só trocou
`impl super::App` por `impl super::host::WindowHost` em
`apply_layer_effects` (613 linhas, não cresceu).

`crates/engine/examples/layers` (`main.rs` 123, `render.rs` 143): abre
duas janelas em `resumed`. a primeira é a demo de layers de antes (fundo
de pontos, chrome, o card do foreground que muda a cada 60 frames), com
`set_animating(true)`; a segunda é um card parado ("the other window")
que mostra quantos frames desenhou. `window_event` roteia por
`Windows::route`: `Redraw` desenha a janela pedida (`render_layers` ou
`render_side`, cada um constrói no compositor do seu host e chama
`host.present`, o `present_frame` de todo app, no lugar dos passes que
o exemplo codificava à mão), e pede o próximo só se `needs_redraw`;
`CloseRequested` fecha aquela e sai no `should_exit`. os `LayerId` da
demo são criados no compositor do host principal em `resumed` (um id
nomeia uma layer de um compositor). não rodei a janela de verdade:
o exemplo não tem `PLEV_BACKGROUND` (isso é do showcase) e abrir uma
janela que toma o foco está proibido aqui. o que prova o caminho são os
testes headless do `Windows`; o `create_window` duas vezes no mesmo
loop é o uso comum do winit. `cargo run -p engine --example layers` é
do orquestrador.

### o showcase

o shell (`crates/showcase/src/app.rs` 546, `app/events.rs` 195,
`app/detached.rs` 96, `app/pace.rs` 158) dirige as janelas por
`Windows`: `main: Option<WindowId>` e `page_window:
Option<detached::PageWindow>` (desktop). `resumed` abre a principal por
`Windows::open` (nativo com o device; wasm faz o spawn e `attach_gpu` no
`GpuReady`), prende o bridge e `viewport_changed(id)` faz o
`view.resize` com o tamanho lógico do host. o shell não roteia o
pointer pelo `Windows::route`: continua entregando `WidgetEvent` à view
na hora, como antes (bufferizar no `InputState` do host pediria frame a
cada movimento de mouse, e o `InputState` não é o caminho do showcase);
ele pede ao host o resize, a escala e o frame. cada evento sabe a
janela: o cursor é por janela (`cursor_in`), o pointer vai para
`view.detached_event` na janela da página e para `view.handle_event` na
principal; hover e scroll invalidam só a janela onde aconteceram
(`invalidate_in`), um clique que mudou algo invalida todas (`changed`),
porque a mudança aparece nas duas. teclas vão para a view seja qual for
a janela com foco.

o destacar: `D` fora de campo de texto (`view.handle_key` dá
precedência ao campo focado, como o `t` do tema) marca
`panes.detach_asked`; o shell lê com `take_detach_request` e
`toggle_page_window`: abre uma janela de 900x640 ("plev, <página>"),
`view.detach(lw, lh, sf)`; página placeholder não tem o que mostrar e
não abre. a janela principal continua com o chrome e a página; a segunda
mostra a página sozinha, sobre o mesmo estado e o mesmo dispatcher:
`view::dispatch::Detached` (`view/dispatch/detached.rs`, 226) guarda o
tamanho, a escala, as `Layers` criadas no compositor daquela janela no
primeiro frame, o scroll daquela janela e o `Wake` da página;
`render_detached(c)` faz o layout num viewport com gutter, clipa e chama
o mesmo `pane_render`; `detached_event` chama o mesmo `pane_event` com o
rect daquela janela e depois o scroll da página dela; `page_wake` passou
a guardar uma cópia do ask da página em `Detached::wake`
(`detached_wake` entrega ao shell), e a principal continua recebendo o
dela. escolhi espelhar, não mover: o aceite pede "um clique numa reflete
na outra no próximo frame", o que só existe com a página nas duas.
Escape na janela da página, o botão de fechar dela ou `D` de novo fecham
só ela (`close_page_window`: `view.attach()`, `windows.close`,
invalida); fechar a principal sai do app, como antes. um clique em
qualquer das duas passa pelo mesmo `Request::observe` por
`Actor::Human` no dispatcher da galeria.

a projeção: enquanto a janela da página existe, `ShowcaseView::semantics`
acrescenta um nó `window` (`Role::Window`, label "plev, <página>",
estado `page` com o título e `shows` com a lista dos ids raiz da página
mostrados nela) ao lado dos nós da página, que ficam com os mesmos ids,
árvore, estado e ações. escolhi um nó de janela e não um estado no nó da
página: as páginas (`apps/**`) são proibidas, e um nó só existe quando há
segunda janela, então com uma janela a projeção é a da página, byte a
byte (o golden do todo e os `semantics.json` das 161 baselines não
mudam). o nó entra reconstruindo o snapshot por `SnapshotBuilder`
(`with_window`): o `sem` não tem como levar um snapshot pronto para
dentro de outro (`Snapshot::from_parts` é `pub(crate)`). a reconstrução
mantém tudo menos o `source` de cada nó (o builder grava o call site,
que passa a ser o `detached.rs`); ver limitações. a porta do agente lê
os bounds do compositor da janela da página enquanto ela existe
(`page_window_id`). a capacidade `shell.detach` no contrato não entrou:
uma ação pelo dispatcher pede um `Operable` do shell no dispatcher da
galeria (emprestado por `TodoPage::with_dispatcher`) e um nó com ação,
o que é a forma que o item 4 (menus) vai precisar de todo jeito para o
"destacar" virar item de menu; fica lá.

a gravação: só os frames da janela principal (`record_frame` usa o
compositor dela); um clique na janela da página é gravado com o
controle sob ele naquele compositor, com a posição nas coordenadas
daquela janela (registrado como limite). o bridge de acessibilidade fica
na principal; a janela da página não tem (limite). o `regress` e o
`--render` não mudam: uma janela, `stage_page` sem host.

teste (`crates/showcase/tests/detached.rs`, dois `WindowHost::headless`,
1200x800 e 900x640, um `ShowcaseView` na Todo): com uma janela não há nó
`window`; a página se liga nos dois compositores com dois layouts
(`binding(todo.item[1])` em cada, rects diferentes) e a principal
continua com a página; o nó `window` tem role, label, `page` e `shows`
certos, e todo nó da página é igual ao de antes; um clique na janela da
página (o checkbox do item 1) alterna `completed` e o frame seguinte da
principal mostra o item mudado (pixels do rect antes e depois), e o da
janela da página também; um clique na principal desfaz e o frame da
janela da página volta ao de antes; `attach()` devolve a projeção com a
mesma revision de antes. e: a Card numa janela de 400x300 rola pela roda
sem mexer no scroll da principal; uma página placeholder não destaca.
os testes do shell (`source.rs`, `pace/tests.rs`) usam
`App::with_bare_window` (um `WindowHost::bare()` sob `WindowId::dummy()`)
e continuam sem GPU.

### gate e regress

`df -h /` antes do gate: 39 GB livres. `./script/gate` verde no worktree, com a
perna wasm e a perna urnaui (`../urna` presente): antes `cargo test
--workspace` 2485 passed, 0 failed, 16 ignored em 115 binários; depois
2494 passed, 0 failed, 16 ignored em 117 binários (+9: 7 de
`engine/tests/windows.rs`, 2 de `showcase/tests/detached.rs`; +2
binários, os dois arquivos de teste novos). perna urnaui: 85 passed, 0
failed, 1 ignored em 6 binários, igual a antes (o ide e o urnaui não
usam o `App` do engine e compilam sem mudança). clippy `-D warnings`,
fmt e a checagem wasm limpos nas duas pernas. 33 GB livres depois.

`cargo run -p showcase --example regress -- check`: 161 baselines, 161 iguais,
0 mudaram, sem frame desenhado e sem GPU aberta (788.9 ms). o golden do
todo (`uir_fixture`) e os testes do `regress` também verdes: com uma
janela a projeção não mudou.

linhas de produção (regra do guard): engine `window/host.rs` 551,
`window/windows.rs` 311, `window/events.rs` 194, `window/render.rs`
244, `window/lifecycle.rs` 225, `window/mod.rs` 159,
`window/render_passes.rs` 613 (era 633: só trocou o alvo do `impl`),
`examples/layers/main.rs` 123, `examples/layers/render.rs` 143; showcase
`app.rs` 546 (era 620; foi a 715 e o `window_event` saiu para
`app/events.rs`), `app/events.rs` 195, `app/detached.rs` 96,
`app/pace.rs` 158, `app/record.rs` 316, `app/source.rs` 67,
`view/dispatch.rs` 452 (era 415; foi a 665 e o destacar saiu para
`view/dispatch/detached.rs`), `view/dispatch/detached.rs` 226,
`view/input.rs` 400, `view/panes.rs` 134. tudo abaixo de 639.

### ADR

`docs/ADR/windows-are-hosts-the-app-owns-the-state.md` (proposed): o que
é da janela e o que é do app, o host por etapas, o `Windows` e o
roteamento, render on demand por janela, o fechar, o destacar do
showcase, o que a projeção diz, os limites. linha pro índice, que é do
orquestrador (`docs/ADR/README.md` é proibido aqui):

| 2026-09-24 | [windows-are-hosts-the-app-owns-the-state](windows-are-hosts-the-app-owns-the-state.md) | proposed | `engine::window::WindowHost` agrupa o que morre com a janela (a janela, o `GpuContext` com a surface e a perda de device, compositor, `InputState`, touch, IME, escala, área segura, relógio, perf, eventos bufferizados) e `Windows` guarda vários por `WindowId`: `open`, `route` (o dono toma input e resize; close e redraw voltam ao chamador), `close` de uma sem o app, `should_exit` na última (configurável), redraw por janela (parada não desenha enquanto outra anima), deadline mais cedo; o `App` do engine com uma janela sem mudança, o `layers` com duas; o showcase destaca a página atual numa segunda janela sobre o mesmo estado e o mesmo dispatcher, a projeção ganha um nó `window` só enquanto ela existe; provado headless com hosts sob `WindowId::dummy()` (task-76 item 2) |

### interfaces adicionadas (só crescimento, salvo o `AppEvent`)

`engine::window::{WindowHost, Windows, Routed}`; `WindowHost::{bare,
new, open, open_with_config, headless, attach_window, attach_gpu,
window, id, has_gpu, gpu, gpu_mut, text_system, compositor,
compositor_mut, input, input_mut, ime, safe_area, scale_factor,
physical_size, logical_size, tick, frames, resize,
scale_factor_changed, handle_event, process_events, pending_events,
request_redraw, redraw_asked, needs_redraw, set_animating,
is_animating, set_deadline, deadline, begin_frame, present,
device_lost, read, suspend, resume, purge_caches, set_accessibility,
accessibility_mut}`; `Windows::{new, len, is_empty, ids, first,
contains, get, get_mut, iter, iter_mut, insert, open, open_with_config,
attach_gpu, route, close, should_exit, set_exit_when_last_closes,
exit_when_last_closes, request_redraw, request_pending_redraws,
invalidate_all, set_animating, earliest_deadline, purge_caches, suspend,
has_suspended, resume}`; `App::{windows, windows_mut}`.
`ShowcaseView::{is_detached, take_detach_request, detach, attach,
resize_detached, render_detached, detached_event, detached_wake}`,
`view::dispatch::Detached`. o que mudou: `engine::window::AppEvent::GpuReady`
passou de `{ gpu, text_system, effect_processor, texture_pool }` para
`{ window: WindowId, gpu }` (o único construtor era o próprio engine);
`App::{set_animating, is_animating}` mantidos. `present_frame`,
`draw_frame`, `resolve_layer_text`, `encode_*` sem mudança: ide, urnaui,
o headless e os outros exemplos compilam sem tocar.

### arquivos tocados fora da lista permitida, e por quê

- `crates/showcase/src/view/dispatch/detached.rs` (novo): o destacar
  levou `view/dispatch.rs` a 665 linhas de produção; o módulo filho é a
  forma que `dispatch/agent.rs` já tem, e acessa os métodos privados
  (`pane_render`, `pane_event`) por ser descendente.
- `crates/showcase/src/app/events.rs` (novo, previsto pelo briefing para
  `app.rs` passar de 639): o `window_event` do shell.
- `crates/engine/src/window/state.rs` removido (o `GpuState` virou o
  `Option<Gpu>` do host).

### limitações e decisões, sem esconder

- a janela de verdade não foi aberta nesta máquina: nem o `layers` com
  duas nem o showcase com `D`. o exemplo não tem `PLEV_BACKGROUND` e
  ninguém pode apertar `D` por mim (tecla sintética proibida). a prova é
  headless: o `Windows` com hosts sob ids falsos e a view com dois
  hosts. o caminho do winit (`create_window` duas vezes, eventos por id)
  é o comum e está lido na fonte, não medido.
- `source` dos nós enquanto a janela da página existe: a reconstrução
  por `SnapshotBuilder` põe o call site do `detached.rs` em todo nó, então
  o cmd+click e o `state` da gravação apontam para a view nesse
  intervalo. a forma certa é um `SnapshotBuilder::extend(&Snapshot)` (ou
  um `Snapshot::with_node`) no `sem`, que guardaria o `source`; é mudança
  de contrato e fica para decisão do orquestrador. com uma janela nada
  disso acontece.
- a janela da página não tem bridge de acessibilidade; a árvore de um
  leitor de tela é a da principal, que continua com a página. a
  gravação guarda só os frames da principal; os cliques na janela da
  página entram com o alvo certo (o compositor dela) e a posição nas
  coordenadas dela. o clique direito é só da principal (o menu da Git e
  a demo de Context Menu fazem layout lá). os overlays que uma página
  abre (modal, toast) desenham na principal, onde estão as layers de
  overlay. um hover é estado compartilhado dos widgets: aparece na outra
  janela no frame seguinte dela, não na hora.
- a janela da página segue a página ativa: trocar de página na
  principal troca o que a segunda mostra; o título dela fica o da página
  que a abriu.
- wasm e mobile: uma janela. `Windows::open` no wasm abre sem device
  (o shell faz o spawn), `D` loga e não abre. o `resume` do android
  passou pelo `Windows` sem ser medido num aparelho.
- `crates/showcase/src/renderer.rs` (`render_frame`) ficou sem chamador
  no shell (o `pace.rs` usa `host.present`, o mesmo `present_frame`);
  está fora da lista permitida e ficou.
- o showcase não passa o pointer por `Windows::route`; o `App` do engine
  e o `layers` passam tudo. dois caminhos de entrada, como já eram
  (`arc.toml` já anota que os shells entregam o pointer direto à view).
- `is_animating` do `App` do engine é um pedido do app aplicado a todas
  as janelas; por janela é `WindowHost::set_animating`.

### o que os itens 4, 7 e 8 precisam disto

- item 4 (menus nativos): no macOS a barra de menu é do app e o menu de
  contexto é da janela; o host é onde um item de menu por janela se
  liga, e o `route` é onde um evento de menu de uma janela chega ao
  dono. o "destacar" do showcase como item de menu é a capacidade que
  não entrou aqui: um `Operable` do shell no dispatcher da galeria e um
  nó com a ação, pelo mesmo `Actor::Human`.
- item 7 (sleep e wake): pausar é por host (`set_animating(false)`, o
  `deadline` limpo, `request_pending_redraws` não pede nada); acordar é
  `invalidate_all` e um frame por janela, e a recuperação de device já é
  por contexto (fase 1). o "sem frame velho" da fase 1 vale por janela.
- item 8 (diálogos com janela pai): um diálogo é um `Windows::open` com
  atributos de filho (`with_parent_window` do winit no macOS) e o id do
  pai guardado pelo shell; `set_exit_when_last_closes` já permite fechar
  o diálogo sem fechar o app, e `route` entrega ao diálogo o que é dele.
  a confirmação humana da task-58 como janela cabe aqui.

### comandos de validação

```text
export CARGO_TARGET_DIR=/Users/nn/Dev/hoff/plevdev/target/agents/one
export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_PROFILE_TEST_DEBUG=line-tables-only
cargo test --offline -p engine --test windows
cargo test --offline -p showcase --test detached
cargo test --offline -p engine -p showcase
./script/gate
cargo run --offline -p showcase --example regress -- check
cargo run --offline -p engine --example layers          (duas janelas; do orquestrador)
cargo run --offline -p showcase -- Todo                  (D destaca a página; do orquestrador)
```
