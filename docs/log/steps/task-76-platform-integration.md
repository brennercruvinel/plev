---
project: plev
audience: [ai-agents, contributors]
status: done
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

- clipboard: `rust/engine/src/runtime/platform/clipboard.rs` (arboard no desktop,
  `LocalClipboard` em teste, wasm e mobile).
- diálogo de arquivo nativo (rfd) e drop de arquivo: só no urnaui
  (`apps/urnadbapp/src/app.rs`, `apps/urnadbapp/src/view/explorer/mod.rs`).
- IME básico: `rust/engine/src/runtime/platform/ime.rs`.
- lifecycle (estados de app, aviso de memória):
  `rust/engine/src/runtime/platform/lifecycle.rs`; safe areas em
  `rust/engine/src/runtime/platform/mod.rs`.
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

`rust/engine/src/runtime/platform/**`, `rust/engine/src/runtime/window` na parte de
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

`rust/engine/src/drawing/gpu/loss.rs`. `GpuContext::from_device` (o corpo em que
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

`rust/engine/src/drawing/gpu/recover.rs`. `GpuContext::recover` (bloqueante,
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

`rust/engine/tests/device_loss.rs`, headless 240x140 a escala 2
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

`./tool/tasks/fullcheck.sh` verde no worktree (com a perna wasm e a perna urnaui, que
achou `../urna`): antes `cargo test --workspace` 2454 passed, 0 failed,
16 ignored em 114 binários; depois 2465 passed, 0 failed, 16 ignored em
115 binários (+11: os testes novos). clippy `-D warnings` e fmt limpos.
`cargo run -p regress -- check`: 159 baselines, 159
iguais, 0 mudaram, sem frame desenhado. contagem de linhas:
`gpu/context.rs` 507 de produção (era 461), `gpu/recover.rs` 231,
`gpu/loss.rs` 118, tudo abaixo de 639.

### ADR

`docs/adr/rendering/0054-gpu-device-loss-recovers-in-place.md` (proposed): o sinal, a
ordem de recriação, o que sobrevive e o que não, o que o app faz (nada; a
exceção é recurso próprio no device). linha pro índice, que é do
orquestrador (`docs/adr/README.md` é proibido aqui):

| 2026-09-24 | [gpu-device-loss-recovers-in-place](../../adr/rendering/0054-gpu-device-loss-recovers-in-place.md) | proposed | `GpuContext` instala o callback de device lost em todo device e guarda o flag; `recover` refaz adapter, device, surface config, pipelines e atlas pelo mesmo `from_device`; o frame recupera antes do acquire ou pula com um erro por perda; o store de imagens vira log de uploads com os pixels; provado headless com `Device::destroy` e pixels iguais antes e depois (task-76 fase 1) |

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
inline em `rust/engine/Cargo.toml`, na seção do `arboard`, lock com
`--offline` (opção B); o shell do showcase ganha um braço só (opção 2);
`open_file` com `Actors { human, assistive, test: true; agent: false }`;
posição opcional, junção no `App` do engine, trait síncrono por caminho,
`web_picker.rs` intocado.

### a sonda

(a) o que o urnaui faz hoje com `rfd` e com os eventos de drop, por
linha:

- `apps/urnadbapp/Cargo.toml:45`: `rfd = "0.15"`, features padrão
  (`xdg-portal` e `async-std`), na seção
  `[target.'cfg(not(target_arch = "wasm32"))']`. o `rfd` 0.15.4 só tem
  backend para linux/bsd, macos, windows e wasm (`backend.rs:7-45`): em
  android e ios `FileDialog::pick_file` não compila (nenhum
  `FilePickerDialogImpl`). o urnaui só chama `rfd` sob
  `not(any(wasm32, android, ios))`, mas declara a dependência para todo
  não-wasm; no engine ela entra na seção do `arboard`, que já exclui
  android e ios.
- `apps/urnadbapp/src/view/explorer/mod.rs:400-407`, `Action::PickFile` no
  desktop: `rfd::FileDialog::new().set_title("Open a .urna corpus")
  .add_filter("urna corpus", &["urna", "nest"]).pick_file()`; `Some(path)`
  vira `open_database(path)`, `None` (cancelado) não faz nada. síncrono,
  modal, sem parent, sem diretório inicial.
- `apps/urnadbapp/src/view/explorer/mod.rs:445-476`, `export_blob` no desktop:
  `rfd::FileDialog::new().set_title("Export media blob")
  .set_file_name(&name).save_file()`; `Some(dest)` vira
  `UrnaCommand::ExportBlob { index, dest }` e um toast; `None` devolve
  `true` (tratado, nada a fazer). `explorer.rs:478-486`: em wasm, android e
  ios a exportação só toasta "blob export needs the desktop app".
- `apps/urnadbapp/src/view/explorer/mod.rs:393-399` e `app.rs:482-488`: em
  wasm `PickFile` marca `pick_requested`; o shell dispara
  `web_picker::trigger` no `about_to_wait`, e o `<input type=file>`
  devolve `UserEvent::FileLoaded { name, bytes }` (bytes, nunca caminho),
  que vira `view.file_picked`. é a entrada de arquivo do alvo web e não
  passa por `rfd`.
- `apps/urnadbapp/src/app.rs:362-377` (só não-wasm):
  `WindowEvent::DroppedFile(path)` vira `view.file_dropped(path)` mais
  invalidate; `HoveredFile(_)` vira `view.set_file_hover(true)`, o caminho
  do arquivo pairando é descartado; `HoveredFileCancelled` vira
  `set_file_hover(false)`.
- `apps/urnadbapp/src/view/mod.rs:217-233`: `file_dropped(path)` desliga o
  hover, chama `explorer.open_database(path)` e toasta "opening dropped
  file…"; `set_file_hover(bool)` repassa ao explorer.
- `apps/urnadbapp/src/view/explorer/mod.rs:67,121,186-190,869` e
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
  (`rust/engine/Cargo.toml:97-102`), então o `rfd` compilaria para wasm
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

(c) o molde do `clipboard.rs` (`rust/engine/src/runtime/platform/clipboard.rs`):

- `ClipboardProvider` (linhas 9-14): o trait, compilado em todo alvo.
- `LocalClipboard` (16-36): a implementação em memória, compilada em todo
  alvo; é o falso dos testes e o fallback de wasm e mobile.
- `SystemClipboard` (40-75): `arboard`, sob
  `#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]`;
  `new()` degrada para `None` quando o `arboard` falha (CI headless) e as
  operações viram no-op com um `log::warn!`.
- `default_clipboard() -> Box<dyn ClipboardProvider>` (78-89): o `cfg`
  escolhe `SystemClipboard` no desktop e `LocalClipboard` no resto.
- `rust/engine/Cargo.toml:82-85`: `arboard` só na seção
  `cfg(all(not(wasm32), not(android), not(ios)))`.
- por teste: quem consome guarda `Box<dyn ClipboardProvider>` ou o tipo
  concreto; teste usa `LocalClipboard`. o urnaui constrói
  `SystemClipboard::new()` direto (`explorer.rs:80,124`, `app.rs:333`).
- o módulo de diálogos segue o mesmo desenho: `FileDialogs` (trait),
  `FakeDialogs` (em todo alvo; responde o que o teste carregou e registra
  o que foi pedido), `NativeDialogs` (`rfd`, sob o cfg do `arboard`),
  `NoDialogs` (wasm e mobile: `Unsupported`), `default_dialogs()`.

o que entra de dependência (e o motivo da parada): o `Cargo.lock` da raiz
não tem `rfd`. o urnaui é workspace próprio (`apps/urnadbapp/Cargo.lock`
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

o shell do showcase: `apps/uigallery/src/shell/mod.rs:341-478` não trata
`DroppedFile`, `HoveredFile` nem `HoveredFileCancelled`, e o arquivo é
proibido nesta task. a página Files recebe o drop pelos testes, pelo
headless e pelo dispatcher; para o drop chegar numa janela de verdade o
shell precisa de um braço a mais no `match`, que repassa o evento do
engine a `ShowcaseView::handle_file_drop`.

### o trait, o falso e o que cada alvo faz

`rust/engine/src/runtime/platform/files.rs` (350 linhas de produção).
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

`rust/engine/Cargo.toml`: `rfd = { version = "0.15", default-features =
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

`rust/engine/src/control/input/drop.rs` (110 de produção): `FileDrop::{Hovered
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
mexem). `demo/inputs/src/state.rs` ganhou o braço
`InputEvent::FileDrop` que a variante nova exige (único match exaustivo
no repositório).

### o que mudou no urnaui

- `apps/urnadbapp/Cargo.toml`: `rfd` sai (o comentário aponta pro engine).
  `apps/urnadbapp/Cargo.lock`: 370 linhas a menos, 35 pacotes fora
  (`ashpd`, `url`, `icu_*`, `zbus`... a árvore de linux que o `rfd` com
  features padrão trazia); `rfd` continua no lock pelo engine, sem
  features.
- `apps/urnadbapp/src/view/explorer/mod.rs`: campo `dialogs: Box<dyn
  FileDialogs>` (`default_dialogs()`); `set_dialogs`, `opening` e
  `file_hover` sob `cfg(test)`; `Action::PickFile` chama `pick_file` fora
  do wasm (no wasm continua marcando `pick_requested` pro `web_picker`);
  `set_file_hover` perdeu o cfg (compila em todo alvo, a web nunca manda).
  992 linhas de produção (era 1019, congelado pelo guard de higiene).
- `apps/urnadbapp/src/view/explorer/dialogs.rs` (novo, 85 linhas):
  `pick_file` (título "Open a .urna corpus", filtro "urna corpus" com
  `urna` e `nest`; `Ok(Some)` abre, `Ok(None)` nada, `Err` toasta "the
  file picker needs the desktop app") e `export_blob`, uma versão só
  pra todo alvo (título "Export media blob", `file_name` do blob; `Err`
  toasta "blob export needs the desktop app", o texto que o mobile e a
  web já mostravam). as duas variantes cfg'd de antes viraram uma.
- `apps/urnadbapp/src/view/mod.rs`: `handle_file_drop(&FileDrop) -> bool`
  no lugar de `file_dropped` e `set_file_hover`: `Hovered` liga o hover
  da tela Open, `Cancelled` desliga, `Dropped` desliga e abre cada caminho
  em ordem com o toast "opening dropped file…" (o último ganha, como
  antes). compila em todo alvo.
- `apps/urnadbapp/src/app.rs`: os três braços cfg'd de drop saíram; o
  braço `other =>` entrega todo evento não tratado a
  `engine::input::FileDrop::from_window_event` e, se é drop, a
  `view.handle_file_drop` mais `invalidate`. `app.rs` não nomeia
  `DroppedFile`, `HoveredFile` nem `HoveredFileCancelled`; o engine
  decide o que é drop.
- `apps/urnadbapp/src/web_picker.rs` não mudou: é a entrada de bytes da
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

`apps/uigallery/src/views/apps/files/mod.rs` (394 linhas), `files/semantics.rs`
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
`apps/uigallery/src/shell/mod.rs:476-485` (aprovado, opção 2): o braço
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

golden: `rust/plevir/tests/fixtures/uigallery-files.snapshot.json` (4 nós,
revision `d7a661b0bdabefd9`), mantido por
`files::tests::the_page_projection_is_the_uir_golden`; `UIR_BLESS=1
cargo test -p uigallery --lib files::tests::the_page_projection` regrava.
os testes do `uir` continuam verdes.

baselines: `tool/bench/regress/baselines/files@core@1280x800` e
`files@hoff@1280x800` (`semantics.json`, `scene.json`, `frame.png`; 4
layers, 4 controles bound, 4 nós, settled em 2 passos), abençoadas com
`cargo run -p regress -- bless --page Files --theme
core --png` e o mesmo com `hoff`.

render headless: `cargo run -p uigallery -- --render Files --theme core
--out files-core.png --semantics files-core.json` e o mesmo com `hoff`
deram dois PNG 1200x800 (o card centrado na largura legível, a zona com
"drop files here", o botão "open file" com o ícone de pasta, o status
"no files yet: drop some on the window, or open one" ao lado); o
`frame.png` de cada baseline é a mesma cena a 1280x800 e fica na árvore.
a projeção do render é o golden byte a byte (só o `\n` final que o teste
acrescenta).

### gate e regress

`df -h /` antes de cada gate: 15 GB livres no primeiro, 12 GB no
segundo. o primeiro `./tool/tasks/fullcheck.sh` ficou vermelho no
`file_hygiene_guard`: `apps/urnadbapp/src/view/explorer/mod.rs` está
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

`cargo run -p regress -- check`: 161 baselines (eram
159), 161 iguais, 0 mudaram, sem frame desenhado.

linhas de produção (regra do guard): engine `platform/files.rs` 350,
`input/drop.rs` 110, `input/handlers.rs` 165, `input/types.rs` 256,
`window/events.rs` 227; showcase `apps/files.rs` 394,
`files/semantics.rs` 289, `view/dispatch.rs` 415, `view/input.rs` 394;
urnaui `view/explorer.rs` 992 (era 1019, congelado, encolheu),
`view/explorer/dialogs.rs` 85, `app.rs` 562. tudo abaixo de 639, salvo o
congelado.

### ADR

`docs/adr/interface/0055-file-dialogs-and-drops-are-capabilities.md` (proposed): o
trait e o falso, o `rfd` sem features no desktop, o evento de drop com
posição opcional e a junção, o drop como request, a regra "diálogo
nativo só abre por pessoa" na policy, o que o wasm faz, o que os itens 4
e 8 reaproveitam. linha pro índice, que é do orquestrador
(`docs/adr/README.md` é proibido aqui):

| 2026-09-24 | [file-dialogs-and-drops-are-capabilities](../../adr/interface/0055-file-dialogs-and-drops-are-capabilities.md) | proposed | `engine::platform::files` (trait síncrono por caminho, `rfd` no desktop sem features padrão, `FakeDialogs` pra teste, `NoDialogs` em wasm e mobile) e `engine::input::FileDrop` (uma tradução dos três eventos do winit, posição opcional, junção dos N por arquivo no `App`); o urnaui usa os dois sem segunda implementação; a página Files do showcase projeta `open_file` só pra pessoa e `drop_files` pra todo ator, pelo dispatcher; diálogo nativo só abre por pessoa (task-76 item 3) |

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

- `demo/inputs/src/state.rs`: o braço `InputEvent::FileDrop`
  (a variante nova quebrava o match exaustivo; clippy `--all-targets`
  compila o exemplo).
- `apps/urnadbapp/src/view/mod.rs`: `file_dropped` e `set_file_hover`
  viraram `handle_file_drop` (a tradução do drop do urnaui morava aí
  também, não só em `app.rs`); um teste novo.
- `apps/urnadbapp/src/view/explorer/dialogs.rs` (novo): o guard de higiene
  congela `explorer.rs`; sem o módulo filho o gate fica vermelho.
- `apps/uigallery/src/shell/mod.rs`: o braço aprovado (opção 2).
- `apps/urnadbapp/Cargo.lock`: consequência do `Cargo.toml`.

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
  teste de integração em `apps/uigallery/tests/` que construísse a view
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
cargo test --offline -p engine -p uigallery
CARGO_TARGET_DIR=$CARGO_TARGET_DIR/urnaui cargo test --offline --manifest-path apps/urnadbapp/Cargo.toml
./tool/tasks/fullcheck.sh
cargo run --offline -p regress -- check
cargo run --offline -p uigallery -- --render Files --theme core --out files.png --semantics files.json
UIR_BLESS=1 cargo test --offline -p uigallery --lib files::tests::the_page_projection   (regrava o golden)
cargo run --offline -p regress -- bless --page Files --theme core --png   (e --theme hoff)
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

- `App` do engine (`rust/engine/src/runtime/window/mod.rs:49`): da janela
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
- shell do showcase (`apps/uigallery/src/shell/mod.rs:82`): da janela
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
- o ide (`apps/idestudio/src/main.rs:106`) e o urnaui
  (`apps/urnadbapp/src/app.rs:75`) repetem a forma do showcase; fora do
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

(b) `rust/engine/tests/windows.rs`,
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

`rust/engine/src/runtime/window/host.rs` (551 linhas de produção).
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

`rust/engine/src/runtime/window/windows.rs` (311). `Windows` guarda
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

`rust/engine/src/runtime/window/mod.rs` (159): o `App` fica com `windows:
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

`demo/layers/src` (`main.rs` 123, `render.rs` 143): abre
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
loop é o uso comum do winit. `cargo run -p layers` é
do orquestrador.

### o showcase

o shell (`apps/uigallery/src/shell/mod.rs` 546, `app/events.rs` 195,
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

teste (`apps/uigallery/tests/detached.rs`, dois `WindowHost::headless`,
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

`df -h /` antes do gate: 39 GB livres. `./tool/tasks/fullcheck.sh` verde no worktree, com a
perna wasm e a perna urnaui (`../urna` presente): antes `cargo test
--workspace` 2485 passed, 0 failed, 16 ignored em 115 binários; depois
2494 passed, 0 failed, 16 ignored em 117 binários (+9: 7 de
`engine/tests/windows.rs`, 2 de `showcase/tests/detached.rs`; +2
binários, os dois arquivos de teste novos). perna urnaui: 85 passed, 0
failed, 1 ignored em 6 binários, igual a antes (o ide e o urnaui não
usam o `App` do engine e compilam sem mudança). clippy `-D warnings`,
fmt e a checagem wasm limpos nas duas pernas. 33 GB livres depois.

`cargo run -p regress -- check`: 161 baselines, 161 iguais,
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

`docs/adr/structure/0056-windows-are-hosts-the-app-owns-the-state.md` (proposed): o que
é da janela e o que é do app, o host por etapas, o `Windows` e o
roteamento, render on demand por janela, o fechar, o destacar do
showcase, o que a projeção diz, os limites. linha pro índice, que é do
orquestrador (`docs/adr/README.md` é proibido aqui):

| 2026-09-24 | [windows-are-hosts-the-app-owns-the-state](../../adr/structure/0056-windows-are-hosts-the-app-owns-the-state.md) | proposed | `engine::window::WindowHost` agrupa o que morre com a janela (a janela, o `GpuContext` com a surface e a perda de device, compositor, `InputState`, touch, IME, escala, área segura, relógio, perf, eventos bufferizados) e `Windows` guarda vários por `WindowId`: `open`, `route` (o dono toma input e resize; close e redraw voltam ao chamador), `close` de uma sem o app, `should_exit` na última (configurável), redraw por janela (parada não desenha enquanto outra anima), deadline mais cedo; o `App` do engine com uma janela sem mudança, o `layers` com duas; o showcase destaca a página atual numa segunda janela sobre o mesmo estado e o mesmo dispatcher, a projeção ganha um nó `window` só enquanto ela existe; provado headless com hosts sob `WindowId::dummy()` (task-76 item 2) |

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

- `apps/uigallery/src/views/dispatcher/detached.rs` (novo): o destacar
  levou `view/dispatch.rs` a 665 linhas de produção; o módulo filho é a
  forma que `dispatch/agent.rs` já tem, e acessa os métodos privados
  (`pane_render`, `pane_event`) por ser descendente.
- `apps/uigallery/src/shell/window/events/mod.rs` (novo, previsto pelo briefing para
  `app.rs` passar de 639): o `window_event` do shell.
- `rust/engine/src/runtime/window/state.rs` removido (o `GpuState` virou o
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
- `apps/uigallery/src/renderer.rs` (`render_frame`) ficou sem chamador
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
cargo test --offline -p uigallery --test detached
cargo test --offline -p engine -p uigallery
./tool/tasks/fullcheck.sh
cargo run --offline -p regress -- check
cargo run --offline -p layers          (duas janelas; do orquestrador)
cargo run --offline -p uigallery -- Todo                  (D destaca a página; do orquestrador)
```

## handoff do item 4: menus nativos como capacidades

branch `task/76d-menus`, base `8ce7225c` (main). commits, em ordem:

- `3338c38e` task-76: the native menu probe
- `da659be1` task-76: the menu model and the trait, with a fake
- `5c351c62` task-76: the muda menu on the desktop, and the poll
- `32794c25` task-76: the shell projected, and its keys through the
  dispatcher
- `79ecbaed` task-76: the showcase menu bar
- `af75dd50` task-76: the urnaui lock follows the engine's muda
- (este) task-76: ADR and the item 4 handoff

### a sonda

(a) o lock, com `muda = { version = "0.20", default-features = false }`
na seção de desktop não móvel de `rust/engine/Cargo.toml` (a do
`arboard` e do `rfd`), resolvido com `cargo metadata --offline`: "Locking
2 packages: Adding keyboard-types v0.8.3, Adding muda v0.20.0". o
`Cargo.lock` da raiz ganhou só esses dois pacotes; `objc2-app-kit 0.3.2`
ganhou a aresta `objc2-core-foundation` (feature). o resto do que o
`muda` pede já estava no lock: `crossbeam-channel 0.5.15`, `dpi 0.1.2`,
`objc2 0.6.4`, `objc2-core-foundation 0.3.2`, `objc2-foundation 0.3.2`,
`once_cell 1.21.4`, `png 0.18.1`, `thiserror 2.0.18` e, como entrada só
de windows, `windows-sys 0.61.2`. as features padrão (`gtk3`, `libxdo`)
ficaram desligadas: no linux o `muda` compila no backend `noop`
(`platform_impl/mod.rs`) até o bring-up escolher. `cargo tree --offline
-p engine -i muda` mostra só o engine como consumidor. nada faltou no
cache; não parei. o `apps/urnadbapp/Cargo.lock` (workspace próprio, que
depende do engine por caminho) ganhou `muda`, `keyboard-types` e um
`crossbeam-channel 0.5.17`, como ganhou `rfd` no item 3; a perna urnaui
do gate o resolveu offline.

(b) o código do `muda` 0.20 (`~/.cargo/registry/src/*/muda-0.20.0/src/`):

- o clique: cada `NSMenuItem` criado pelo `muda` tem como alvo um
  `NsMenuItem` (`platform_impl/macos/mod.rs:820-840`) cuja ação chama
  `MenuEvent::send(MenuEvent { id })`; para um `CheckMenuItem` ela
  inverte o `checked` antes de enviar. `MenuEvent::send`
  (`menu_event.rs`) entrega ao handler único do processo se
  `set_event_handler(Some(..))` foi chamado, senão ao canal global
  (`MENU_CHANNEL`, um `crossbeam_channel::unbounded`) que
  `MenuEvent::receiver()` lê; com handler, o canal não recebe nada. o
  handler é um `OnceCell`: uma vez por processo. o `id` é o `MenuId`
  (uma `String`) do item, o que o app passou em `MenuItem::with_id` ou um
  contador. a nota do próprio `muda` para winit: usar o handler e acordar
  o laço pelo `EventLoopProxy`.
- por app e por janela: no macOS `Menu::init_for_nsapp` faz
  `NSApplication::sharedApplication().setMainMenu(Some(menu))`
  (`macos/mod.rs:164-168`), a barra é do aplicativo, uma para todas as
  janelas; um `Menu` só aceita `Submenu` no topo. no windows
  `Menu::init_for_hwnd(hwnd)` prende a barra a uma janela. no gtk,
  `init_for_gtk_window`. `Menu::new` e `Submenu::new` exigem a thread
  principal (`MainThreadMarker::new().expect(..)`), então nenhum teste
  do cargo constrói um `Menu`.
- o laço do winit: o winit 0.30.13 instala a barra padrão dele (About,
  Services, Hide, Hide Others, Show All, Quit) em
  `applicationDidFinishLaunching` quando `default_menu` está ligado
  (`platform_impl/macos/app_state.rs:139-142`, `menu.rs`), logo antes de
  `dispatch_init_events`, que dispara o `resumed`. chamar
  `init_for_nsapp` no `resumed` substitui a barra padrão sem mexer no
  laço; `Menu::drop` só tira do `NSApp` a barra que ainda é a principal,
  então a nova entra antes da antiga sair. ide e urnaui não precisam
  mudar nada.
- uma janela real: um teste não prova o `setMainMenu` num `NSApp` vivo,
  então rodei o showcase uma vez com `PLEV_BACKGROUND=1` (`Todo`, 8 s,
  morto pelo pid exato, nenhum clique nem tecla): o log diz "menu: 100
  items shown" e "menu: 100 items in the bar" (14 temas, 81 páginas, o
  destacar, 4 ações da Todo), sem pânico. com a política de ativação
  proibida a barra não aparece, então o clique num item real fica pro
  orquestrador.

(c) da projeção do showcase, o que hoje podia virar item de menu (ação
sem parâmetro, permitida a `human`):

- `showcase-todo.snapshot.json`: `todo.input` `submit` (desabilitado com
  o campo vazio), `todo.filter[all|active|completed]` `select`,
  `todo.item[n]` `toggle` e `remove` (destrutivo). `set_value` fica de
  fora (param texto).
- `showcase-silences.snapshot.json`: `silences.threshold` e
  `silences.min_duration` `increment` e `decrement`, `silences.detect`
  `detect_silence`, `silences.cut[n]` `preview_segment`, `apply_cut`,
  `reject_cut`, `silences.apply_all` `apply_all`. `set_value` fora. a
  projeção viva da página (`projection()`) traz ainda a linha de
  propor (`proposer: open/close`, `propose`, `cancel`, `close the
  surface`), que o golden não tem.
- `showcase-files.snapshot.json`: `files.open` `open_file` (só pessoa);
  `drop_files` fora (param lista).
- atalhos do shell só como tecla: `T` (cicla core, hoff, dark, light),
  os dígitos `1` a `9` e `0` (página pelo índice), `D` (a página numa
  janela própria, `panes.detach_asked`), `Esc` (fecha overlay, fecha a
  janela da página, ou sai) e `cmd-shift-s` (salva a gravação, só com
  `PLEV_RECORD`). o clique na sidebar e nos cards da página Theme
  também mudavam página e tema direto (`set_page`, `set_theme`).

### o modelo e o trait

`rust/engine/src/runtime/platform/menu/mod.rs` (606 linhas de produção) e
`platform/menu/native.rs` (317). a mesma forma do `files.rs`: trait,
real sob o cfg do `arboard`, falso em todo alvo, nenhum em wasm e
mobile, `default_menu(wake)` escolhendo.

- `MenuModel { menus: Vec<Submenu> }`; `Submenu { label, entries }`;
  `Entry::{Item(MenuItem), Separator, Predefined(Predefined),
  Submenu(Submenu)}`; `Predefined::{About, Hide, HideOthers, ShowAll,
  Quit}` (os itens da plataforma, em palavras dela; nunca capacidade).
- `MenuItem { sem: SemId, action: ActionName, param: Option<Value>,
  label, shortcut: Option<Shortcut>, enabled, checked: Option<bool> }`.
  `MenuItem::of(snapshot, sem, action)` lê uma ação de um nó (label da
  ação ou do nó; `enabled` quando nó e ação estão e o nó não é inert; a
  marca do `on` de um Toggle ou do `checked` de um Checkbox);
  `MenuItem::options(snapshot, sem, action)` um item por opção de
  `constraints.options` (param `Text(opção)`, marca na `selected`);
  `MenuItem::plain(snapshot)` toda ação sem parâmetro que uma pessoa pode
  pedir (`policy.actors.human`), em ordem de projeção. `key()` é o texto
  `sem#action[#param]` que nomeia o item no `muda`. o app decide o que
  entra e em que submenu; a plataforma nunca.
- `Shortcut { primary, shift, alt, ctrl, key }`, `Shortcut::parse("cmd-shift-s")`
  (aceita `cmd`, `ctrl`, `shift`, `alt`, `option`; `cmd` é o primário,
  cmd no macOS e ctrl fora); exige um modificador: uma letra solta como
  key equivalent roubaria a tecla de todo campo de texto (por isso `D` e
  `T` não têm atalho no menu).
- `Clicked { sem, action, param }` e `Clicked::request(&seen)` =
  `Request::observe(seen, Actor::Human, sem, action, param)`, o mesmo
  request do botão.
- `trait NativeMenu { install(&MenuModel), update(&MenuModel), poll() ->
  Vec<Clicked> }`. `FakeMenu` guarda o modelo, conta `installs` e
  `updates`, `click(sem, action, param)` enfileira só item mostrado e
  habilitado, `poll` devolve. `NoMenu` responde `MenuError::Unsupported`.
  `impl NativeMenu for Arc<Mutex<M>>` deixa um teste segurar o falso
  enquanto o shell segura o `Box<dyn NativeMenu>`.
- `MudaMenu::new(wake)`: registra o handler único do `muda` (uma vez por
  processo, `Once`) que empilha o `MenuId` numa fila global e chama o
  waker atual; `install` monta `Menu` + `Submenu`s + `MenuItem`/
  `CheckMenuItem`/`PredefinedMenuItem` com `with_id(key)`, guarda `id ->
  Clicked`, `init_for_nsapp` no macOS (`init_for_hwnd` no windows quando
  `attach_window` deu o hwnd); `update` compara: igual não faz nada,
  mesma forma (`MenuModel::same_shape`: as mesmas chaves nos mesmos
  lugares, mesmo atalho, mesmo tipo marcado ou não) muda `set_text`,
  `set_enabled`, `set_checked` no lugar, forma diferente refaz a barra
  (item cuja capacidade sumiu sai); `poll` drena a fila e mapeia os ids
  (um id de barra refeita é logado e descartado). um `MudaMenu` por
  processo: a fila é do processo.
- `MenuError::{Unsupported, BadShortcut(String), Platform(String)}`.

### por janela e por app

macOS: a barra é do app (`NSApp.mainMenu`), uma para todas as janelas; o
shell do showcase guarda um `menu: Option<menu::Menu>` (`app.rs`) ao lado
do estado que projeta, instala no `resumed` depois da janela principal
(substitui a barra padrão do winit) e a janela da página mostra a mesma
página, logo a mesma barra. windows: a barra é da janela;
`MudaMenu::attach_window(&Window)` extrai o hwnd pelo
`raw_window_handle` e chama `init_for_hwnd`, escrito e não compilado
aqui (windows não é alvo). linux: backend `noop` do `muda` até o
bring-up. `WindowHost`, `Windows::route`, `lifecycle.rs` não mudaram: a
janela não guarda menu porque no macOS ele não é dela. o `poll` acontece
no laço do app: o clique acorda o laço (`UserEvent::Menu` pelo proxy),
`user_event` drena (`menu_clicks`), despacha pela view e toma o que é do
shell (`detach_requested`, `save_requested`); a barra segue a projeção
depois de cada frame da janela principal (`sync_menu` em `pace.rs`).

### o shell projetado

`apps/uigallery/src/views/dispatcher/shell/mod.rs` (430 de produção;
`shell/tests.rs` com 7 testes). `Shell` mora em `Panes` no lugar de
`detach_asked`; o `Operable` é um `ShellOp` que projeta a partir da view
(tema, página, destacada) e escreve os pedidos (`Asks`), despachado pelo
dispatcher da galeria (`TodoPage::with_dispatcher`, como Surface,
Silences e Files). nós:

- `shell` (Region, "plev").
- `shell.page` (Select, "page"): `selected` = título da página ativa,
  `Constraints { editable, options: títulos de PAGES }`, `select` com
  param `Text` constrained lendo `shell.page:selected`; policy padrão
  (qualquer ator). `execute` guarda o índice; a view aplica `set_page`
  logo após o dispatch. os dígitos e o clique na sidebar pedem por ele.
- `shell.theme` (Select, "theme"): `selected` = `theme_name`, opções =
  `THEMES`, `select` idem; qualquer ator. `T` e o card da página Theme
  pedem por ele; a view aplica `set_theme`.
- `shell.detach` (Toggle, "page in a window of its own"): `on` =
  destacada, `enabled` só no desktop e com página que tem pane
  (placeholder não), `toggle` lendo `shell.detach:on`, policy `Actors {
  human, assistive, test: true; agent: false }` (a mesma do
  `open_file`). `execute` marca o pedido; o shell lê com
  `take_detach_request` (como antes) e abre ou fecha a janela.
- `shell.save_session` (Button, "save the session recording"), só
  enquanto `set_recording(true)` (o `App::record` chama com
  `PLEV_RECORD`): `activate`, só pessoa. `execute` marca; o shell lê com
  `take_save_request` e chama `save_session("asked")`. `cmd-shift-s`
  (`record_key`) pede por ele (`ShowcaseView::request_save`).
- nenhum é `focusable` (nada na tela toma foco por eles).

`ShowcaseView::semantics()` continua a projeção da página (com o nó
`window` quando destacada), byte a byte: o golden do todo, os 161
baselines, a árvore de acessibilidade, a gravação e o cmd+click leem
essa. o shell entra em `gallery_semantics()` (shell + página, um
snapshot, os nós da página reconstruídos por `detached::extend`, que
`with_window` também usa), que a porta do agente passa a servir
(`Served::project`) e o menu lê. `act(req)` roteia por `is_shell(target)`
para `act_shell`, senão para a página. `ask_shell(target, action, param)`
é o pedido da pessoa observado no shell; `menu_click(&Clicked)` é o
pedido observado na galeria e roteado por `act`.

### o menu do showcase

`apps/uigallery/src/shell/system/menu.rs` (234 de produção; 2 testes). a barra,
de `gallery_semantics`: "plev" (About, separador, Hide, Hide Others,
Show All, separador, Quit, como a barra padrão do winit menos Services),
"view" (submenu "theme" com uma opção por tema e a marca na atual,
submenu "go to" com as 81 páginas e a marca na ativa, separador, o item
marcável "page in a window of its own", e, gravando, separador e "save
the session recording" com `cmd-shift-s`) e o título da página ativa com
as ações simples dela (`MenuItem::plain` sem os nós do shell e sem
linhas: ListItem, Row, Cell, TreeItem, Option; rótulo: label da ação se
declarada, Button pelo label do nó, Tab por "lista: aba" ("filter:
Active"), o resto "nó: ação" ("new todo: submit")), só quando há alguma.
na Todo: "new todo: submit" (desabilitado sem texto), "filter: All",
"filter: Active", "filter: Completed"; na Silences: os increment e
decrement dos dois sliders, "detect silences", "apply all" e a linha de
propor; na Card, sem submenu de página. trocar de página muda a forma e
a barra é refeita; digitar no campo do todo muda só o `enabled` do
submit, no lugar. o clique num item corre a mesma capacidade que o
botão, pelo mesmo `Request::observe` por `Actor::Human`: inert atrás de
um modal, disabled, stale e a policy valem iguais.

### testes

engine: `platform/menu.rs` 8 unitários (atalho, item e opções de um
snapshot, ações simples, o clique é o request do botão, forma do modelo,
o falso, o handle compartilhado, `NoMenu`), `menu/native.rs` 2 (o
acelerador do `muda`, o `&` do rótulo), `tests/menu.rs` 6 sobre os
goldens do todo e das silences: os itens, rótulos e habilitados; um
clique no falso gera o `Request` por `Actor::Human` igual ao do botão e o
mesmo receipt (mesmos estágios, mesmo outcome) num `Operable` de teste,
e um clique com observação velha é `Stale`; a atualização desabilita o
que ficou `enabled: false` e tira o item cuja capacidade sumiu (a forma
muda); atalho num item e letra solta recusada; `NoMenu` não quebra.

showcase: `dispatch/shell/tests.rs` 7 (a projeção e as policies; as
teclas deixam receipts, `take_detach_request` e `take_save_request`; os
cards da Theme e a sidebar pedem igual; um agente troca tema e página por
`act` e é recusado no destacar como `NotAuthorized`, `Assistive` e
`Test` podem, título fora das opções é `InvalidParam`; um pick velho é
`Stale` com o delta; a galeria carrega shell e página e a projeção da
página fica sozinha; um clique de menu roda a capacidade da página como
o botão, disabled é recusado, item inexistente não pede nada);
`dispatch/agent.rs` 1 pela porta real (o agente vê `shell.*` ao lado de
`todo`, troca tema e página a partir da revisão vista, é recusado no
destacar); `app/menu.rs` 2 com `App::with_bare_window` e um `FakeMenu`
compartilhado (a barra, as marcas, um tema e um filtro clicados pela
view, o destacar esperando o shell, a troca de página refazendo a barra,
o save com o atalho enquanto grava). os testes de hoje de `T`, dos
dígitos, da sidebar, do `D` (detached.rs) e do save key (record.rs)
continuam verdes.

### gate e regress

`df -h /` antes do gate: 31 GB livres; 29 GB depois. `./tool/tasks/fullcheck.sh`
verde no worktree, com a perna wasm e a perna urnaui (`../urna`
presente): antes `cargo test --workspace` 2494 passed, 0 failed, 16
ignored em 117 binários; depois 2520 passed, 0 failed, 16 ignored em 118
binários (+26: 10 unitários do `platform/menu`, 6 de `engine/tests/menu.rs`,
7 do shell, 1 da porta, 2 do menu do app; +1 binário, o
`tests/menu.rs`). perna urnaui: 85 passed, 0 failed, 1 ignored em 6
binários, igual a antes. clippy `-D warnings`, fmt e a checagem wasm
limpos nas duas pernas (o showcase compila para wasm com `NoMenu`).

`cargo run -p regress -- check`: 161 baselines, 161
iguais, 0 mudaram, sem frame desenhado e sem GPU aberta (807 ms). o
golden do todo (`tests/uir_fixture.rs`) e os goldens das silences e da
files continuam iguais: a projeção da página não mudou.

linhas de produção (regra do guard): engine `platform/menu.rs` 606,
`platform/menu/native.rs` 317, `platform/mod.rs` 59; showcase `app.rs`
580 (era 546), `app/menu.rs` 234, `app/pace.rs` 161, `app/record.rs`
330, `view/dispatch.rs` 467 (era 452), `view/dispatch/shell.rs` 430,
`view/dispatch/detached.rs` 229, `view/dispatch/agent.rs` 101,
`view/input.rs` 416 (era 400), `view/panes.rs` 136. tudo abaixo de 639.

### ADR

`docs/adr/interface/0057-native-menus-are-another-view-of-capabilities.md` (proposed):
o modelo puro do menu, o clique como o request do botão, o trait com o
real, o falso e o nenhum, por app no macOS e por janela onde a
plataforma diz, o shell projetado e onde ele aparece, a barra do
showcase, o que o tray e os diálogos reaproveitam. linha pro índice, que
é do orquestrador (`docs/adr/README.md` é proibido aqui):

| 2026-09-24 | [native-menus-are-another-view-of-capabilities](../../adr/interface/0057-native-menus-are-another-view-of-capabilities.md) | proposed | `engine::platform::menu`: um `MenuModel` puro que o app monta da própria `Snapshot` (cada item é um `SemId` e uma ação, com label, atalho, habilitado e marca como a projeção diz), `NativeMenu` (`install`, `update`, `poll`) com o `muda` no desktop (macOS: a barra do app, uma pra todas as janelas; windows: por hwnd), `FakeMenu` pra teste e `NoMenu` em wasm e mobile; o clique volta como `Clicked` e vira o mesmo `Request::observe` por `Actor::Human` do botão, pelo mesmo dispatcher; o showcase projeta o shell (`shell.page`, `shell.theme`, `shell.detach` só pessoa, `shell.save_session` só pessoa) na projeção da galeria que a porta serve, e monta a barra dela e da página ativa; a projeção da página não muda (task-76 item 4) |

### interfaces adicionadas (só crescimento)

`engine::platform::menu::{MenuModel, Submenu, Entry, MenuItem,
Predefined, Shortcut, Clicked, MenuError, NativeMenu, FakeMenu, NoMenu,
locked, default_menu}`, `MudaMenu` (desktop; `new`, `installed`,
`attach_window` no windows) e `impl NativeMenu for Arc<Mutex<M>>`.
`ShowcaseView::{shell_semantics, gallery_semantics, is_shell,
request_save, take_save_request, set_recording, menu_click}`,
`ask_shell` (pub(crate)), `view::dispatch::{Shell, ShellTarget}`;
`take_detach_request` mudou de módulo (`detached.rs` para `shell.rs`),
mesma assinatura. `Panes.shell` no lugar de `Panes.detach_asked` (o
único leitor era a própria view). `UserEvent::Menu` no shell do
showcase. `App::{install_menu, sync_menu, menu_clicks, save_requested}`
(pub(super)). nenhuma assinatura pública existente mudou; ide, urnaui,
wasm e os exemplos compilam sem tocar.

### arquivos tocados fora da lista permitida, e por quê

- `apps/urnadbapp/Cargo.lock`: consequência do `muda` no `Cargo.toml` do
  engine (o urnaui depende dele por caminho; a perna urnaui do gate
  resolve o lock). o mesmo que o `rfd` fez no item 3.
- `apps/uigallery/src/views/mod.rs` foi tocado num rascunho (um
  re-export) e voltou ao da base antes do commit: `ShellTarget` fica
  dentro de `view`; o `app` nomeia as capacidades pelos ids
  (`shell.theme`...) e pede o save por `ShowcaseView::request_save`.

### limitações e decisões, sem esconder

- a barra real não foi vista nem clicada: com `PLEV_BACKGROUND=1` a
  política de ativação é proibida e o macOS não mostra a barra de um app
  assim; o que a rodada prova é o `setMainMenu` no `NSApp` vivo sem
  pânico e os 100 itens montados. o clique num item real, o `Quit` pelo
  menu (o `terminate:` do `muda`, o mesmo do winit) e o `About` são do
  orquestrador. o caminho do clique está lido na fonte do `muda` e
  provado com o falso.
- `ShowcaseView::semantics()` (página) e `gallery_semantics()` (shell +
  página) são duas projeções do mesmo estado: a porta observa a galeria,
  a acessibilidade, a gravação, o `--semantics`, o regress e o golden
  leem a página. escolhi assim porque a projeção da página é o contrato
  com os 161 baselines e o golden; a alternativa (o shell em
  `semantics()`) movia todos. a árvore de acessibilidade não tem os nós
  do shell (eles não têm binding na tela); é candidata a ganhar quando
  houver um widget que os desenhe.
- `gallery_semantics` reconstrói os nós da página por `SnapshotBuilder`
  (como `with_window`), então o `source` deles passa a ser o
  `detached.rs`; a porta descarta `source` a menos que peçam. o
  `SnapshotBuilder::extend` no `sem` continua sendo a mudança que
  guardaria.
- um `MudaMenu` por processo: a fila de cliques e o waker são globais
  (o handler do `muda` é um por processo). dois menus no mesmo processo
  dividiriam a fila; o primeiro a `poll` levaria os cliques do outro.
- a barra segue a projeção depois de cada frame da janela principal:
  cada frame custa uma projeção da galeria e a montagem do modelo (a
  ordem de dezenas de microssegundos; na Todo com 103 itens uns 50 us,
  pela medição do task-59), mesmo quando só o caret piscou. um frame da
  janela da página sozinha não sincroniza a barra (a página é a mesma).
- windows: `attach_window` e `init_for_hwnd` estão escritos contra a API
  do `muda` e do `raw-window-handle` e não compilaram aqui. o windows
  precisa também do `TranslateAcceleratorW` no laço para os aceleradores
  (nota do `muda`), que o winit não faz; fica pro bring-up.
- `D` e `T` não têm atalho na barra: uma letra solta como key equivalent
  roubaria a tecla de todo campo de texto (o AppKit resolve o key
  equivalent antes do `keyDown` da view). o item existe e a tecla
  continua no shell, pelo mesmo `shell.detach` e `shell.theme`. só o
  save tem atalho (`cmd-shift-s`, com modificador).
- com o atalho na barra, o `cmd-shift-s` chega pelo menu (o AppKit toma
  o key equivalent antes da janela) e não mais pelo `record_key`; os
  dois caminhos pedem o mesmo `shell.save_session`, então o resultado é
  o mesmo. em teste sem barra o `record_key` é o caminho.
- os dígitos, a sidebar e os cards da Theme agora deixam um receipt cada
  no dispatcher da galeria (retenção 256): um agente que lê receipts vê
  a pessoa trocando de página; é o desejado (uma capacidade, várias
  representações), mas é novo.
- a página Theme continua com a própria geometria dos cards
  (`card_rects` privado em `foundations/theme.rs`, fora da lista); o
  teste dos cards acha um card perguntando à página, não ao layout.
- `Predefined` tem só About, Hide, HideOthers, ShowAll e Quit; Services
  (que a barra padrão do winit tinha) ficou de fora, e Copy, Cut, Paste,
  Undo não entraram: são capacidades do editor, não itens da plataforma,
  e o editor não as projeta ainda.
- o `page` submenu usa o título da página como rótulo, então a barra é
  refeita a cada troca de página (uma `setMainMenu`); é o comportamento
  esperado do `same_shape`, não medido em tempo.
- `menu.rs` do engine está em 606 linhas de produção, perto do limite;
  um crescimento (ícones, mais itens da plataforma) pede o split em
  `model.rs` e `fake.rs`.

### o que os itens 5 e 8 reaproveitam daqui

- item 5 (tray e notificações): o tray é outro `MenuModel` mostrado por
  outro objeto da plataforma (o `tray-icon` do mesmo autor consome
  `muda::Menu` como menu do ícone); o `Clicked` e o `menu_click` da view
  servem iguais, e `Windows::set_exit_when_last_closes(false)` do item 2
  deixa o app viver só no tray. o `wake` pelo proxy e a fila global são
  os mesmos; um segundo consumidor da fila (barra e tray) pede que o
  `MudaMenu` passe a distribuir por id em vez de drenar tudo. uma
  notificação que tem ação é um `Clicked` também.
- item 8 (diálogos nativos, com janela pai e `rfd::MessageDialog`): a
  policy "só pessoa" (`PERSON_ONLY` do shell, a mesma do `open_file`) e o
  molde trait, real, falso, nenhum; um item "Save session..." ou
  "Open..." do menu já dispara a capacidade da página pelo mesmo
  caminho, então um diálogo de confirmação nativo para uma ação
  `destructive` ou `Confirm::Always` entra no dispatcher entre o
  `NeedsConfirmation` e o `confirm`, onde o botão também pararia; a
  janela pai vem do `Windows` do item 2.

### comandos de validação

```text
export CARGO_TARGET_DIR=/Users/nn/Dev/hoff/plevdev/target/agents/one
export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_PROFILE_TEST_DEBUG=line-tables-only
cargo metadata --offline --format-version 1 > /dev/null   (o lock resolve offline)
cargo test --offline -p engine --test menu
cargo test --offline -p uigallery --lib -- dispatch::shell app::menu dispatch::agent
cargo test --offline -p engine -p uigallery
./tool/tasks/fullcheck.sh
cargo run --offline -p regress -- check
PLEV_BACKGROUND=1 RUST_LOG=showcase=info cargo run --offline -p uigallery -- Todo   (a barra instala; sem clique aqui)
cargo run --offline -p uigallery -- Todo   (a barra plev / view / Todo; do orquestrador)
```

## handoff do item 8: diálogos nativos de alerta e confirmação

branch `task/76e-alerts`, base `0e49f828` (main após o item 4). worktree
`.claude/worktrees/agent`, target `target/agents/one`.

### a sonda

(a) o caminho de hoje no ide, por teste sem janela
(`cargo test --offline -p idestudio --lib -- workbench::tests`, 22 verdes, 0,19 s):

- um request que exige confirmação chega ao dispatcher e fica em
  `AwaitingConfirmation`: o `discard` de um agente (a policy do nó é
  `Confirm::ForAgents`, `an_agent_discard_waits_and_a_test_cannot_confirm_it`)
  e o `stage` com `--ask` (`Request::ask`, task-92, mesmo sem policy:
  `an_agents_question_is_asked_in_the_modal_and_never_projected`).
- `Workbench::dispatch` chama `report(id)` (`workbench/person.rs`): o
  receipt em `AwaitingConfirmation` entra na fila `awaiting` e
  `ask_next_confirmation` pega o mais velho, monta título, corpo e botão
  em `confirmation(&request)` (`notice.rs`: "Discard changes?",
  "test-agent asks to discard all changes to b.txt. This cannot be
  undone.", "Discard"; para o `--ask`: "Question from test-agent",
  "test-agent asks the person: \"...\" If allowed, it will stage b.txt.",
  "Allow") e chama `workspace.open_confirm_modal(id, title, body,
  confirm)` (`views/workspace/overlays.rs:136`), que empilha um
  `OverlayKind::Modal` com `PendingAction::ConfirmRequest { request }`.
  um modal por vez: o segundo espera o primeiro fechar
  (`requests_waiting_for_a_person_are_asked_one_at_a_time`).
- a pessoa responde pela view: o clique no botão de confirmar vira
  `UiRequest::Confirm { request }` e o cancelar, o Escape e o clique fora
  viram `UiRequest::Decline` (`dismiss_overlays`); o workbench traduz em
  `confirmed_by_person` -> `Workbench::confirm(id, Actor::Human)` ->
  `Dispatcher::confirm` (segunda checagem de stale, `Stage::Confirmed {
  by: human }`, `Executing`) e `declined_by_person` ->
  `Dispatcher::decline` -> `Stage::Rejected(Declined { by: human })`,
  nada roda. `Workbench::confirm` e `decline` fecham o modal
  (`close_confirm_modal`), avisam por toast e chamam
  `ask_next_confirmation` de novo. um agente ou um teste que tenta
  confirmar recebe `SemError::ConfirmByMachine` e o modal continua
  (`a_screen_reader_declines_through_the_workbench_and_a_test_cannot`).
- o que a gravação registra (`apps/idestudio/tests/record.rs`, o sink do
  `rec` no dispatcher): cada estágio do receipt, inclusive
  `awaiting_confirmation`, `confirmed { by: human }` ou `rejected {
  declined { by: human } }`; o `plev-inspect` mostra o estágio com
  `stage_detail` ("by human", `inspect/src/describe.rs:148`). o clique no
  modal entra pelo shell como `Kind::Input { PointerDown, at: Some([x,
  y]), target }` (`main.rs`, `record_input`) com `target =
  compositor.hit(x, y)`: o modal não liga nenhum id, então o alvo gravado
  é o controle ligado que estiver por baixo do diálogo, ou nenhum. quem
  respondeu está no estágio, não no input.

(b) o `rfd` 0.15.4 (`~/.cargo/registry/src/*/rfd-0.15.4/src/`):

- `MessageDialog::show` (`message_dialog.rs:82`) é síncrono e bloqueante:
  no macOS (`backend/macos/message_dialog.rs`) roda em `run_on_main`
  (exige a thread principal; fora dela, `dispatch2::run_on_main` se o
  `NSApp` está rodando, senão pânico) e chama `NSAlert::runModal()`, um
  laço modal aninhado do AppKit; o laço do winit fica parado até a
  resposta. com `set_parent` o alerta abre como sheet da janela
  (`beginSheetModalForWindow_completionHandler` e o mesmo `runModal`);
  sem pai, `sync_pop_dialog` abre um painel solto. `MessageLevel` vira
  `NSAlertStyle` (Informational, Warning, Critical); `MessageButtons`
  vira `addButtonWithTitle` na ordem, e o resultado mapeia pelo índice do
  botão: com `OkCancelCustom(a, b)` a resposta é `Custom(a)` ou
  `Custom(b)`, e qualquer outra coisa (Escape) é `Cancel`.
- `set_parent<W: HasWindowHandle + HasDisplayHandle>(&W)`
  (`message_dialog.rs:70`) guarda `RawWindowHandle` e `RawDisplayHandle`.
  `W` é `Sized`: um `&dyn Trait` não entra direto, então o engine passa um
  wrapper de uma linha (`ParentRef(&dyn Parent)`) que delega os dois
  métodos. a `Arc<Window>` do host serve: o `raw-window-handle` 0.6.2
  (único no lock, o do winit 0.30.13 e o do rfd) implementa
  `HasWindowHandle` e `HasDisplayHandle` para `Arc<H>` (`borrowed.rs:68,
  196`) e o `winit::window::Window` implementa os dois. o winit
  reexporta o crate (`winit::raw_window_handle`), então o engine não
  ganha dependência.
- alvos: macOS (`NSAlert`), windows (`win_cid`, botões custom só com
  `common-controls-v6`), linux gtk3 e portal xdg (o portal não tem
  diálogo de mensagem: `xdg_desktop_portal.rs:273` cai no `zenity` por
  processo). wasm tem `window.alert` e `window.confirm`
  (`backend/wasm.rs:329`), síncronos, mas o `rfd` só entra no engine na
  seção de desktop não móvel (`rust/engine/Cargo.toml:89`), então wasm
  e mobile ficam com `NoAlerts`. android e ios: nenhum backend.
  `AsyncMessageDialog` existe (mesma builder, `show` devolve um future);
  fica fora deste item.

(c) onde os apps mostram aviso e erro hoje, por conta própria:

- ide: o toast (`WorkspaceView::notify(message, intent)`,
  `views/workspace/mod.rs:305`), chamado só por `Workbench::report`
  (`person.rs:29`) com o texto de `notice.rs` (recusa, decline, falha do
  git; `Intent::Destructive` para stale e falha). os toasts são
  projetados como nós `notices.*` (`Role::Alert` para falha,
  `Role::Status` para decline) e o agente lê a mesma frase pela porta,
  em terceira pessoa.
- showcase: a página Files diz no `files.status` (`Role::Status`,
  `message`) o que o último request deu, com cor `Destructive` quando o
  alvo não tem diálogo (`files.rs:368`); as silences e a todo têm
  `Role::Status` para contagens; a página Toast é demonstração do
  widget. nada disso abre painel nativo.
- comps: `Toast`, `Alert` e `Modal` são widgets desenhados. o alerta
  nativo entra como uma capacidade a mais (um botão que pede
  `Alerts::alert` pelo dispatcher, só pessoa), com o texto do último
  alerta projetado como estado: o toast e o `status` continuam sendo a
  verdade do app; o painel nativo é outra representação, nunca uma
  segunda fonte.

### o trait, o falso e o que cada alvo faz

`rust/engine/src/runtime/platform/alerts.rs` (449 linhas de produção, 627 com
os testes), pelo molde do `files.rs`, linha por linha: trait, real sob
o cfg do `arboard` e do `rfd`, falso em todo alvo, nenhum em wasm e
mobile, `default_alerts()` escolhendo.

- `Alert { level: Level (Info, Warning, Error), title, body }`, com
  `Alert::info`, `warning`, `error`; `Confirm { title, body,
  confirm_label, cancel_label, destructive }`, com `Confirm::new(title,
  body)` ("OK", "Cancel", não destrutivo), `.labels(confirm, cancel)` e
  `.destructive(bool)`. `Answered { Confirmed, Declined }`;
  `AlertError::Unsupported(AlertKind { Alert, Confirm })` ("alert: no
  native dialog on this target"); `Shown<T> = Result<T, AlertError>`.
- `Parent`: marcador sobre `HasWindowHandle + HasDisplayHandle`
  (`winit::raw_window_handle`, sem dependência nova), implementado para
  todo tipo `Sized` com os dois handles: a `Arc<Window>` do host entra
  como `&dyn Parent` sem wrapper (teste de compilação
  `the_hosts_window_is_a_parent`). o `rfd` quer um tipo `Sized` no
  `set_parent`, então o `NativeAlerts` embrulha o `&dyn Parent` num
  `ParentRef` de uma linha.
- `trait Alerts { alert(&Alert, Option<&dyn Parent>) -> Shown<()>;
  confirm(&Confirm, Option<&dyn Parent>) -> Shown<Answered> }`,
  síncrono e modal como o `rfd`, na thread que pediu.
- `NativeAlerts` (desktop não móvel): `rfd::MessageDialog` com o nível
  como ícone, o alerta com `MessageButtons::Ok`, a confirmação com
  `OkCancelCustom(confirm_label, cancel_label)` e ícone de aviso quando
  `destructive`; a resposta lida pelo rótulo do botão (`Custom(confirm)`
  confirma; `Custom(cancel)`, `Cancel` do Escape e qualquer outra coisa
  recusa); `set_parent(ParentRef)` quando há pai (sheet da janela no
  macOS). os builders nunca abrem nada; o teste
  `the_native_builders_carry_the_request` confere o `Debug` do builder e
  `the_native_answer_is_read_by_the_button_pressed` o mapa da resposta.
- `FakeAlerts`: `will_confirm()`, `will_decline()`, `will_refuse()`
  respondem em ordem (um alerta consome uma resposta como o diálogo de
  arquivo consome), `asked()`/`take_asked()` devolvem `Asked { dialog:
  Dialog::Alert(..) | Dialog::Confirm(..), with_parent }`, `pending()`.
  sem nada carregado, um alerta é mostrado e uma confirmação é recusada,
  então uma página sob teste que pergunta sem querer não roda nada.
  `Arc<Mutex<A>>` implementa `Alerts` para o teste ficar com a outra
  ponta, como no `files.rs`.
- `NoAlerts` (wasm, android, ios): `Unsupported` nos dois. o wasm até
  tem `window.alert`/`confirm` pelo `rfd`, mas o `rfd` não entra no
  engine nesse alvo; fica como capacidade a nomear.

### a opção do ide e o que a gravação registra

`apps/idestudio/src/capability/mod.rs` (369 linhas), `person.rs` (133),
`main.rs` (588).

- `Workbench::confirm_natively(alerts: Box<dyn Alerts>, parent:
  Option<Box<dyn Parent>>)` e `confirms_natively()`. o `Workbench` guarda
  um `NativeConfirm { alerts, parent }` opcional; `None` (o padrão) é o
  modal desenhado de hoje.
- `ask_next_confirmation` continua igual até ter título, corpo e botão
  de `confirmation(&request)` (o texto nasce no mesmo lugar). com a
  opção ligada, `ask_natively` monta `alerts::Confirm::new(title,
  body).labels(confirm, "Cancel").destructive(d)`, com `d` lido da
  policy da ação no nó projetado agora (`destructive` do `discard` é
  true, do `stage` é false; o modal desenhado usa `Intent::Destructive`
  para toda confirmação), chama `alerts.confirm(&ask,
  parent.as_deref())` e a resposta entra por `confirmed_by_person` ou
  `declined_by_person`: o mesmo `Workbench::confirm`/`decline` com
  `Actor::Human`, a mesma segunda checagem de stale, o mesmo toast de
  decline, o mesmo `ask_next_confirmation` para o próximo da fila.
  `Unsupported` cai no modal desenhado (log de aviso). nenhum modal
  abre no caminho nativo (`modal_open()` fica false).
- o shell: `PLEV_NATIVE_CONFIRM=1` (`native_confirm()` ao lado de
  `background()`), aplicado em `resumed` logo depois de criar a janela:
  `bench.confirm_natively(default_alerts(), Some(Box::new(window.clone())))`,
  então o diálogo abre como sheet da janela do ide. desligado por
  padrão. `views/**`, `record.rs`, `agent.rs` e o `sem` não mudaram.
- o que a gravação registra: os estágios do receipt pelo sink que já
  está no dispatcher (`awaiting_confirmation`, `confirmed { by: human
  }` ou `rejected { declined { by: human } }`) e nenhum `Kind::Input`,
  porque nenhum clique nem tecla chegou à janela: o diálogo é da
  plataforma. o `plev-inspect` diz quem respondeu pelo estágio (`by
  human`, o mesmo `stage_detail` do modal), e `why.between.inputs` fica
  vazio. decidi não inventar um input sem coordenada: o `rec` só tem
  `PointerDown/Up/Move/Wheel/Key/Text`, um deles mentiria sobre o que
  aconteceu, e "quem respondeu" já mora no estágio, como com o modal (o
  clique no modal também não liga id nenhum, ver a sonda). o `rec` e o
  `record.rs` ficaram como estavam.
- um agente pela porta não abre nem responde nada: o request dele fica
  em `AwaitingConfirmation` como hoje e é a pessoa que responde no
  diálogo; `confirm` pelo agente continua `ConfirmByMachine`
  (`an_agents_question_is_asked_in_the_dialog_and_the_agent_never_answers_it`,
  `without_a_native_dialog_the_modal_is_drawn_instead`).

### o alerta no showcase

a página Files, sem página nova (`apps/uigallery/src/views/apps/files/mod.rs`
452 linhas, `files/semantics.rs` 353): um botão "warn" (ícone
`alert-triangle`) ao lado do "open file", com o status depois dos dois.

- `files.warn` (`Role::Button`, ação `warn`, `reads files:received`,
  policy `OPEN_FILE_ACTORS`: human, assistive, test; agente recusado
  como `NotAuthorized` antes do `execute`). o clique e o Enter viram
  `warn` por `Actor::Human` pelo dispatcher da galeria; `execute` monta
  `Alert::warning("Files", "<n> files received; nothing was written to
  disk.")` ("1 file" no singular), chama `alerts.alert(&alert, None)` e
  projeta o corpo mostrado em `files:alert` (texto, vazio até o primeiro
  alerta) e no resultado do receipt (`alert`). o status diz "the warning
  was shown"; em `Unsupported` o request falha ("alert: no native dialog
  on this target") e o status diz "no native alert on this target". a
  página não conhece a janela (o shell é dono das janelas e `app.rs`
  está fora deste item), então o painel abre solto, sem pai.
- Tab agora percorre os dois botões: open, warn, nenhum; `focus(id)`
  aceita `files.warn`; `FilesPage::with(dialogs, alerts)` e
  `set_alerts` (cfg(test)); `FilesPage::new()` usa `default_alerts()` na
  janela e `FakeAlerts` sob `cfg(test)`, como os diálogos.
- a projeção mudou de propósito: `files:alert` e o nó `files.warn`. o
  golden `rust/plevir/tests/fixtures/uigallery-files.snapshot.json` foi regravado
  (`UIR_BLESS=1`; só cresceu: 27 linhas) e as duas baselines da página
  (`files@core@1280x800`, `files@hoff@1280x800`) foram blessed com
  `--png` (a camada `files` mudou num retângulo de 379x48 em 562,300 no
  core e 415x44 em 561,286 no hoff: o botão novo; 5 controles ligados, 5
  nós). o subtítulo da página em `pages/table.rs` ainda fala só do
  diálogo e do drop; `table.rs` está fora da lista, fica para o
  orquestrador se quiser.

### testes

engine: `platform/alerts.rs` 9 unitários (o falso em ordem e o que
mostrou, sem nada carregado, `Unsupported`, o handle compartilhado,
`NoAlerts`, os builders e os defaults, a `Arc<Window>` como `Parent`, o
builder do `rfd` carregando o pedido, a resposta pelo rótulo);
`tests/alerts.rs` 4 sobre um `Operable` de teste (o `warn` da pessoa
mostra o alerta e o texto vira estado que o agente lê; o agente é
recusado antes de abrir e o teste e o `Assistive` não; `NoAlerts` falha
dizendo por quê; a confirmação da plataforma entra no dispatcher como
`confirm`/`decline` de pessoa: confirmado roda depois da segunda
checagem, recusado não roda, o agente não responde).

ide: `tests/native_confirm.rs` 5 contra repositório real e o worker de
verdade, com o `FakeAlerts` (um discard do agente confirmado roda e o
receipt tem `confirmed { by: human }`, recusado não roda e o toast diz,
nenhum modal abre, o falso recebeu título, corpo e botão do modal com
`destructive` true e sem pai; a pergunta do `--ask` chega pelo mesmo
diálogo em terceira pessoa, sem aviso destrutivo, e o agente não
responde; `Unsupported` cai no modal; a opção desligada abre o modal
como hoje; a sessão gravada tem a resposta como estágio, nenhum input, e
o `Inspector::why` diz `Confirmed { by: Human }` e `Declined { by: Human
}`). os testes de hoje do ide (`human_agent` 24, `agent_port` 9, `lab`
10, `record` 1, `rerun` 9, lib 59) verdes sem mudança.

showcase: `files/tests.rs` 10 (os 8 de antes com o nó novo, o Tab de
dois passos e o `files.warn` entre os ids ligados e observados pela
porta, mais `the_warn_button_shows_the_platforms_warning_and_the_text_shown_is_state`
e `an_agent_is_refused_the_alert_and_reads_what_the_person_saw`; pela
porta real o agente é recusado no `warn` com `["requested", "rejected"]`
e lê `files:alert` vazio).

### gate e regress

`df -h /` antes do gate: 29 GB livres; 29 GB depois. `./tool/tasks/fullcheck.sh`
verde no worktree com a perna wasm e a perna urnaui (`../urna`
presente): antes (o fechamento do item 4, base `0e49f828`) 2520
passed, 0 failed, 16 ignored em 118 binários; depois 2540 passed, 0
failed, 16 ignored em 120 binários (+20: 9 unitários do
`platform/alerts`, 4 de `engine/tests/alerts.rs`, 5 de
`ide/tests/native_confirm.rs`, 2 da Files; +2 binários, os dois
`tests/*.rs` novos). perna urnaui: 85 passed, 0 failed, 1 ignored em 6
binários, igual a antes. clippy `-D warnings`, fmt e a checagem wasm
limpos nas duas pernas (o showcase compila para wasm com `NoAlerts`).
`Cargo.lock` e `apps/urnadbapp/Cargo.lock` não mudaram.

`cargo run -p regress -- check`: antes do bless, 161
baselines, 159 iguais, 2 mudaram (`files@core`, `files@hoff`, o botão
novo); depois do bless dito acima, 161 iguais, 0 mudaram, sem frame
desenhado e sem GPU aberta (676 ms). o golden do todo e os das silences
continuam iguais.

linhas de produção (regra do guard): engine `platform/alerts.rs` 449,
`platform/mod.rs` 80; ide `workbench/mod.rs` 369, `workbench/person.rs`
133, `main.rs` 588; showcase `view/apps/files.rs` 452,
`view/apps/files/semantics.rs` 353. tudo abaixo de 639.

### ADR

`docs/adr/interface/0058-native-dialogs-answer-the-same-contract.md` (proposed): o
trait com o real, o falso e o nenhum; a confirmação nativa como opção do
app sobre o mesmo `confirm`/`decline`; o alerta só de pessoa com o texto
projetado; o que o wasm e o mobile fazem; o que o item 5 e o
`AsyncMessageDialog` levam daqui. linha pro índice (`docs/adr/README.md`
está fora da lista, fica para a integração):

```text
| 2026-09-24 | [native-dialogs-answer-the-same-contract](native-dialogs-answer-the-same-contract.md) | proposed | task-76 item 8: `engine::platform::alerts` com `Alert` (nível, título, corpo) e `Confirm` (título, corpo, dois rótulos, destrutivo) atrás do trait `Alerts` (`alert`, `confirm` -> `Answered`, `Unsupported`), `NativeAlerts` (`rfd::MessageDialog`, só desktop, `set_parent` com a `Arc<Window>` do host via o marcador `Parent`), `FakeAlerts` (responde em ordem e registra o que mostrou) e `NoAlerts` (wasm e mobile); a confirmação humana da task-58 pelo diálogo nativo como opção do ide (`PLEV_NATIVE_CONFIRM=1`, `Workbench::confirm_natively`, desligada por padrão) sobre o mesmo `confirm`/`decline` de `Actor::Human`, mesmo texto do modal, mesma segunda checagem, gravação e inspector iguais e sem input; o alerta como capacidade só de pessoa na Files do showcase (`files.warn`, `files:alert` com o texto que a pessoa viu); o diálogo bloqueia o laço, o `AsyncMessageDialog` fica nomeado |
```

### interfaces adicionadas (só crescimento)

- `engine::platform::alerts::{Level, Alert, Confirm, Answered,
  AlertKind, AlertError, Shown, Parent, Alerts, Answer, Dialog, Asked,
  FakeAlerts, NoAlerts, NativeAlerts (desktop), default_alerts}`.
- `ide::workbench::Workbench::{confirm_natively, confirms_natively}`.
- showcase (crate-private): `FilesPage::with(dialogs, alerts)`,
  `set_alerts` (cfg(test)), `Ids::warn`, `Target::Warn`; a projeção da
  Files ganhou `files.warn` e `files:alert`.
- variável do shell do ide: `PLEV_NATIVE_CONFIRM`.

### arquivos tocados fora da lista permitida, e por quê

nenhum. `rust/engine/src/runtime/platform/{alerts.rs,mod.rs}`,
`rust/engine/tests/alerts.rs`, `apps/idestudio/src/capability/{mod.rs,person.rs}`,
`apps/idestudio/src/main.rs`, `apps/idestudio/tests/native_confirm.rs`,
`apps/uigallery/src/views/apps/files/mod.rs`, `files/{semantics.rs,tests.rs}`,
`rust/plevir/tests/fixtures/uigallery-files.snapshot.json`,
`tool/bench/regress/baselines/files@{core,hoff}@1280x800/**`, o ADR
novo e este arquivo. `rust/engine/src/lib.rs` não precisou mudar
(`platform` já é `pub mod`). `record.rs` não mudou (ver a decisão da
gravação acima).

### limitações e decisões, sem esconder

- o diálogo nativo bloqueia a thread principal enquanto está aberto
  (`NSAlert::runModal`): com a opção ligada, o `act` de um agente pela
  porta cujo request espera uma pessoa só recebe resposta quando o
  diálogo fecha, e já com o estágio final, em vez de
  `awaiting_confirmation` na hora como com o modal; resultados do git e
  frames ficam na fila do laço enquanto isso. é por isso que o modal
  desenhado continua o padrão. o `AsyncMessageDialog` do `rfd` (mesma
  builder, `show` devolve um future) é o que manteria o laço vivo; fora
  deste item.
- o diálogo é pedido de dentro do `dispatch` (`report` ->
  `ask_next_confirmation`), como o modal é aberto; um por vez, o próximo
  da fila é perguntado quando a resposta entra.
- `destructive` da confirmação nativa segue a policy do nó; o modal
  desenhado continua `Intent::Destructive` para toda confirmação (não
  mexi em `views/**`).
- o alerta da Files abre sem janela pai: a página não tem o handle, e
  dar a ela exigiria mexer em `app.rs`/`dispatch.rs`, fora da lista. o
  ide, que tem a janela, passa o pai.
- nenhum diálogo nativo abriu nesta máquina: a implementação real ficou
  atrás do trait, cada teste usa o falso, e os dois testes do
  `NativeAlerts` só constroem o builder e leem o mapa da resposta. a
  prova na tela é do orquestrador (abaixo).
- windows: rótulos custom no `rfd` exigem a feature
  `common-controls-v6`; não é alvo, não liguei. linux: gtk3 ou `zenity`
  pelo portal, escolha do bring-up.
- o `FakeAlerts` consome uma resposta por chamada, alerta incluído
  (mesma regra do `FakeDialogs`); um teste que carrega `will_confirm()`
  e cuja página alerta antes de confirmar gasta a resposta no alerta.
- o subtítulo da página Files (`pages/table.rs`) não fala do alerta.

### o que o item 5 (notificações e tray) reaproveita, e o que fica para provar na tela

- item 5: uma notificação é um `Alert` que a plataforma mostra sem
  bloquear e sem resposta, então o trait não serve direto: o molde
  (trait, real, falso, nenhum, `default_*()`), o `Level` e o
  `Alert { title, body }` servem como estão; uma notificação com ação é
  um `Clicked` do menu (item 4) pelo mesmo dispatcher; o tray reaproveita
  a policy só pessoa e o `MenuModel`. o `Parent` serve para qualquer
  painel que precise da janela.
- para o brenner provar na tela, do orquestrador, nunca de teste:
  `PLEV_NATIVE_CONFIRM=1 PLEV_AGENT=1 cargo run --offline -p idestudio <repo>`,
  um `plev act --ask` ou um discard pelo agente: o sheet do macOS abre
  sobre a janela do ide com "Discard changes?" / "test-agent asks to
  discard all changes to X. This cannot be undone." e os botões
  "Discard" e "Cancel"; um clique em cada, e o `plev-inspect` da sessão
  (`PLEV_RECORD=1`) mostra `confirmed by human` e `rejected declined`.
  e na Files do showcase, um clique em "warn": o painel de aviso do
  sistema com "Files" / "0 files received; nothing was written to
  disk.", e `files:alert` com o texto pela porta.

### comandos de validação

```text
export CARGO_TARGET_DIR=/Users/nn/Dev/hoff/plevdev/target/agents/one
export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_PROFILE_TEST_DEBUG=line-tables-only
cargo test --offline -p engine --lib platform::alerts
cargo test --offline -p engine --test alerts
cargo test --offline -p idestudio --test native_confirm
cargo test --offline -p uigallery --lib files::
cargo test --offline -p engine -p idestudio -p uigallery
./tool/tasks/fullcheck.sh
cargo run --offline -p regress -- check
cargo run --offline -p uigallery -- --render Files --theme core --out files.png --semantics files.json
UIR_BLESS=1 cargo test --offline -p uigallery --lib files::tests::the_page_projection   (regrava o golden)
cargo run --offline -p regress -- bless --page Files --theme core --png   (e --theme hoff)
PLEV_NATIVE_CONFIRM=1 PLEV_AGENT=1 PLEV_RECORD=1 cargo run --offline -p idestudio <repo>   (o sheet; do orquestrador)
```

## handoff do item 5: notificações do sistema e tray como capacidades

branch `task/76f-tray-notify`, base `2f8274e9` (main após o item 8).
worktree `.claude/worktrees/agent`, target `target/agents/one`. commits,
em ordem:

- `b66c601f` task-76: the tray and notification probe
- `2550c324` task-76: the tray as another view of the capabilities, with
  the fake and the tray-icon one
- `fbb7641d` task-76: the notification as a capability out of the window,
  with the fake and why the desktop cannot yet
- `aeaf95b6` task-76: the showcase tray, the Files notice and the job that
  ends in the background
- (este) task-76: ADR and the item 5 handoff

### a sonda

(a) o lock. `tray-icon = { version = "0.25", default-features = false }`
e `objc2-user-notifications = { version = "0.3", default-features =
false, features = [std, block2, UNUserNotificationCenter,
UNNotificationContent, UNNotificationRequest, UNNotificationResponse,
UNNotificationAction, UNNotificationCategory, UNNotification] }` em
`rust/engine/Cargo.toml`, resolvidos com `cargo metadata --offline`:
"Locking 2 packages to latest compatible versions: Adding
objc2-user-notifications v0.3.2, Adding tray-icon v0.25.1". o
`Cargo.lock` da raiz ganhou só essas duas entradas `[[package]]`; nada
mais entrou: `crossbeam-channel 0.5.15`, `muda 0.20.0`, `objc2 0.6.4`,
`objc2-app-kit 0.3.2`, `objc2-core-foundation 0.3.2`, `objc2-core-graphics
0.3.2`, `objc2-foundation 0.3.2`, `once_cell 1.21.4`, `png 0.18.1`,
`thiserror 2.0.18`, `windows-sys 0.61.2`, `bitflags 2.13.0` e `block2
0.6.2` já estavam. arestas novas: `objc2-foundation 0.3.2` ganhou `block2
0.6.2` (feature), e a entrada `objc2-user-notifications 0.2.2` que o lock
já tinha (via o winit, alvo ios) passou a ser nomeada com a versão no
consumidor, porque haveria duas. features do `objc2-app-kit 0.3.2` que o
`tray-icon` liga a mais: `NSStatusBar`, `NSStatusItem`,
`NSStatusBarButton`, `NSTrackingArea`, `NSButton`, `NSCell`, `NSControl`
(as outras, `NSMenu`, `NSImage`, `NSEvent`, `NSView`, `NSWindow`, o winit,
o muda e o rfd já ligavam). `cargo tree --offline -p engine -i tray-icon`
mostra só o engine como consumidor. nada faltou no cache; não parei.

depois da sonda, o `objc2-user-notifications` saiu do `Cargo.toml` (ver
(b)): o lock final da raiz tem só o `tray-icon` (21 linhas). o `tray-icon`
entrou em `cfg(any(target_os = "macos", target_os = "windows"))`, não na
seção de desktop toda: no linux o crate tem `compile_error!` sem a feature
`libappindicator` ou `ksni` (escolha do bring-up de linux), então o linux
fica com `NoTray`.

o `apps/urnadbapp/Cargo.lock` (workspace próprio, depende do engine por
caminho) não pôde ser resolvido pelo cargo: aquele workspace não declara
`resolver` e a edição 2024 lhe dá o resolver 3, ciente de MSRV, e qualquer
relock ali ("Locking 23 packages to latest Rust 1.85 compatible versions")
rebaixa 22 pacotes que não têm nada com o tray (async-trait, bytes,
blocking...) e ainda pede download de um que não está no cache. escrevi a
entrada do `tray-icon` e as duas arestas (`engine -> tray-icon`,
`objc2-foundation 0.3.2 -> block2 0.6.2`) à mão, na forma que aquele lock
usa (`thiserror 2.0.20`, `crossbeam-channel` sem versão), e `cargo check
--locked --offline --manifest-path apps/urnadbapp/Cargo.toml` aceitou: o
lock é exatamente o que o cargo quer, sem tocar nos outros 22. a perna
urnaui do gate rodou verde sobre ele. na raiz o `resolver = "2"` explícito
não tem esse comportamento.

(b) `UNUserNotificationCenter` num binário sem bundle, por leitura (não
rodei). o `objc2-user-notifications` 0.3.2 é binding gerado:
`currentNotificationCenter() -> Retained<UNUserNotificationCenter>`,
`requestAuthorizationWithOptions_completionHandler`,
`addNotificationRequest_withCompletionHandler(&UNNotificationRequest,
Option<&DynBlock>)`, `setDelegate(Option<&ProtocolObject<dyn
UNUserNotificationCenterDelegate>>)` com
`userNotificationCenter_didReceiveNotificationResponse_withCompletionHandler`
para o clique, `setNotificationCategories(&NSSet<UNNotificationCategory>)`
para o botão nomeado. o que a Apple faz: `+[UNUserNotificationCenter
currentNotificationCenter]` procura o bundle principal do processo
(`bundleProxyForCurrentProcess`) e, quando o executável não está dentro de
um `.app`, levanta `NSInternalInconsistencyException` ("bundleProxyForCurrentProcess
is nil: mainBundle.bundleURL = file:///.../target/debug/"); o objc2 não
apanha exceção por padrão, então o `cargo run` de hoje abortaria na
primeira chamada. e escrever esse caminho pede `objc2` (o `define_class!`
do delegate, `Retained`, `MainThreadMarker`) e `objc2-foundation`
(`NSString`, `NSSet`, `NSArray`) como dependências diretas, além das duas
aprovadas (as duas já estão no lock, mas não estavam na aprovação).

alternativas pesadas: `NSUserNotification` (deprecado desde o 10.14,
também quer bundle identifier; quem usa sem bundle troca o
`bundleIdentifier` por swizzling, uma gambiarra); `osascript -e 'display
notification'` como processo filho (aparece como Script Editor, não tem
botão, e o clique não volta: não cumpre o contrato do `Clicked`);
`Unsupported` até o app ter bundle. recomendação, e o que fiz:
`NativeNotifier` no macOS responde `NotifyError::NeedsBundle` quando o
executável está fora de um `.app` (lido pelo caminho, `bundled(exe)`:
`<Nome>.app/Contents/MacOS/<exe>`, o mesmo lugar por onde o sistema acha o
bundle) e `Unsupported` dentro de um, até o caminho do framework ser
escrito contra um bundle que exista; o `objc2-user-notifications` não
entra no manifesto sem uso. o app loga e segue; o texto projetado é a
verdade. registro, não parada, como o briefing previa.

(c) no `tray-icon` 0.25.1 (`src/lib.rs`, `platform_impl/macos/mod.rs`):
`TrayIcon::new` exige `MainThreadMarker::new()` (senão
`Error::NotMainThread`) e cria o `NSStatusItem`
(`NSStatusBar::systemStatusBar().statusItemWithLength(..)`) na hora, com
o ícone, o tooltip e uma `TrayTarget` (uma `NSView` que recebe os cliques)
dentro do botão: o ícone aparece ao ser criado; `set_visible(false)` só o
remove depois. logo não dá para criar um sem aparecer, e nenhum teste
cria um real. a nota do crate para winit: criar do `StartCause::Init` em
diante, com o laço rodando (o showcase cria no `resumed`, depois da janela,
como o menu). os eventos: `TrayIconEvent::{Click {id, position, rect,
button, button_state}, DoubleClick (windows), Enter, Move, Leave}`
chegam na thread da plataforma pelo handler único do processo
(`set_event_handler`, um `OnceCell`) ou pelo canal global
(`receiver`); com handler, o canal não recebe. o menu do ícone é
`Box<dyn muda::ContextMenu>` (`Menu` e `Submenu` implementam), e os
cliques dos itens saem pelo `MenuEvent` do `muda`, o mesmo handler que a
barra do item 4 já registrou: logo a barra e o tray dividem uma fila.
com `menu_on_left_click` ligado, o `mouseDown` esquerdo abre o menu num
laço aninhado (`performClick`) e o `mouseUp` envia `Click { Up }` depois.

(d) da projeção, o que entra no tray e o que uma notificação da Files
leva: do shell, `shell.theme#select#<tema>` (14 opções),
`shell.page#select#<título>` (81), `shell.detach#toggle`,
`shell.save_session#activate` (só gravando), todos já na barra; e o novo
`shell.show#activate` (só pessoa). da Files, `files.open#open_file` e
`files.warn#warn` (só pessoa) e `files.drop#drop_files` (param lista);
a notificação "N files received" leva `open_file`. da Silences, um job
que termina é o request `detect_silence` completando (`cuts` no outcome)
ou falhando; a notificação leva `shell.show`, não `apply_all`: o
`apply_all` aplica cortes de uma página que pode ter mudado até a pessoa
clicar, e "mostrar a janela" é o que quem lê "2 silences found" quer.

### os dois modelos e o que cada alvo faz

`rust/engine/src/runtime/platform/tray/mod.rs` (346 linhas de produção, 562 com os
testes) e `tray/native.rs` (181), pelo molde do `menu.rs`:

- `TrayImage { width, height, rgba, template }`: `from_rgba(w, h,
  bytes)` (recusa tamanho vazio e bytes que não batem), `mark(size)` (um
  anel que o engine mesmo desenha, mínimo 4 px, como template para a
  plataforma tingir), `template(bool)`.
- `TrayModel { icon, tooltip, menu: Submenu, on_click: Option<Clicked>
  }`: o menu é um `Submenu` do `platform::menu` (os mesmos `MenuItem`,
  separadores, `Predefined`, submenus; o rótulo não aparece), e não um
  `MenuModel`, porque um menu de tray tem itens no topo (o "show", o
  Quit) e o `MenuModel` só tem submenus no topo. `on_click` é a
  capacidade que o clique no ícone pede: com ela, o botão esquerdo pede e
  o direito abre o menu; sem ela, os dois abrem o menu. `items()`,
  `find()`, `same_shape()` (a do menu: ícone, tooltip, rótulos, marcas e o
  clique mudam no lugar).
- `trait Tray { install, update, poll -> Vec<Clicked> }`;
  `TrayError::{Unsupported, BadIcon(String), Platform(String)}` com
  `From<MenuError>`.
- `FakeTray`: guarda o modelo, conta `installs` e `updates`, `click(sem,
  action, param)` só em item mostrado e habilitado, `click_icon()`
  enfileira o `on_click`, `poll` devolve. `NoTray` (linux até o bring-up,
  wasm, mobile) responde `Unsupported`. `impl Tray for Arc<Mutex<T>>`.
  `default_tray(wake)`.
- `NativeTray` (macOS e windows; windows escrito contra a API e não
  compilado): `install` monta o `muda::Submenu` com o `build` do
  `menu/native.rs` (agora `pub(in crate::platform)`, com o prefixo
  `tray:` nos ids), converte o `TrayImage` em `tray_icon::Icon` e cria o
  `TrayIcon` com `with_icon`, `with_icon_as_template`, `with_tooltip`,
  `with_menu`, `with_menu_on_left_click(on_click.is_none())`; um segundo
  `install` (forma nova) troca o menu no mesmo ícone (`set_menu`), sem
  recriar o item. `update`: igual não faz nada; mesma forma muda ícone
  (`set_icon_with_as_template`), tooltip, o modo do clique e as entradas
  no lugar (`refresh` do menu); outra forma refaz o menu. `poll`: os
  cliques no ícone (contados pelo handler do `tray-icon`, um por
  processo, só `Left`/`Up`) viram o `on_click`, e os ids `tray:` saem da
  fila compartilhada e mapeiam para o `Clicked`.
- a fila dividida: `menu/native.rs` ganhou `TRAY_PREFIX`,
  `route_clicks_to(wake)` (o handler único do `muda`, o waker é o último
  dado), `wake()` e `take_clicks(mine)` (retira da fila só os ids que o
  chamador reconhece, deixa os outros); o `MudaMenu::poll` toma os ids
  sem prefixo, o `NativeTray::poll` os com. `build(submenu, ids,
  prefix)`, `refresh`, `Installed` e `platform` ficaram visíveis dentro
  de `platform`. um id de menu (`sem#action[#param]`) nunca começa com
  `tray:` (id não tem `:`).

`rust/engine/src/runtime/platform/notify.rs` (404 de produção, 584 com os
testes), pelo molde do `alerts.rs`:

- `Action { label, sem, action, param }`: `Action::new(label,
  clicked)`; `Action::of(snapshot, sem, action)` lê pelo `MenuItem::of`
  e recusa (None) o que uma pessoa não poderia apertar (policy sem
  `human`, ou desabilitado); `label()`, `clicked()`.
- `Notification { level: alerts::Level, title, body, action:
  Option<Action> }` com `new`, `info`, `warning`, `error`, `action()`.
- `NotificationId(u64)` (`Display` "notification-N");
  `NotifyError::{Unsupported, NeedsBundle, Platform(String)}`;
  `Shown<T>`.
- `trait Notifier { notify(&Notification) -> Shown<NotificationId>, poll
  -> Vec<Clicked> }`: `notify` volta na hora; o clique na notificação ou
  no botão é o `Clicked` da ação, e uma sem ação não devolve nada.
- `FakeNotifier`: `sent()`, `take_sent()` (`Sent { id, notification }`),
  `click(id)` (só de notificação enviada e com ação), `will_refuse()`
  (`Unsupported`), `will_want_a_bundle()` (`NeedsBundle`), `pending()`.
  `NoNotifier`. `impl Notifier for Arc<Mutex<N>>` e `for Box<N>`.
  `default_notifier(wake)`.
- `NativeNotifier` (desktop): no macOS `NeedsBundle` fora do bundle,
  `Unsupported` dentro (até o caminho existir); windows e linux
  `Unsupported`; `poll` vazio. `bundled(exe: &Path)` pública e testada.

### o showcase

- `apps/uigallery/src/views/dispatcher/shell/mod.rs` (495 de produção):
  `shell.show` (Button "show plev", `activate`, `PERSON_ONLY`, `enabled`
  no desktop, não focável); `execute` marca `asks.show`;
  `take_show_request()`. o nó `shell` ganhou `notice` (`shell:notice`):
  a última coisa que o shell contou por notificação por conta própria,
  `set_notice(text)`. `set_window_focused(on)` repassa à Files;
  `take_notifications()` recolhe o que a Files pediu. `menu_click` serve
  igual para barra, tray e notificação (documentado; não renomeei).
  `ShellTarget` não ganhou `Show`: o `shell.show` só é pedido de fora da
  janela, por `Clicked`.
- `apps/uigallery/src/shell/system/tray.rs` (145 de produção; 1 teste):
  `TrayIcon { native, shown }`, `install_tray` (no `resumed`, depois do
  menu; um alvo sem tray é logado), `sync_tray` (depois de cada frame da
  janela principal, junto do `sync_menu`), `tray_clicks` (dispacha pela
  view), `set_tray` (teste). o modelo: `TrayImage::mark(36)` (18 pontos na
  barra, nítido em 2x), tooltip "plev", menu "show plev" (e o `on_click`
  do ícone é o mesmo `shell.show#activate`), separador, as entradas do
  submenu "view" da barra (`view_menu`, agora `pub(super)`: tema, go to,
  a página em janela própria, o save gravando), separador, Quit. as
  ações da página ficam na barra: o tray é do app atrás da janela. o
  clique na barra, no tray ou numa notificação acorda o laço pelo mesmo
  `UserEvent::Clicked` (era `UserEvent::Menu`), que faz `menu_clicks`,
  `tray_clicks`, `notice_clicks`, depois `show_requested`,
  `save_requested`, `detach_requested`.
- `apps/uigallery/src/shell/system/notify.rs` (205 de produção; 2 testes):
  `Notices { notifier: Box<dyn Notifier>, jobs }`; `install_notices`
  (`default_notifier`, no `resumed`) e `set_notices(notifier)` (o teste
  passa o falso), que também põe um `JobSink: ReceiptSink` no dispatcher
  da galeria (`ShowcaseView::add_sink`): ele ouve todo receipt e guarda o
  fim dos `detect_silence` (`Completed` com `cuts`, ou `Failed` com o
  erro). `focus_changed(id, on)` (do `WindowEvent::Focused` em
  `events.rs`) guarda qual janela nossa tem o foco e diz à view; no
  `resumed` a view começa "sem foco" até a plataforma dizer (com
  `PLEV_BACKGROUND=1` nunca diz: o que cai é contado). `notify_pending`
  (no fim de `advance`, a cada acordar do laço, logo depois do `wake` da
  view, onde o `settle` da Silences completa o request): recolhe o que a
  Files pediu e, sem foco, o que os jobs deram ("2 silences found", "1
  silence found", ou o erro como `Notification::error`), com "show"
  (`Action::of(shell.show)`) como botão e o texto em `shell:notice`; envia
  cada um; recusa é logada. `notice_clicks` dispacha pela view.
  `show_requested`: `set_visible(true)`, `set_minimized(false)`,
  `focus_window()` na janela principal.
- a Files (`files.rs` 485 linhas, `files/semantics.rs` 385):
  `window_focused` (padrão true: uma página sozinha é olhada),
  `set_window_focused`, `notices` (a caixa de saída) e
  `take_notifications`, `notice` projetado como `files:notice`. no
  `execute` de `drop_files`, fora de foco: `received_notice(n)`
  (`Notification::info("Files", "N files received")`, "1 file" no
  singular, com `Action::of(snapshot, files.open, "open_file")` como
  botão, rótulo "open file"), o corpo vai para `files:notice` e para o
  `notice` do outcome, a notificação para a caixa. quem soltou não
  importa (um agente pela porta também): a pessoa não estava olhando. a
  página não tem `Notifier`: o shell é dono do da plataforma, como do
  menu, e mostra o que a página pediu; o texto é estado mostrado ou não.
- silences: nenhum arquivo tocado. o fim do job chega ao shell pelo
  receipt do próprio request (o `settle` chama `d.complete`/`d.fail` no
  dispatcher da galeria), que é o contrato; não precisei de uma chamada
  ao `Notifier` no fim do job nem de um campo na página.

### testes

engine: `platform/tray.rs` 5 unitários (o anel e os bytes, o modelo e a
forma, o falso com clique no menu e no ícone, o handle compartilhado,
`NoTray`), `tray/native.rs` 1 (a imagem vira `tray_icon::Icon` ou diz
por quê, sem tocar a plataforma), `platform/notify.rs` 7 (a ação é uma
capacidade que uma pessoa pode pedir agora, o falso e o clique, as
recusas, o handle compartilhado e o `Box`, `NoNotifier`, `bundled`, o
`NativeNotifier` dizendo por quê), `tests/tray.rs` 4 (o menu do tray a
partir do golden da todo; um clique no falso gera o `Request` por
`Actor::Human` igual ao do botão e o mesmo receipt, o clique no ícone pede
o `show`, um clique velho é `Stale`, um agente é `NotAuthorized` no
`show`; o tray segue a projeção, desabilitado e sumido; `NoTray`),
`tests/notify.rs` 4 sobre um inbox de teste (a notificação com botão, o
clique é o request do botão com o mesmo receipt, o texto projetado, um
clique velho é `Stale`; um agente arquiva e o botão sumiu: o clique não
gera request nenhum; uma sem botão não devolve clique; a capacidade de
agente nunca é botão e o agente é recusado no botão; sem plataforma o
texto ainda é projetado).

showcase: `dispatch/shell/tests.rs` (os 7 de antes estendidos: o
`shell.show` e a policy, `shell:notice`, `menu_click` no show deixa
receipt e `take_show_request` toma uma vez, o agente recusado no show,
+1 nó na galeria), `files/tests.rs` +1
(`files_that_land_while_no_window_has_the_focus_are_told_by_notification`:
olhada não conta; sem foco conta com o botão open, o texto é estado e
sai no outcome, o clique do botão é o `open_file` da pessoa pelo
dispatcher, o singular, o drop do agente conta, o último notice fica),
`app/tray.rs` 1 (o tray sobre uma janela nua com `FakeTray`: show
primeiro e no ícone, tema, go to, detach, sem save, sem ações da página,
Quit por último; o ícone pede o show; um tema clicado pela view e a
atualização no lugar; troca de página com a mesma forma e 1 install; o
detach espera o shell; o save aparece gravando), `app/notify.rs` 2 (a
Files fora de foco pela App com `FakeNotifier`: contado uma vez, o clique
abre pelo diálogo falso e o status diz; com foco de volta não conta; a
detecção pedida por uma pessoa na Silences, cujo job termina falhando
porque ffmpeg não é procurado nos testes do crate: `advance` conta o erro
com "show" e projeta em `shell:notice`, o clique pede o show, com foco de
volta o fim não é contado; o texto de 1 e 2 cortes).

### gate e regress

`df -h /` antes do gate: 27 GB livres; 25 GB depois. `./tool/tasks/fullcheck.sh`
verde no worktree com `CARGO_NET_OFFLINE=true`, com a perna wasm e a
perna urnaui (`../urna` presente): antes (o fechamento do item 8, base
`2f8274e9`) 2540 passed, 0 failed, 16 ignored em 120 binários; depois
2565 passed, 0 failed, 16 ignored em 122 binários (+25: 13 unitários do
engine, 4 de `tests/tray.rs`, 4 de `tests/notify.rs`, 1 da Files, 1 do
tray do app, 2 das notificações do app; +2 binários, os dois `tests/*.rs`
novos). perna urnaui: 85 passed, 0 failed, 1 ignored em 6 binários, igual
a antes. clippy `-D warnings`, fmt e a checagem wasm limpos nas duas
pernas (o showcase compila para wasm com `NoTray` e `NoNotifier`). o
único aviso no log é o `block v0.1.6` de incompatibilidade futura, que já
estava.

`cargo run -p regress -- check`: antes do bless, 161
baselines, 159 iguais, 2 mudaram (`files@core@1280x800`,
`files@hoff@1280x800`: "semantics changed: notice none -> ''; visual
unchanged, layer files"); `bless --page Files --theme core` e `--theme
hoff` (sem `--png`: só `scene.json`, a revisão, e `semantics.json`
mudaram; os `frame.png` são os mesmos), depois 161 iguais, 0 mudaram, sem
frame desenhado e sem GPU aberta. o bless é dito aqui e o motivo é o
`files:notice` que o briefing pediu; se a integração preferir o texto só
no shell, os dois voltam com `git checkout` e o `notice` sai da Files. o
golden `rust/plevir/tests/fixtures/uigallery-files.snapshot.json` foi regravado
(`UIR_BLESS=1`; +4 linhas, o `notice`). o golden da todo e os das
silences continuam iguais.

linhas de produção (regra do guard, `file_hygiene_guard` verde): engine
`platform/tray.rs` 346, `platform/tray/native.rs` 181,
`platform/notify.rs` 404, `platform/menu/native.rs` 356 (era 317),
`platform/menu.rs` 606, `platform/mod.rs` 61; showcase `app.rs` 621
totais (era 580; a exceção `cfg(test)` desconta), `app/tray.rs` 145,
`app/notify.rs` 205, `app/menu.rs` 239, `app/events.rs` 208,
`app/pace.rs` 166, `view/dispatch/shell.rs` 495 (era 430),
`view/apps/files.rs` 485 totais (era 452), `view/apps/files/semantics.rs`
385 (era 353). tudo abaixo de 639.

### ADR

`docs/adr/interface/0059-tray-and-notifications-are-capabilities-out-of-the-window.md`
(proposed): os dois modelos, o clique de volta como o request do botão,
a fila com dois donos, quem notifica, o que exige bundle no macOS e o que
fica para o bring-up, o showcase. linha pro índice (`docs/adr/README.md`
está fora da lista, fica para a integração):

```text
| 2026-09-24 | [tray-and-notifications-are-capabilities-out-of-the-window](tray-and-notifications-are-capabilities-out-of-the-window.md) | proposed | task-76 item 5: `engine::platform::tray` com `TrayModel` (imagem RGBA ou o anel do engine como template, tooltip, um `Submenu` dos mesmos `MenuItem` da barra, e a capacidade que o clique no ícone pede) atrás do trait `Tray` (`install`, `update`, `poll`), `NativeTray` (`tray-icon`, macOS e windows; a fila de cliques do `muda` dividida com a barra por prefixo), `FakeTray`, `NoTray` (linux até o bring-up, wasm, mobile); `engine::platform::notify` com `Notification` (nível, título, corpo, no máximo uma `Action` que nomeia uma capacidade da projeção) atrás do trait `Notifier` (`notify` -> id, `poll`), `FakeNotifier`, `NoNotifier`, `NativeNotifier` que no macOS diz `NeedsBundle` fora de um `.app` (o framework recusa processo sem bundle; o bundle é do bring-up); todo clique volta como o `Clicked` do menu e vira o mesmo `Request::observe` por `Actor::Human` do botão, recusado onde o botão seria e sem request quando a capacidade sumiu; o showcase projeta `shell.show` (só pessoa) e `shell:notice`, mostra o tray com o shell, a Files conta por notificação os arquivos que caem sem janela em foco (`files:notice`, botão `open_file`) e o shell conta o fim da detecção da Silences ouvindo o receipt (`ReceiptSink`), sem mexer na página |
```

### interfaces adicionadas (só crescimento)

- `engine::platform::tray::{TrayImage, TrayModel, TrayError, Tray,
  FakeTray, NoTray, default_tray}` e `NativeTray` (macOS, windows; `new`,
  `installed`); `impl Tray for Arc<Mutex<T>>`; `impl From<MenuError> for
  TrayError`.
- `engine::platform::notify::{Level (o de alerts), Action, Notification,
  NotificationId, NotifyError, Shown, Notifier, Sent, FakeNotifier,
  NoNotifier, bundled, default_notifier}` e `NativeNotifier` (desktop);
  `impl Notifier for Arc<Mutex<N>>` e `for Box<N>`.
- `engine::platform::menu::native` passou a `pub(in crate::platform)`
  (era privado), com `TRAY_PREFIX`, `route_clicks_to`, `wake`,
  `take_clicks`, `build(submenu, ids, prefix)`, `refresh`, `Installed`
  visíveis dentro de `platform`; `MudaMenu::new` e o trait `NativeMenu`
  não mudaram.
- `ShowcaseView::{take_show_request, set_notice, set_window_focused,
  take_notifications}`; a projeção do shell ganhou `shell.show` e
  `shell:notice`; a da Files ganhou `files:notice` (e `notice` no outcome
  do `drop_files` fora de foco). `FilesPage::{set_window_focused,
  take_notifications}`. `UserEvent::Menu` virou `UserEvent::Clicked`
  (privado do app). `App::{install_tray, sync_tray, tray_clicks,
  install_notices, set_notices, focus_changed, notify_pending,
  notice_clicks, show_requested}` (pub(super)); `menu::view_menu`
  (pub(super)). nenhuma assinatura pública existente mudou; ide, urnaui,
  wasm e os exemplos compilam sem tocar.

### arquivos tocados fora da lista permitida, e por quê

- `apps/urnadbapp/Cargo.lock`: consequência do `tray-icon` no
  `Cargo.toml` do engine, como `muda` e `rfd` antes; à mão, pelo motivo
  da sonda (a), aceito por `--locked` e pela perna urnaui do gate.
- nenhum outro. `rust/engine/src/lib.rs` não precisou mudar (`platform`
  já é `pub mod`). `silences/**` não foi tocado.

### limitações e decisões, sem esconder

- nenhum ícone de tray e nenhuma notificação de verdade nesta máquina: o
  `NativeTray` está compilado e não rodou (criar um mostra na barra), a
  notificação real não existe (bundle). o que a rodada prova é o modelo,
  o falso, o clique de volta pelo dispatcher e o showcase sobre janela
  nua. a prova na tela é do orquestrador: `cargo run --offline -p uigallery
  -- Files`, o anel na barra de menus; clique direito abre o menu (show
  plev, theme, go to, page in a window of its own, Quit); clique esquerdo
  traz a janela para frente; com a janela atrás de outra, soltar arquivos
  nela mostra "notification not shown (notification: the system's
  notification center wants an app bundle...)" no log com `RUST_LOG=
  showcase=info` e `files:notice` pela porta diz "2 files received".
- o `objc2-user-notifications` não entrou: seria dependência sem uso. a
  linha exata para quando o bundle existir está na sonda (a); o caminho
  precisa ainda de `objc2` e `objc2-foundation` diretos, um `define_class!`
  para o `UNUserNotificationCenterDelegate` (o clique volta em
  `didReceiveNotificationResponse`, com `actionIdentifier` igual ao id da
  nossa `Action` e `UNNotificationDefaultActionIdentifier` para o toque no
  corpo), uma `UNNotificationCategory` com a `UNNotificationAction` do
  botão, `requestAuthorization` na primeira vez, e `Info.plist` com
  `CFBundleIdentifier`. tudo do bring-up de plataforma.
- o `NativeNotifier` é, hoje, um `NoNotifier` que sabe dizer por quê;
  existe para o molde (real, falso, nenhum) e para o `bundled`. quando o
  real chegar, ele pode não ser `Send` (objetos objc): o showcase o guarda
  em `Box<dyn Notifier>` sem `Arc`, de propósito.
- um `NativeTray` por processo (a contagem de cliques no ícone e o
  handler do `tray-icon` são globais) e um laço por processo (o waker do
  `muda` é o último dado; barra e tray dão o mesmo proxy). a perna windows
  está escrita e não compilada.
- o tray segue a projeção depois de cada frame da janela principal, como
  a barra: mais uma projeção da galeria por frame (a ordem de dezenas de
  microssegundos, medida no item 4). o menu do tray carrega o acelerador
  `cmd-shift-s` do save gravando, o mesmo da barra; num menu de contexto
  ele vale só com o menu aberto.
- o app continua fechando ao fechar a janela principal: viver só no tray
  (`Windows::set_exit_when_last_closes(false)`, reabrir a janela pelo
  `show`) fica para depois; o `shell.show` hoje traz para frente, tira de
  minimizado e torna visível a janela que existe.
- a Files conta o drop de quem quer que seja fora de foco, agente
  incluído; um agente que soltasse arquivos em série fora de foco geraria
  uma notificação por drop. é o desejado (a pessoa não estava olhando) e
  é novo; um agente nunca pede notificação por si.
- a notificação de fundo da Silences leva `shell.show` e não `apply_all`
  (sonda (d)); o fim de uma detecção com a janela em foco não é contado
  (a pessoa vê na tela). o `Failed` também é contado, como erro: um job
  que morreu enquanto ninguém olhava merece aviso.
- a projeção da Files mudou (`files:notice`) e com ela o golden e duas
  baselines, por significado; o subtítulo da página em `pages/table.rs`
  continua sem falar de notificação (fora da lista).
- o `ShellTarget` não ganhou `Show`: nada dentro da janela pede o show
  por alvo, e a variante daria dead code; se uma tecla ou um botão vier a
  pedir, ela entra.

### o que fica para o bring-up e o que o item 6 reaproveita

- bring-up de plataforma: o bundle do app (e com ele o caminho
  UserNotifications acima); um backend de tray no linux (`libappindicator`
  ou `ksni` como feature do `tray-icon`); compilar e provar a perna
  windows do `NativeTray`.
- item 6 (deep links): um `Request` vindo de fora tem a mesma forma do
  `Clicked` (uma capacidade nomeada, um param opcional) e o mesmo caminho
  (chega na thread da plataforma, acorda o laço pelo proxy, é `poll`ado
  e vira `Request::observe(gallery, Actor::Human, ..)` por `menu_click`);
  o `UserEvent::Clicked` é o acordar que serve; a recusa quando a
  capacidade sumiu e a policy valem iguais. a diferença: o esquema de URL
  do app no macOS também exige bundle (`CFBundleURLTypes`), o mesmo
  bring-up.

### comandos de validação

```text
export CARGO_TARGET_DIR=/Users/nn/Dev/hoff/plevdev/target/agents/one
export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_PROFILE_TEST_DEBUG=line-tables-only
export CARGO_NET_OFFLINE=true
cargo metadata --offline --format-version 1 > /dev/null   (o lock da raiz resolve offline)
cargo check --locked --offline --manifest-path apps/urnadbapp/Cargo.toml   (o lock do urnaui é o que o cargo quer)
cargo test --offline -p engine --lib platform::tray platform::notify
cargo test --offline -p engine --test tray --test notify --test menu
cargo test --offline -p uigallery --lib -- dispatch::shell app::menu app::tray app::notify files::
cargo test --offline -p engine -p uigallery
./tool/tasks/fullcheck.sh
cargo run --offline -p regress -- check
UIR_BLESS=1 cargo test --offline -p uigallery --lib files::tests::the_page_projection   (regrava o golden)
cargo run --offline -p regress -- bless --page Files --theme core   (e --theme hoff; sem --png)
RUST_LOG=showcase=info cargo run --offline -p uigallery -- Files   (o anel na barra, o menu, o show; do orquestrador)
```

## handoff do item 7: sleep e wake

branch `task/76g-sleep-wake`, base `e1386d5a` (main após o item 5).
worktree `.claude/worktrees/agent`, target `target/agents/one`. commits,
em ordem:

- `7d88e4f3` task-76: the sleep and wake probe
- `f4c03f08` task-76: a covered window and a sleeping app ask for no
  frame, and show with a new one
- `022c423f` task-76: the showcase sleeps, is covered, and wakes with the
  work beside the frames
- `1bd8a11f` task-76: the gallery's first frame after a sleep or an
  uncover is the state now, byte for byte
- (este) task-76: ADR and the item 7 handoff

os testes do item 5 do briefing saíram junto do comportamento que provam
(commits 2 e 3), como as diretrizes pedem; o commit 4 é o teste de
integração do showcase com pixels.

### a sonda

(a) o que o winit 0.30.13 entrega, lido na fonte (nenhuma janela aberta,
a máquina nunca dormiu):

- macOS: `platform_impl/macos/window_delegate.rs:348`,
  `windowDidChangeOcclusionState:` lê `NSWindowOcclusionState::Visible`
  ("ao menos parcialmente na tela") e enfileira `WindowEvent::Occluded(!visible)`.
  logo minimizada, escondida (cmd-H) ou totalmente coberta por outras
  janelas é `Occluded(true)`; parcialmente visível não é. `request_redraw`
  (`window_delegate.rs:916`) chama `app_delegate.queue_redraw(id)`: é a
  fila do próprio winit, não o `setNeedsDisplay` do AppKit, então uma
  janela coberta que anima recebe `RedrawRequested` a cada pedido e
  desenha numa surface que ninguém vê. `resumed` é emitido uma vez no
  `Init` (`app_state.rs:327`: "for consistency all platforms must emit a
  'resumed' event even though macOS..."); `suspended` nunca
  (`application.rs:125`: só android, ios e web têm ciclo de vida). o sleep
  do sistema não chega por winit: existe como
  `NSWorkspaceWillSleepNotification` e `NSWorkspaceDidWakeNotification`
  nos dois `objc2-app-kit` do lock (0.2.2 do winit, 0.3.2 do tray-icon e
  muda; `src/generated/NSWorkspace.rs`), atrás da feature `NSWorkspace`,
  que nenhum dos dois consumidores liga (a lista do winit vai de
  `NSAppearance` a `NSWindowTabGroup`, sem ela; a do tray-icon é a barra
  de status). e o engine não tem `objc2-app-kit` como dependência direta
  (`rust/engine/Cargo.toml` só o cita num comentário do item 5). ligar a
  feature é mudar o manifesto, arquivo proibido aqui: o gancho fica
  registrado, o item segue com o que o winit dá. o que acontece na tela
  quando o monitor dorme ou a tela trava não foi medido (proibido dormir
  a máquina); o AppKit muda o estado de oclusão nesses casos, mas o
  handoff não afirma o que não mediu.
- iOS: `Occluded(false)` em `applicationWillEnterForeground`,
  `Occluded(true)` em `applicationDidEnterBackground` (`event.rs:408`);
  web: a interseção do canvas com o viewport, sem contar borda, padding e
  transform; android, wayland e windows: não suportado (`event.rs:420`).
  `suspended`/`resumed` de verdade: android, ios e web.

(b) por teste headless (`rust/engine/tests/sleep_wake.rs`, o commit
`7d88e4f3` era a sonda, substituída pelos testes reais em `f4c03f08`):
dois `WindowHost::headless` sob `Windows`, a janela A com
`set_animating(true)` e um deadline, um frame vermelho desenhado.
`route(A, Occluded(true))` respondia `Handled` mas `handle_event` dizia
`false`: o evento não era de ninguém. nada pausava: `needs_redraw()`
seguia true, `request_pending_redraws()` dava 1, `earliest_deadline()`
mantinha o deadline. com o estado mudado durante a cobertura e nenhum
frame desenhado, `read()` devolvia o frame vermelho de antes (a textura
guarda o último frame); o primeiro frame desenhado depois era novo,
porque a cena se constrói do estado, mas ninguém o pedia ao descobrir
(o winit não pede frame no `Occluded(false)`): na tela, uma página
parada mostraria o frame de antes de cobrir até algo pedir um frame.
`Windows::suspend()`: os hosts saem do mapa (`is_empty`,
`has_suspended`), então `request_pending_redraws()` 0 e
`earliest_deadline()` None por ausência, `should_exit()` false, e um
evento para A cai em `Unknown`. o `resume` precisa de um `ActiveEventLoop`
vivo e não foi medido headless.

(c) os trabalhos de fundo dos três apps, e o que acontecia com o app
suspenso ou a janela coberta:

- showcase, Surface: `asking::Claude::start` sobe a thread
  `surface-page-claude` rodando `propose_verified(Llm)` sobre `claude -p`;
  devolve `Msg::Attempt(n)` e `Msg::Done(Answer)` num canal mpsc. não
  acorda o laço: a página olha o canal no `wake` e pede `Wake::after(100 ms)`
  enquanto espera, o que vira o `WaitUntil` do laço. suspenso, o laço
  acordaria a cada 100 ms pelo deadline, o `settle` aplicaria e pediria
  frame, e a janela desenharia coberta.
- showcase, Silences: `worker.rs` sobe `silences-ffmpeg`, um filho por
  vez, `Done::{Detected, Frame}` no canal; igual: polling de 100 ms pelo
  `wake` enquanto `busy()`.
- showcase, Git: o worker do `git` acorda o laço pelo
  `EventLoopProxy` (`UserEvent::Wake`), o shell faz `apply_git` e
  `changed()` (frame se a Git mostra). o resto que acorda o laço: a porta
  do agente (`UserEvent::Agent`), o leitor de tela (`Access`), barra,
  tray e notificação (`Clicked`).
- ide: `GitClient::spawn` acorda o laço por `AppEvent::Git(event)`; o
  shell aplica na hora, serve o agente, grava e `invalidate` (o winit
  enfileira o `RedrawRequested` e a janela desenha, coberta ou não). o
  watcher de disco (`AppEvent::FsChange`) e a porta (`AppEvent::Agent`)
  idem. o ide não usa `Windows` (handoff do item 2): registrado, não
  mudado.
- o `EventLoopProxy` acorda o laço com o app "suspenso" em qualquer
  plataforma (um user event é um evento); o que muda com este item é o
  que o shell faz ao acordar.

### os estados e a regra dos frames

`rust/engine/src/runtime/window/sleep.rs` (163 linhas), mais dois campos no
host (`host.rs` 571) e um no `Windows` (`windows.rs` 323):

- **coberta** é por janela: `WindowHost::occluded`, de
  `WindowEvent::Occluded` em `handle_event` (então `Windows::route`
  entrega; um shell que não roteia chama `set_occluded`). **dormindo** é
  por app: `Windows::asleep`, de `Windows::sleep()`/`wake()` (o shell) e
  de `suspend()`/`resume()` (a plataforma tomou as janelas); cada host
  ganha `sleeping` junto, e um host inserido num app dormindo dorme com
  ele. **acordada**: `WindowHost::awake()` = nem coberta nem dormindo;
  `Windows::awake(id)` para uma janela (false para id desconhecido).
- a regra: um host que não está acordado não pede nada à plataforma.
  `request_redraw` é retido (e ao cobrir o pedido pendente é retirado,
  `redraw_asked` false), `needs_redraw()` é false, `deadline()` é None,
  então `request_pending_redraws` e `earliest_deadline` passam por ele e
  um `Wake` contínuo vira "esperando mostrar" (`is_animating()` fica
  true). um `RedrawRequested` que a plataforma ainda mande não é
  desenhado: o `App` do engine (`lifecycle.rs`, `Routed::Redraw if
  awake`) e o `redraw` do showcase (`pace.rs`) checam `awake`. ao
  descobrir (`Occluded(false)`) ou ao acordar, cada host é invalidado e
  pede um frame: o primeiro frame depois é construído do estado como está
  então, nunca o de antes. a regra do render on demand não muda; é mais
  um motivo para não desenhar.
- os relógios saltam, não pausam. coberta, a view segue: o `catch_up` do
  shell tica com o tempo real quando algo acorda o laço, e um spring que
  teria terminado terminou quando a janela mostra. dormindo, o showcase
  não tica o relógio (`catch_up` e `advance` voltam cedo), então o tempo
  todo dormido cai inteiro no primeiro tick depois do wake, com o mesmo
  efeito. pausar exigiria os relógios da animação (`animation/**`, comps)
  saberem da cobertura, e mostraria a quem volta um floreio do momento em
  que saiu. decidido: saltam.
- `Windows::invalidate_all` e `request_redraw(id)` continuam marcando o
  compositor sujo; só o pedido à plataforma é retido. o `invalidate()`
  basta para o frame ser novo: o passe de composite redesenha toda layer
  na surface a cada frame, e uma textura de layer com conteúdo igual
  continua válida num device vivo; se o device não sobreviveu ao sleep,
  o `recover` da fase 1 (`drop_gpu_resources`, tudo sujo) roda antes do
  acquire desse mesmo frame.

### o contrato dos trabalhos de fundo e como cada app o aplica

`engine::window::Background` (em `sleep.rs`): `Background::sleeping(bool)`
é dito por `Windows::sleep`/`wake` (e por `suspend`/`resume`);
`Background::asleep()` é lido por quem começa um job ou aplica um
resultado. thread-local (`Cell<bool>`): um app por processo, as páginas
rodam na thread do laço, e num binário de teste cada teste é uma thread
com o seu app. o contrato: um resultado que chega dormindo fica onde o
worker o pôs (o canal) e é aplicado no primeiro `wake` depois de acordar;
o shell não faz `settle` nem desenha enquanto dorme, e os `settle` das
páginas não leem nada enquanto `Background::asleep()`; um job pedido
dormindo não começa, ou começa ao acordar, por app. nenhum worker mudou.

- showcase, shell (`app/sleep.rs` 50 de produção, 205 com os testes;
  `app.rs` 633; `pace.rs` 181; `events.rs` 213): `Occluded` vai para
  `covered(id, on)` (o host); `suspended` faz `sleep()`; um `resumed` com
  a janela principal já aberta faz `wake()`: `windows.wake()` (um frame
  novo por janela) e `catch_up()` (as respostas que chegaram dormindo são
  aplicadas, um ask retido começa, o que terminou é contado por
  notificação se ninguém tem o foco). `redraw` não desenha janela que não
  está acordada; `catch_up`/`advance` não ticam nem acordam a view
  dormindo. as surfaces não são soltas no `suspended` do showcase: isso é
  do bring-up mobile, e hoje o showcase não as soltava.
- showcase, Claude (`surface_page/asking.rs` 402; `page.rs` 332 de
  produção): `Claude::start` com `Background::asleep()` devolve um
  `Pending` com `Work::Held(Box<Held>)`, tudo o que a thread levaria
  (`ClaudeCli`, intent, snapshot, caps, surface atual, policy) sem thread;
  `Execution::Started` e o request fica aberto, o botão de propor
  desabilitado, o cancelar habilitado. `Pending::poll` (chamado do
  `settle`) com o app acordado troca `Held` por `Running(rx)` chamando o
  `spawn` (a mesma thread de antes, extraída de `start`), `started` a
  partir daí; dormindo, `(false, None)`. um cancel enquanto retido marca
  `cancelled`, o `dispatch` falha o request "cancelled by ..." e nenhum
  filho nasce. `Page::settle` volta `false` dormindo.
- showcase, ffmpeg (`silences/jobs.rs` 192 de produção): `settle` volta
  `false` dormindo; a resposta espera no canal. o adiar de um job novo
  não entrou: o `detect_silence` e o `preview_segment` submetem em
  `semantics.rs::execute` (`self.worker.submit(...)`), fora dos arquivos
  do item, e guardar o job adiado pede um campo em `SilencesPage`
  (`silences.rs`), também fora. a mudança é uma linha em cada:
  `if Background::asleep() { self.deferred.push(job); return Execution::Started }`
  no `execute`, e `for job in self.deferred.drain(..) { self.worker.submit(job) }`
  no começo do `settle` acordado. hoje não tem efeito observável: o
  desktop não dorme (sem gancho) e fora do desktop não há ffmpeg.
- ide (git): segue, por decisão: o resultado é o estado do repositório,
  dormindo ou não. e o ide não tem `Windows`; nada mudou (registrado).
  urnaui idem.

### os testes e os pixels

- `rust/engine/tests/sleep_wake.rs` (5): janela A animando com deadline,
  `Occluded(true)` roteado: `!needs_redraw`, `!redraw_asked`, `deadline()`
  None, `request_pending_redraws()` 1 (só B), `earliest_deadline` o de B,
  `set_animating`/`invalidate`/`request_redraw` retidos, a mesma palavra
  duas vezes não muda nada; A desenhada vermelha, coberta, o estado vira
  azul sem frame (a textura guarda o vermelho, `frames()` 1),
  `Occluded(false)`: `redraw_asked`, `needs_render`, um pedido, o frame
  desenhado é azul e igual byte a byte ao de um host de controle que
  nunca foi coberto, diferente do anterior, e em repouso nada mais é
  pedido; `Windows::sleep()`: `asleep`, `Background::asleep()`, nenhum
  pedido, deadline None, `invalidate_all` dormindo não pede, dormir duas
  vezes é dormir, um host inserido dormindo dorme, `wake()`: cada host
  acordado, com pedido e cena suja, 3 pedidos, o primeiro frame de A e B
  verde e igual byte a byte ao controle, diferente dos de antes de dormir,
  A segue animando e B descansa; `awake` nas quatro combinações e nas duas
  ordens (coberta e dormindo: acordar o app não basta, descobrir depois
  pede o frame então; `suspend()` dorme e o flag segue), `resume` só
  como `wake()` (precisa do laço); o flag é da thread (outra thread lê
  false).
- `apps/uigallery/tests/sleep_wake.rs` (2, desktop): `ShowcaseView` na
  Todo sobre um host headless 1200x800 sob `Windows`; frame; `sleep()`;
  `todo.item[1]` `toggle` por `Actor::Human` pelo dispatcher, `settle`;
  `invalidate_all` não pede, deadline None, a textura guarda o frame de
  antes; `wake()`: um pedido; o frame igual byte a byte ao de uma segunda
  view no mesmo estado num host que nunca dormiu, diferente do de antes,
  e diferente dentro do rect do item; em repouso de novo. o mesmo com
  `Occluded(true)`/`Occluded(false)` roteados (coberta não é dormindo:
  `Background::asleep()` false).
- `apps/uigallery/src/shell/system/sleep.rs` (2, o shell sobre janela nua com
  `FakeNotifier`, sem foco): uma detecção pedida na Silences termina no
  canal na hora (ffmpeg não é procurado nos testes), o `detect` fica
  desabilitado; `sleep()`, cinco `advance`+`catch_up`: segue desabilitado
  (não aplicado), nenhum frame pedido, ninguém avisado, `changed()` não
  pede; `wake()`: `detect` habilitado, o status fala de ffmpeg, um frame
  pedido, uma notificação enviada, acordar acordado não muda nada. e a
  principal coberta: um toast empurrado e `changed()` não pedem frame,
  setenta `advance(0.1)` sem pedido e sem deadline, o toast viveu o tempo
  dele coberto, descobrir pede um frame, servido, a mesma palavra de novo
  não pede.
- `surface_page/page.rs` (2, desktop, contra o `fake-claude` do
  `claude-model`): `Background::sleeping(true)`, `propose` pelo Claude:
  `Executing`, `waiting()`, `held()`, cinco `wake` sem `now`, nenhum
  `prompt.1` no diretório do fake (o filho nunca rodou);
  `sleeping(false)`, um `wake`: não mais `held`, e o request completa com
  `attempts` 1, `prompt.1` existe, a surface mostrada. e a resposta que
  chega dormindo: `propose` acordado, o filho roda (`prompt.1`), o app
  dorme, três `wake` sem `now` e `Executing`, a surface sem mudar;
  acordado, completa e mostra.
- `silences/jobs.rs` (1): `Worker::fake(hold)`, `detect_silence`,
  `sleeping(true)`, `release()`, `wake`: `!now`, `Executing`, `busy()`;
  `sleeping(false)`, `wake`: `now`, `Completed` com 2 cortes.

### gate e regress

`df -h /` antes do gate: 25 GB livres; 24 GB depois. `./tool/tasks/fullcheck.sh`
verde no worktree com `CARGO_NET_OFFLINE=true`, com a perna wasm e a
perna urnaui (`../urna` presente): antes (o fechamento do item 5, base
`e1386d5a`) 2565 passed, 0 failed, 16 ignored em 122 binários; depois
2577 passed, 0 failed, 16 ignored em 124 binários (+12: 5 de
`engine/tests/sleep_wake.rs`, 2 de `showcase/tests/sleep_wake.rs`, 2 de
`app/sleep.rs`, 2 de `surface_page/page.rs`, 1 de `silences/jobs.rs`; +2
binários, os dois `tests/sleep_wake.rs`). perna urnaui: 85 passed, 0
failed, 1 ignored em 6 binários, igual a antes (o urnaui não usa
`Windows` e compila sem mudança). clippy `-D warnings`, fmt e a checagem
wasm limpos nas duas pernas (o `Work` do `asking.rs` tem uma variante só
fora do desktop; o `match` com o braço em `cfg` compila nos dois). o
único aviso no log é o `block v0.1.6` de incompatibilidade futura, que
já estava.

`cargo run --offline -p regress -- check`: 161
baselines, 161 iguais, 0 mudaram, sem frame desenhado e sem GPU aberta
(840.5 ms). nada deste item muda um frame acordado: os goldens e os
`semantics.json` ficaram como estavam, nenhum bless.

linhas de produção (regra do guard, `file_hygiene_guard` verde): engine
`window/host.rs` 571 (era 551), `window/sleep.rs` 163 (novo),
`window/windows.rs` 323 (era 311), `window/events.rs` 197 (era 194),
`window/lifecycle.rs` 227 (era 225), `window/mod.rs` 161 (era 159);
showcase `app.rs` 633 (era 621; seis linhas do limite, o próximo item que
tocar o shell abre um módulo), `app/sleep.rs` 50 (205 com os testes),
`app/pace.rs` 181 (era 168), `app/events.rs` 213 (era 208),
`view/apps/surface_page/asking.rs` 402 (era 288), `surface_page/page.rs`
332 (era 320; 490 com os testes), `silences/jobs.rs` 192 (era 181; 246
com os testes). tudo abaixo de 639.

### ADR

`docs/adr/rendering/0060-a-covered-window-does-not-draw-and-wakes-new.md` (proposed): os
três estados e quem os entrega em cada plataforma, a regra dos frames e
das deadlines, os relógios que saltam, o contrato do `Background` e como
cada app o aplica, o que o sleep do sistema tem e não tem. linha pro
índice (`docs/adr/README.md` está fora da lista, fica para a integração):

```text
| 2026-09-24 | [a-covered-window-does-not-draw-and-wakes-new](a-covered-window-does-not-draw-and-wakes-new.md) | proposed | task-76 item 7: `WindowHost` sabe quando está coberta (`WindowEvent::Occluded`, no macOS minimizada, escondida ou totalmente atrás; no iOS o background) e quando o app dorme (`Windows::sleep`/`wake`, e `suspend`/`resume`); enquanto uma das duas, `request_redraw` é retido, `needs_redraw` é false e o `deadline` é None, então nenhum frame nem `WaitUntil` por ela e um `Wake` contínuo espera; ao mostrar ou acordar cada host invalida e pede um frame, construído do estado de então, nunca o de antes; os relógios saltam (o catch-up tica com o tempo real); `engine::window::Background` é o mesmo fato para o trabalho ao lado dos frames (thread-local, um app por processo): resultado que chega dormindo fica no canal e aplica no primeiro wake acordado, o Claude da Surface fica retido em `Pending` e começa ao acordar, o git do ide segue; o desktop não ouve o sleep do sistema (a feature `NSWorkspace` do `objc2-app-kit` não está no lock) e o `suspended` de verdade é do bring-up mobile; provado headless com `Occluded` injetado e byte a byte contra um host de controle |
```

### interfaces adicionadas (só crescimento)

- `engine::window::Background::{sleeping, asleep}`;
  `WindowHost::{awake, occluded, sleeping, set_occluded}` (e
  `set_sleeping`, `pub(crate)`); `Windows::{asleep, awake, sleep, wake}`.
  `WindowHost::handle_event` toma `WindowEvent::Occluded` (antes
  `false`); `needs_redraw`, `deadline` e `request_redraw` passaram a
  respeitar `awake` (semântica nova, registrada); `Windows::insert` põe o
  host no sono do app; `suspend` dorme e `resume` acorda.
- showcase: `App::{sleep, wake, covered}` (pub(super));
  `ApplicationHandler::suspended` implementado; `asking::Pending::held`
  (test); o `Msg` e o `Answer` iguais; `Claude::start` com a mesma
  assinatura. nenhuma assinatura pública existente mudou; ide, urnaui,
  wasm e os exemplos compilam sem tocar.

### arquivos tocados fora da lista permitida, e por quê

nenhum. `rust/engine/src/runtime/window/sleep.rs`, `apps/uigallery/src/shell/system/sleep.rs`,
`rust/engine/tests/sleep_wake.rs` e `apps/uigallery/tests/sleep_wake.rs`
são novos dentro dos diretórios permitidos; `rust/engine/src/lib.rs`
não precisou mudar (`window` já é `pub mod`);
`rust/engine/src/runtime/platform/lifecycle.rs` não mudou (o `AppState` do
`LifecycleManager` já tem `Suspended`, e o `Windows` é onde o estado
vive agora).

### limitações e decisões, sem esconder

- nenhuma janela real foi coberta, minimizada ou descoberta nesta
  máquina, e a máquina não dormiu: o que a rodada prova é o engine e o
  showcase com o `Occluded` injetado sobre hosts headless. o caminho do
  winit está lido na fonte, não medido. a prova na tela é do
  orquestrador: `RUST_LOG=engine=debug cargo run --offline -p uigallery --
  Spinner`, minimizar, e ver "window paused (covered true, asleep
  false)" e nenhum "frame:" no log até restaurar, quando aparece "window
  shows again: a new frame" e um frame só, depois o spinner segue.
- o sleep do sistema no desktop não é ouvido (sonda (a)). a app com o
  monitor dormindo segue a cobertura que o AppKit reportar, se reportar;
  não medido. o gancho, quando o manifesto abrir: `objc2-app-kit` com
  `NSWorkspace` (e `objc2-foundation` com `NSNotificationCenter`), um
  observador em `NSWorkspace::sharedWorkspace().notificationCenter()`
  para as duas notificações, chamando `Windows::sleep`/`wake` na thread
  do laço (acordando-o pelo proxy, como o tray faz).
- o `Background` é thread-local, não um valor que o shell passa para as
  páginas: a assinatura do `wake` da view (`view/mod.rs`,
  `view/dispatch.rs`) e os structs das páginas (`proposing.rs`,
  `silences.rs`) estavam fora da lista. um trait `Background` que os
  donos dos jobs implementam e o shell chama é a forma a migrar quando
  esses arquivos abrirem; o flag passaria a ser deles.
- o adiar do ffmpeg não entrou (ver o contrato acima): dois arquivos fora
  da lista, uma linha em cada. sem efeito observável hoje.
- coberta, uma página que faz polling do worker (`Wake::after(100 ms)`)
  perde o deadline: o resultado do Claude ou do ffmpeg é aplicado quando
  outra coisa acorda o laço (input, um deadline de outra janela, o proxy
  de um worker, a porta do agente) ou quando a janela mostra (o frame do
  descobrir tica e faz `settle`). o request fica `Executing` por mais
  tempo enquanto ninguém olha; nada se perde. é o "não acorda a janela
  por conta própria" do briefing, dito.
- a janela da página (`D`) coberta é independente da principal: cada uma
  tem o seu `occluded`; o `view.wake` é um só e tica quando qualquer
  coisa acorda o laço.
- `suspended` no showcase dorme sem soltar as surfaces: no android isso
  é insuficiente (a surface morre com a janela) e é o que já era;
  bring-up.
- o teste da resposta que chega dormindo (`page.rs`) espera 200 ms
  depois do `prompt.1` aparecer para a resposta estar no canal; uma
  máquina muito lenta poderia ver o primeiro `wake` acordado ainda sem
  ela, o que o `pump` de 10 s cobre.

### o que fica para o bring-up e o que os itens 9 e 10 reaproveitam

- bring-up: o gancho de sleep do sistema no macOS (acima); o `suspended`
  do android soltando as surfaces pelo `Windows::suspend` (o engine `App`
  já faz; o showcase não); o `Occluded` do iOS como o background (já
  cai em `set_occluded`, não medido num aparelho); o adiar do ffmpeg;
  linux e windows sem `Occluded` (wayland e windows: não suportado no
  winit) ficam só com o sleep por shell.
- item 9 (impressão): um frame de impressão é um `WindowHost::headless`
  desenhando o estado como está, o mesmo caminho que o teste de controle
  usa aqui (`control()`): a janela pode estar coberta ou o app dormindo
  e a impressão não depende de `awake`, porque um host headless não é
  coberto nem pede frame à plataforma; e `Background::asleep()` diz se
  um job de impressão em fundo deve esperar o wake (um spool é um job
  como o ffmpeg).
- item 10 (câmera e microfone): a captura é um trabalho de fundo com o
  contrato daqui: enquanto o app dorme, os frames capturados ficam no
  canal (ou a captura pausa, decisão do item) e a textura só é atualizada
  no primeiro wake acordado; uma janela coberta não deve manter a câmera
  desenhando, e `Windows::awake(id)` é a pergunta; o `Occluded(true)` do
  iOS (background) é onde o sistema exige soltar a captura.

### comandos de validação

```text
export CARGO_TARGET_DIR=/Users/nn/Dev/hoff/plevdev/target/agents/one
export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_PROFILE_TEST_DEBUG=line-tables-only
cargo test --offline -p engine --test sleep_wake
cargo test --offline -p uigallery --test sleep_wake
cargo test --offline -p uigallery --lib -- app::sleep silences::jobs surface_page::page
cargo test --offline -p engine -p uigallery
./tool/tasks/fullcheck.sh
cargo run --offline -p regress -- check
RUST_LOG=engine=debug cargo run --offline -p uigallery -- Spinner   (minimizar e restaurar; do orquestrador)
```

## handoff do item 9: impressão, o frame no papel

branch `task/76h-print`, base `f92e56d1` (main após o item 7). worktree
`.claude/worktrees/agent`, target `target/agents/one`. commits, em ordem:

- `3e13a7f1` task-76: the print probe
- `9c33b535` task-76: the printer behind a trait, with the fake and CUPS
  as a child (o trait, o falso, o CUPS por filho e o documento a partir
  do render, `PrintDocument::from_frame` e `from_headless`, num commit só:
  a assinatura do trait pede o documento)
- `84a95b34` task-76: the entry points move to app/entry.rs (só
  movimento, antes de tocar o `app.rs`: ver "arquivos tocados")
- `dd14f99b` task-76: the showcase prints the page: shell.print, cmd-p and
  the job beside the loop
- (este) task-76: ADR and the item 9 handoff

### a sonda

(a) esta máquina, 2026-09-24, sem impressora (nenhuma foi adicionada):

```text
$ lpstat -p -d
lpstat: No destinations added.          (stderr)
no system default destination           (stdout)
exit 1, 0,14 s
$ lpstat -a
lpstat: No destinations added.          exit 1
$ lpstat -r
scheduler is running                    exit 0
$ lpstat -o
(nada)                                  exit 0
$ lpstat -W not-completed -o Office-42
lpstat: Invalid destination name in list "Office-42".   exit 1
$ lp tiny.png
lp: Error - No default destination.     exit 1
$ lp -d Nowhere -t "plev test" -o fit-to-page tiny.png
lp: No such file or directory           exit 1 (é sobre o destino; o arquivo existia)
$ lp -d Nowhere /nonexistent.png
lp: Error - unable to access "/nonexistent.png" - No such file or directory   exit 1
$ lpoptions
(nada)                                  exit 0
```

`man lp`: `-d destino`, `-n cópias`, `-t título` (o nome do job na fila),
`-o media=a4|letter|legal`, `-o fit-to-page` (a opção genérica do CUPS que
escala o documento ao papel; o `man` desta versão lista `media`,
`number-up`, `orientation-requested`, `sides`, `print-quality` e
`job-sheets`), `--` antes do arquivo, e o `request id is <dest>-<n> (1
file(s))` na saída. sem `-d`, `LPDEST`/`PRINTER`, o `lpoptions -d` e o
`lpadmin -d`, nessa ordem. `man lpstat`: `-p` impressoras, `-d` o padrão,
`-W completed|not-completed` antes de `-o`, e `-o [destino(s)]` os jobs de
um destino: o `-o <job-id>` não existe (a sonda acima), então o status de
um job se lê na fila do destino dele (`Office-42` é o job 42 de `Office`),
uma linha que começa com o id.

(b) a página Todo sem janela em duas escalas, pelo `showcase --render`
(binário debug, `target/agents/one`):

| escala | físico | png | wall frio | wall quente |
|---|---|---|---|---|
| 2,0 (tela retina) | 2400x1600 | 231 289 bytes | 1,16 s | 0,11 a 0,12 s |
| 4,1667 (papel) | 5000x3333 | 704 377 bytes | 0,51 s | 0,29 a 0,46 s |

a conta: 1 pixel lógico = 1 ponto = 1/72 pol, então a escala de `d` dpi é
`d / 72`; a 300 dpi, `300 / 72 = 4,1667`, e a janela de 1200x800 vira
5000x3333 dots, 16,7 x 11,1 pol (mais largo que um A4 deitado, 11,69 pol;
o `fit-to-page` reduz ao papel, o que ainda dá uns 428 dpi efetivos). o
limite do device: o engine pede `wgpu::Limits::default()` nos dois
construtores (`gpu/context.rs:295`), então `max_texture_dimension_2d` é
8192 em qualquer máquina; 5000 cabe, e `fit_scale` escolhe a maior escala
que cabe quando não (uma janela de 2000 de largura imprime a 4,09). o
atlas de glifos não estourou em nenhuma das duas escalas.

por teste (`rust/engine/tests/print.rs`, `--nocapture`), um run de
texto de 15 px em 600x80 lógicos, cada peça medida:

```text
screen 2.0:   1200x160 px, device 176.5 ms, draw+read 86.4 ms, png 1.6 ms (18900 bytes), ink 585 px    (frio: o primeiro device e os pipelines)
paper 4.1667: 2500x333 px, device   9.7 ms, draw+read  7.7 ms, png 6.8 ms (53429 bytes), ink 1219 px   (quente)
```

a tinta do mesmo run vai de 585 a 1219 px: razão 2,084 para uma razão de
escalas de 2,083. os glifos são rasterizados na escala física (o atlas
da task-66), não um bitmap ampliado: o texto continua nítido. o png de
cada escala está em `scratchpad/probe/todo-2.png` e `todo-4.1667.png`, e
os recortes da mesma região lógica (a linha "Absorb the plev demos into
the showcase") em `crop-2.png` (640x150) e `crop-4.png` (1334x313): a
mesma diagramação, bordas limpas nos dois.

(c) o que `engine::headless::svg` já entrega, no Todo: 397 `<path d=>` e
zero `<text>`: texto como contorno de glifo dentro de `<g aria-label>`,
rect e rounded rect exatos, gradiente como `linearGradient` (mistura em
linearRGB, que nem todo viewer honra), clip como `clipPath` em bounds
lógicos (o frame corta em pixel físico inteiro), path tesselado como
triângulos (pode mostrar costura). 8 aproximações relatadas no Todo
(218 115 bytes de svg, 0,14 s). para um PDF vetorial sem dependência
faltaria: um escritor de PDF (objetos de página, um content stream com os
mesmos paths, sem fonte porque o texto já é contorno), a conversão de
`clipPath` e gradiente para operadores PDF, e o sRGB/linear acertado.
registrado como bring-up; o `PrintDocument::with_svg` guarda o svg ao lado
do png para esse caminho.

### o trait e o que cada alvo faz

`rust/engine/src/runtime/platform/print/mod.rs` (470 linhas de produção, 645 com os
testes) e `platform/print/cups.rs` (417, 563 com os testes), pelo molde
do `alerts.rs` e do `notify.rs`: trait, real sob o cfg, falso em todo
alvo, nenhum em wasm e mobile, `default_printer()` escolhendo.

- `Destination { name, default }`; `PrintOptions { destination:
  Option<String>, copies, title, fit_to_page, media: Option<String> }`
  com `Default` (1 cópia, "plev", fit, sem media) e `.to()`, `.title()`,
  `.copies()` (nunca menos de 1), `.fit_to_page()`, `.media()`;
  `PrintDocument { png, width, height, dpi, svg: Option<String> }` (o
  `Debug` mostra tamanhos, nunca os bytes), `from_frame(&Frame, dpi)`,
  `from_headless(&mut Renderer, &mut Compositor, clear_linear, dpi)`,
  `with_svg`, `paper_size()` em polegadas; `paper_scale(dpi)` e
  `fit_scale(w, h, scale, max_side)`; `POINTS_PER_INCH = 72`, `PAPER_DPI
  = 300`. `JobId(String)`; `JobStatus::{Queued, Completed, Gone}`;
  `PrintError::{Unsupported, NoDestination, UnknownDestination(String),
  Spooler(String), Timeout { program, secs }, Io(String)}`; `Shown<T>`.
- `trait Printer { destinations() -> Shown<Vec<Destination>>;
  print(&PrintDocument, &PrintOptions) -> Shown<JobId>; status(&JobId) ->
  Shown<JobStatus> }`. cada chamada pode bloquear (um filho): o app chama
  de uma thread ao lado do laço, nunca do `execute`.
- `CupsPrinter` (unix desktop: macOS e linux): `new()` lê `PLEV_LP` e
  `PLEV_LPSTAT` ou fica com `lp` e `lpstat` do PATH (nada roda ao
  construir); `with(lp, lpstat)`, `.timeout(d)` (20 s por padrão),
  `.spool_dir(dir)` (`$TMPDIR/plev-print` por padrão, 0o700). cada
  chamada é um filho com prazo, `try_wait` a cada 10 ms, morte pelo pid
  e `wait` (reap), stdout e stderr drenados em threads (um neto segurando
  o pipe não segura o chamador), 1 s de graça pro fim da saída: a forma
  do `worker.rs`, reescrita e não dependência. `destinations()`: `lpstat
  -p -d`, uma `Destination` por linha `printer <nome> ...`, o padrão pela
  linha `system default destination: <nome>`; exit 1 com "No
  destinations added" é lista vazia, qualquer outra falha é
  `Spooler(última linha)`. `print()`: escreve o png em
  `<spool>/plev-<pid>-<n>.png` (0o600), roda `lp [-d dest] [-n cópias]
  -t título [-o fit-to-page] [-o media=x] -- arquivo`, remove o arquivo
  (o `lp` copiou pro spool, ou nunca pegou), lê `request id is X`;
  stderr "No default destination" vira `NoDestination`, "No such file or
  directory" sem "unable to access" vira `UnknownDestination(dest)` (é o
  que o `lp` diz de destino desconhecido, sonda (a)), o resto `Spooler`.
  `status(job)`: destino = tudo antes do último `-`; `lpstat -W
  not-completed -o <dest>` lista o id? `Queued`; senão `-W completed`
  lista? `Completed`; senão `Gone`; um destino que o `lpstat` recusa é
  `Spooler`.
- `FakePrinter`: `.destination(nome, default)`, `.will_refuse(err)` (a
  próxima impressão que chega ao spooler, isto é, com destino escolhido),
  `.will_report(status)`, `printed()`/`take_printed()` com `Printed {
  document, options, job }`; sem destino recusa `NoDestination`, destino
  desconhecido `UnknownDestination`, job impresso é `Queued`, outro é
  `Gone`; ids `<dest>-<n>`.
- `NoPrinter`: `Unsupported` nas três. é o windows até o bring-up (sem
  `lp`, e nenhuma dependência aqui), o wasm e o mobile.
- `impl Printer for Arc<Mutex<P>>` e `for Box<P>`: a thread do job segura
  uma ponta, o teste a outra. `default_printer() -> Box<dyn Printer +
  Send>`.

### o documento: escala, DPI, bytes, tempo

no showcase, a escala é `fit_scale(width, height, paper_scale(300),
8192)`: 4,1667 na janela de 1200x800 (5000x3333, 300 dpi); numa janela
mais larga que 1966 lógicos a escala cai pra caber e o dpi do documento
segue (`scale * 72`). o png sai com 700 460 a 704 377 bytes no Todo (a
diferença é o item marcado). o teste de integração mede o job inteiro na
thread dele: device 16 a 48 ms, render + readback + png 224 a 356 ms,
spool no falso 30 µs; a cena construída na thread do laço 71 µs quente e
24 ms frio (o primeiro layout da página naquela escala). o png não leva
chunk de dpi (o `image::codecs::png::PngEncoder` do lock não expõe o
`pHYs` sem mais código, e o `fit-to-page` escala ao papel de qualquer
jeito); o dpi vai no receipt e no `PrintDocument`.

### o trabalho de fundo

decidido: a cena na thread do laço, o resto fora. a view mora na thread
do laço e não é `Send`; o `Compositor` é (compilou sem `unsafe` e sem
wrapper), então `render_for_print(scale)` constrói a cena da janela
principal num compositor próprio (o `layers` e o `scale_factor` da view
são trocados e postos de volta: o único uso do `scale_factor` na cena é o
clip físico da ScrollArea, `render.rs:98`) em microssegundos, e a thread
`print-job` abre um `engine::headless::Renderer` próprio (device
próprio; o mesmo caminho do `regress`, só que fora do laço), desenha,
lê, codifica, entrega ao `Printer` e pergunta o status uma vez. custo
medido acima: o laço nunca espera mais que a construção de uma cena, e o
spool não toca o laço. a alternativa (render síncrono no laço com
contexto próprio) custaria 240 a 400 ms parados por impressão, mais de
um frame, e foi descartada.

`Execution::Started` no `execute` do shell, `complete`/`fail` no
`settle_print` (chamado do `advance` do `pace.rs`, o mesmo `catch_up` que
aplica o ffmpeg e o Claude), com `job`, `destination`, `status`, `width`,
`height`, `dpi`, `bytes`, `seconds` no outcome. sob `Background`: pedido
dormindo fica em `held` e começa no primeiro `settle_print` acordado (a
cena é construída aí, do estado de então); resposta que chega dormindo
fica no canal. os destinos também são um job (`print-destinations`),
disparado no `set_printer`, então o `resumed` não espera o `lpstat` (0,14
s aqui; 20 s no pior caso de um scheduler travado) e o botão liga quando
a resposta cai. um job por vez: o botão desabilita enquanto um roda.

### o showcase: o botão, o menu, a projeção

`apps/uigallery/src/views/dispatcher/shell/print.rs` (528 de produção),
`shell.rs` (538), `app/print.rs` (62, 189 com os testes), `app/menu.rs`
(253), `app/pace.rs` (188), `app/events.rs` (219), `app.rs` (567),
`app/entry.rs` (95, movimento).

- `shell.print` (Button, "print this page"), `PERSON_ONLY` (human,
  assistive, test; agente recusado `NotAuthorized` antes do `execute`),
  `enabled` só com destino conhecido e sem job em curso, `focusable`
  false; estado `destinations` (List), `destination` (o padrão, ou o
  primeiro), `job`, `status` (do último). `shell:print`, uma linha pro
  agente: "no destination", "ready on Office", "printing on Office",
  "Office-7 on Office: queued", "failed on Office: <por quê>".
- cmd-p (ctrl-p fora do macOS; `PRINT_KEYS`, `is_print_key`: com shift
  não é) em `events.rs`, logo depois das teclas de gravação:
  `App::print_key` -> `ShowcaseView::request_print` ->
  `ask_shell_receipt(Print, "activate")` (novo, devolve o receipt; o
  `ask_shell` de sempre chama ele). o item "print this page" com o atalho
  no menu "view", depois de "page in a window of its own" (e no menu do
  tray, que reusa `view_menu`), desabilitado sem destino: `MenuItem::of`
  lê o `enabled` da projeção. `install_print()` no `resumed`, depois das
  notificações: `view.set_printer(default_printer())`.
- nesta máquina: `lpstat` responde sem destino, a lista fica vazia, o
  botão e o item ficam desabilitados e `shell:print` diz `no
  destination`. nada foi impresso.

### testes

- `rust/engine/src/runtime/platform/print/mod.rs` (8): escala e `paper_size`,
  `fit_scale`, as opções, o `Debug` do documento, o falso (padrão e
  nomeado, `printed`, status), as recusas do falso e os textos dos
  erros, o compartilhado por `Arc<Mutex>`, `NoPrinter` e
  `default_printer()` construído sem rodar nada. `cups.rs` (6): os
  programas por env ou PATH, o parse do `lpstat -p -d` (com o padrão, o
  vazio desta máquina, sem padrão, a falha do scheduler), os argumentos
  do `lp`, o id do job e o destino dele, a listagem de um job, as
  recusas do `lp` como esta máquina as deu.
- `rust/engine/tests/print.rs` (7): a sonda (b), o documento a partir do
  render (tamanho do frame, dpi, png que decodifica pros mesmos bytes do
  `read()`, `paper_size`, o svg ao lado), e o CUPS sobre `tests/fake-lp`,
  `tests/fake-lpstat` (Office padrão e Home; Office-7 na fila, Home-7
  completo) e `tests/fake-lpstat-none` (esta máquina): destinos com o
  padrão marcado, lista vazia e `NoDestination` sem `-d`, o comando
  montado (`-d Office -n 2 -t "plev, Todo" -o fit-to-page -o media=a4 --
  <spool>/plev-*.png`, lido do `lp.args` que o falso deixa ao lado do
  documento, e o documento removido depois), `Queued`/`Completed`/`Gone`
  e o destino inválido, as recusas em palavras do `lp`, o programa que
  não existe, e o prazo: `fake-lp -t slow` vira `sleep 30`, o prazo de 2
  s mata pelo pid (`kill -0` falha depois) e o documento sai do spool
  também. nenhum `lp` nem `lpstat` real roda.
- `apps/uigallery/src/views/dispatcher/shell/tests.rs` (+1, e as listas de
  ids com `shell.print`): a projeção sem destino (desabilitado, estados
  vazios, `no destination`), o pedido da pessoa recusado `Disabled`, os
  destinos do falso caindo pela thread e habilitando (o padrão é o
  destino), o agente `NotAuthorized` e nada impresso, o `NoPrinter`
  deixando `no destination`.
- `apps/uigallery/src/shell/system/print.rs` (2): as teclas; a barra com o item,
  desabilitado, com `cmd-p`, as teclas pedindo e recusadas, um `p` solto
  é da página, e o item habilitando depois do `set_printer` e do
  `sync_menu`.
- `apps/uigallery/tests/print.rs` (2, GPU, pula sem adapter): a pessoa
  imprime o Todo com um item marcado pelo falso: `Executing`, `printing
  on Office`, botão desabilitado, o job responde, o receipt `Completed`
  com `Office-1`, 5000x3333, 300 dpi, bytes e segundos; o falso recebeu
  um documento com o tamanho da página, `-d Office`, "plev, Todo", 1
  cópia, fit; o png decodificado é, byte a byte, o frame que uma view no
  mesmo estado desenha a 4,1667 por um `Renderer` (o controle); a
  projeção depois (`Office-1 on Office: queued`, `job`, `status`,
  habilitado de novo); a cena da janela principal igual nó a nó e o
  `scale_factor` 1,0. e o pedido dormindo (`Background::sleeping(true)`):
  retido, nada roda em 250 ms, acorda e imprime.

### gate e regress

`df -h /` antes do gate: 21 GB livres; 21 GB depois. `./tool/tasks/fullcheck.sh` verde
no worktree com `CARGO_NET_OFFLINE=true`, com a perna wasm e a perna
urnaui (`../urna` presente): antes (o fechamento do item 7, base
`f92e56d1`) 2577 passed, 0 failed, 16 ignored em 124 binários; depois
2603 passed, 0 failed, 16 ignored em 126 binários (+26: 14 de
`platform::print` e `print::cups`, 7 de `engine/tests/print.rs`, 3 no lib
do showcase, `dispatch::shell` +1 e `app::print` 2, e 2 de
`showcase/tests/print.rs`; +2 binários, os dois `tests/print.rs`). perna
urnaui: 85 passed, 0 failed, 1 ignored, igual a antes (o urnaui não toca o
`platform::print` e compila sem mudança). clippy `-D warnings`, fmt e a
checagem wasm limpos nas duas pernas; o único aviso no log é o `block
v0.1.6` de incompatibilidade futura, que já estava. antes do gate,
`cargo test --offline -p engine -p uigallery`: 1208 passed, 0 failed, 7
ignored.

`cargo run --offline -p regress -- check`: 161
baselines, 161 iguais, 0 mudaram, sem frame desenhado e sem GPU aberta
(702,6 ms). o shell não está no frame de nenhuma página e
`ShowcaseView::semantics` não mudou: os goldens ficaram como estavam,
nenhum bless.

linhas de produção (regra do guard, `file_hygiene_guard` verde): engine
`platform/print.rs` 470 (novo), `platform/print/cups.rs` 417 (novo);
showcase `app.rs` 567 (era 633; o movimento levou a 561), `app/entry.rs`
95 (novo, movimento), `app/print.rs` 62 (novo), `app/menu.rs` 253 (era
234), `app/pace.rs` 188 (era 181), `app/events.rs` 219 (era 213),
`view/dispatch/shell.rs` 538 (era 497), `view/dispatch/shell/print.rs`
528 (novo). tudo abaixo de 639.

### ADR

`docs/adr/rendering/0061-printing-is-the-frame-on-paper.md` (proposed): o documento como
render do mesmo frame numa escala de papel, o CUPS como filho, o que é só
pessoa e trabalho de fundo, o svg como caminho pro PDF, o que fica pro
bring-up. linha pro índice (`docs/adr/README.md` está fora da lista, fica
para a integração):

```text
| 2026-09-24 | [printing-is-the-frame-on-paper](printing-is-the-frame-on-paper.md) | proposed | task-76 item 9: `engine::platform::print` põe "imprimir esta página" atrás de um trait (`Printer`: destinos, imprimir um `PrintDocument` com destino, cópias, título, fit-to-page e media, status do job); o documento é o mesmo frame da janela renderizado pelo caminho headless numa escala de papel (300 dpi sobre 72 pontos = 4,1667: a janela de 1200x800 vira 5000x3333, glifos nítidos porque o atlas rasteriza na escala física), png para o CUPS, o svg do headless ao lado como caminho para PDF vetorial; `CupsPrinter` roda `lp` e `lpstat` como filhos com prazo e morte pelo pid, sem dependência, `FakePrinter` guarda o que recebeu, `NoPrinter` é windows até o bring-up, wasm e mobile; no showcase `shell.print` (só pessoa, cmd-p e o menu) constrói a cena na thread do laço e faz o render, o png e o spool numa thread própria sob `Background`, com `shell:print` projetado para o agente; sem destino a projeção diz e o botão desabilita; provado com os falsos, byte a byte contra um render de controle, nenhum `lp` real |
```

### interfaces adicionadas (só crescimento)

- `engine::platform::print::{Destination, PrintOptions, PrintDocument,
  JobId, JobStatus, PrintError, Shown, Printer, FakePrinter, Printed,
  NoPrinter, CupsPrinter, default_printer, paper_scale, fit_scale,
  POINTS_PER_INCH, PAPER_DPI, LP_ENV, LPSTAT_ENV}`. nada do `headless`
  mudou: `Frame::encode_png` e `Renderer::render` já eram públicos.
- showcase: `ShowcaseView::{set_printer, request_print, settle_print,
  ask_shell_receipt}` (as duas primeiras desktop), `ShellTarget::Print`;
  `App::{install_print, print_key}` (pub(super)),
  `app::print::{PRINT_KEYS, is_print_key}`; `app::entry::{run, run_web,
  android_main, showcase_ios_main}` reexportados de `app`. a projeção da
  galeria (`gallery_semantics`) ganhou o nó `shell.print` e o estado
  `shell:print`; `ShowcaseView::semantics` (a da página) não mudou: os
  161 baselines e o golden do todo ficaram iguais.

### arquivos tocados fora da lista permitida, e por quê

nenhum. `apps/uigallery/src/shell/mod.rs` estava em 633 linhas e recebeu o
`mod print;` (com doc) e o `install_print()`: o primeiro commit do
showcase (`84a95b34`) foi só movimento, os quatro entry points de
plataforma (`run`, `run_web`, `android_main`, `showcase_ios_main`) para
`app/entry.rs`, palavra por palavra, reexportados; `app.rs` foi a 561 e
está em 567. `app/events.rs`, `app/menu.rs`, `app/pace.rs` são `app/**`.
`apps/uigallery/src/views/dispatcher/mod.rs` (fora da lista) não mudou: por
isso o tipo `Last` do `shell/print.rs` não é reexportado e a projeção
carrega o último job.

### limitações e decisões, sem esconder

- nada foi impresso nesta máquina, e nenhum `lp` real rodou em teste: o
  `CupsPrinter` foi provado contra os scripts falsos, que reproduzem as
  saídas medidas na sonda (a). o formato das linhas do `lpstat -p` e do
  `lpstat -o` com impressora de verdade vem da memória do CUPS e dos
  falsos, não desta máquina; o parser lê só o segundo token de `printer
  ...` e o começo de linha de um job, o que é robusto a variações de
  data e estado. a prova real é do brenner (roteiro abaixo).
- o status é perguntado uma vez, logo depois do spool; ninguém faz poll
  da fila depois, então um job que completa fica "queued" na projeção
  até a próxima impressão. um poll (um `Wake::after`) é uma linha no
  `settle_print` quando for pedido.
- o que vai pro papel é o frame da janela principal inteiro (chrome e
  página), no tamanho lógico da janela; conteúdo que rola imprime o que
  a viewport mostra. paginação fica registrada. a janela da página (`D`)
  não é o que imprime.
- o `render_for_print` troca `layers` e `scale_factor` da view e põe de
  volta: é o único jeito sem tocar `view/render.rs` (fora da lista). o
  método certo, quando o arquivo abrir, é um `render_at(c, scale)` na
  view que não guarde os layers em `self`.
- a thread do job abre um device wgpu por impressão (16 a 48 ms) e
  fecha; não há renderer retido. imprimir é raro.
- `PrintOptions.media` e `copies` existem no trait e no `lp`, mas o
  showcase imprime sempre 1 cópia no media da impressora com
  `fit-to-page`: escolher é do painel nativo (bring-up).
- `Timeout.secs` é o prazo em segundos inteiros: um prazo de 300 ms diz
  "0 s". o teste do prazo usa 2 s por isso e porque, ao lado dos testes
  de GPU do mesmo binário, um filho levou mais de 300 ms só pra começar
  (a primeira rodada falhou assim; sozinho passou em 0,32 s).
- linux: o `CupsPrinter` assume o cups instalado; sem ele o `lp` não
  existe e o `Io("could not start lp")` aparece na projeção como
  "failed on ...". windows: `NoPrinter`.
- notificação: um job que termina com o app sem foco não é contado por
  notificação (o `JobSink` do `notify.rs` ouve só `detect_silence`);
  `shell:print` e o receipt dizem. uma linha no `JobSink` resolve, se
  quiserem.

### o que fica para o bring-up e o que o item 10 reaproveita

- bring-up: o windows com o spooler dele (ou `NoPrinter` até lá); o PDF
  vetorial a partir do svg (texto já em contorno); o painel nativo
  (`NSPrintPanel`, escolhendo media, sides e cópias como `-o` do `lp`),
  outra representação da mesma capacidade com a janela pai do item 2;
  paginação; o poll da fila; o chunk de dpi no png.
- item 10 (câmera e microfone): a captura é o mesmo molde: trait, real,
  falso, nenhum, `default_*()`; um trabalho de fundo com canal, começado
  e aplicado sob `Background` (não começa dormindo, resposta espera o
  wake), com `Execution::Started` e `complete`/`fail` no `settle` do
  `catch_up`; e o `Scene`/`Renderer` próprio da thread mostra que um
  device próprio numa thread ao lado do laço funciona (um frame
  capturado vira textura no device da janela só no `wake` acordado, como
  o item 7 registrou). a permissão do sistema (câmera, microfone) é a
  diferença: pede `Unsupported` sem ela e um estado projetado
  ("permission denied") em vez de falhar, como o `no destination` aqui.

### o roteiro da prova real (do brenner, nunca de teste)

sem impressora física, o CUPS do macOS aceita um destino que imprime em
PDF; nada disto foi feito aqui (a máquina fica sem destino):

```text
lpadmin -p pdf -E -v cups-pdf:/ -m everywhere      (ou o "Save as PDF" de uma fila IPP; ou uma impressora real por System Settings)
lpstat -p -d                                         (o destino aparece; "system default destination: pdf" com lpadmin -d pdf)
RUST_LOG=showcase=info cargo run --offline -p uigallery -- Todo
   cmd-p, ou view > print this page: o log diz "print: the Todo page's scene built for paper at scale 4.167 (300 dpi) in ..." e
   "print: pdf-<n> queued: 5000x3333 at 300 dpi, ... bytes; device ..., render and png ..., spool ..."
lpstat -W completed -o pdf                            (o job listado; o arquivo onde o driver escreve, ~/Desktop ou /var/spool/cups-pdf)
PLEV_AGENT=1 ... e `plev observe`: shell:print = "pdf-<n> on pdf: queued" e shell.print:job
lpadmin -x pdf                                        (o destino sai; a máquina volta a "No destinations added")
```

o que olhar no papel: o frame inteiro da janela (sidebar, cabeçalho,
página) na folha, escalado pelo `fit-to-page`, texto nítido de perto.

### comandos de validação

```text
export CARGO_TARGET_DIR=/Users/nn/Dev/hoff/plevdev/target/agents/one
export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_PROFILE_TEST_DEBUG=line-tables-only
cargo test --offline -p engine --lib platform::print
cargo test --offline -p engine --test print -- --nocapture          (a sonda (b) imprime as medidas)
cargo test --offline -p uigallery --lib -- dispatch::shell app::print app::menu
RUST_LOG=showcase=info cargo test --offline -p uigallery --test print -- --nocapture --test-threads=1
cargo test --offline -p engine -p uigallery
./tool/tasks/fullcheck.sh
cargo run --offline -p regress -- check
cargo run --offline -p uigallery -- --render Todo --scale 4.1667 --out todo-paper.png   (o png do papel, sem impressora)
lpstat -p -d                                                          (esta máquina: No destinations added)
```

### integração do item 9 (orquestrador, 2026-09-24)

- merge da `task/76h-print` na `main` com um commit a mais no branch:
  `task-76: a print thread that is out keeps the loop looking until it
  answers`. o job e a busca de destinos respondiam num canal que nada
  acordava o laço para ler; agora `ShowcaseView::print_wake` pede uma
  olhada a cada 100 ms enquanto uma thread está fora, somado em
  `App::frame_wake`. teste novo em `app/print.rs`.
- o `summary()` perdeu o `let _ = id` que só existia para usar a
  variável.
- fica para depois: o poll do status da fila depois do spool (a
  projeção fica em "queued"), paginação, PDF vetorial, painel nativo.

## handoff do item 10: câmera e microfone

branch `task/76i-capture`, base `9ea37ea9` (main após o item 9).
worktree `.claude/worktrees/agent`, target `target/agents/one`. commits,
em ordem:

- `6d0d1c53` task-76: the capture probe
- `ab221e79` task-76: a stream image changes its pixels in place
- `1331b644` task-76: camera and microphone behind a trait, with ffmpeg
  as a child
- `f9c678e2` task-76: the Capture page, camera and microphone as a
  person's toggles
- `2644e032` task-76: covered or asleep, the shell releases the camera at
  once
- `824ee8ab` task-76: the captured frame is on screen in the first frame
  after the wake
- `de4a4d98` task-76: the camera drops frames before it scales them
- (este) task-76: ADR and the item 10 handoff

a rodada parou uma vez, depois da sonda: trocar os pixels de um
`ImageHandle` exigia `engine/src/gpu/image.rs`, fora da lista. o
orquestrador liberou a opção 1 (`gpu/image.rs` e uma linha de
`gpu/mod.rs`), com a API e cinco condições; o commit `ab221e79` é ela, e a
seção "a imagem de fluxo" diz como cada condição foi cumprida. os testes
de cada comportamento saíram no commit dele; `824ee8ab` é o teste de
integração com pixels.

nenhum dispositivo foi aberto nesta máquina, em sonda, teste ou gate: o
ffmpeg só leu `-f lavfi` (`testsrc`, `sine`), nenhum `-f avfoundation`
com entrada, nenhum `-list_devices`, nenhuma chamada a AVFoundation. o
`fake-ffmpeg` recusa (sai 2) se sobrar uma entrada de dispositivo depois
do `-i`.

### a sonda

(a) o pipe, com `-re` (o lavfi no ritmo de um dispositivo), medido em
python e depois nos testes (`rust/engine/tests/capture.rs`,
`--nocapture`):

```text
camera 640x360 at 15: 921600 bytes a frame, first frame 32.7 a 63.7 ms, 15.04 a 15.08 fps depois do burst
   (sem -re o testsrc sai a 3726 fps: o fake precisa do -re; com ele o primeiro meio segundo sai de uma vez, o readrate_initial_burst)
microphone 16000 Hz mono f32: first bytes 17.0 a 55.7 ms, leituras de 4096 a 24576 bytes (mediana 4096),
   16 555 a 16 921 amostras/s em janelas curtas depois do burst, RMS de 100 ms 0.0884 (o sine do lavfi tem amplitude 1/8)
load_image_rgba 640x360: 0.71 a 0.84 ms; draw e readback headless: 5.6 a 15.8 ms o primeiro (device), 1.5 a 7.2 ms depois
replace_image_rgba 640x360 (depois da decisão): 0.02 a 0.17 ms
```

e o achado que parou a rodada: cada frame carregado como imagem fixa é
um handle novo e um upload novo que o store guarda para sempre. 244
frames de 640x360 encheram o atlas (1024 até o frame 3, 2048 até o 14,
4096 até o 57, 8192 depois), 225 MB no log, 16,3 s de uma câmera a 15
fps; o seguinte volta `TooLarge`, para toda imagem do app (o store é
global). o teste ignorado `a_stream_of_frames_fills_the_grow_only_atlas`
mede isso de novo.

(b) a entrada real por plataforma, só lendo (`man ffmpeg-devices`, `ffmpeg
-hide_banner -h demuxer=avfoundation`, `ffmpeg -devices`, que só lista o
que foi compilado, e as strings de `libavdevice.63.1.101.dylib`):

- macOS: `-f avfoundation -i "<vídeo>:<áudio>"`, por índice ou nome,
  `none` para o outro lado e `default` para o padrão;
  `-video_device_index`/`-audio_device_index` sobrepõem o nome.
  `-pixel_format` padrão yuv420p (um formato que a câmera não tem é
  trocado com aviso, "Overriding selected pixel format"), `-framerate`
  padrão ntsc (29,97). o que o engine usa: `-f avfoundation -i 0:none` e
  `-i none:0`, sem `-video_size` nem `-framerate` na entrada (decisão do
  orquestrador), e a saída `-vf fps=15,scale=640:360:force_original_aspect_ratio=increase,crop=640:360
  -f rawvideo -pix_fmt rgba pipe:1` e `-f f32le -ac 1 -ar 16000 pipe:1`.
  este ffmpeg (homebrew 9.0.1) tem só `avfoundation` e `lavfi` como
  dispositivos de entrada.
- windows: `-f dshow -i video="<nome>"` e `audio="<nome>"`; o dshow abre
  por nome, e o índice só desempata nomes iguais (`video_device_number`).
  sem nome o engine responde `NoDevice("dshow opens a camera by its
  name, and none was given")`: a lista é do bring-up.
- linux: `-f v4l2 -i /dev/video<n>` e `-f alsa -i default` (ou
  `hw:<n>`).
- o que se sabe do erro de permissão no macOS: a libavdevice não tem
  `authorizationStatus` nem `requestAccess` nas strings, então o ffmpeg
  não pede nem confere a permissão; o sistema decide quando a entrada é
  criada, e a recusa sai como "Failed to create AV capture input device:
  %s", com a descrição do sistema no `%s`. o texto dessa descrição não
  foi visto: "Cannot use <câmera>" e "not authorized" são palpite (é o
  que o `classify` lê como `Denied`). o pedido de permissão é do processo
  responsável (o terminal que rodou o `cargo run`, ou o `.app`). o
  microfone negado pode não falhar e entregar silêncio (palpite). as
  outras linhas lidas das strings e usadas: "Invalid device index",
  "Video device not found", "Audio device not found", "No AV capture
  device found" (`NoDevice`), "Selected framerate (%f) is not supported
  by the device.", "Supported modes:" e `  %dx%d@[%f %f]fps` (o retry).
  a última linha do stderr costuma ser genérica ("Error opening input
  files: Input/output error"): o parser lê todas.

(c) uma página "Capture" de rascunho no Apps Demo, depois de Files, com
corpo de placeholder: `regress check` deu 161 baselines, 161 iguais. a
sidebar a 800 px termina em "Badge", então o grupo não entra em nenhum
baseline. o rascunho foi revertido; a página de verdade também deu 161
iguais, e ganhou os dois baselines dela.

### a imagem de fluxo

`rust/engine/src/drawing/gpu/image.rs` (560 de produção, era 411; 898 com os
testes) e a reexportação em `gpu/mod.rs` (a linha de `pub use` que o
rustfmt quebra em três):

- `ImageStore::stream(width, height)` aloca um slot fora do dedupe, com
  pixels transparentes; `ImageStore::replace(&handle, &pixels)` troca os
  pixels da entrada do slot no log, sem entrada nova, e devolve o handle
  com `id` novo e o mesmo lugar (o nó da cena muda, a layer redesenha,
  nada no compositor sabe de fluxo). recusa `NotAStream { id }` e
  `WrongSize { width, height, bytes }`, com o texto dito.
  `stream_image_rgba` e `replace_image_rgba` sobre o store global.
- condição 1: cada `ImageAtlasGpu` guarda um `AtlasCursor` (entradas
  escritas, revisão vista); cada entrada trocada leva a revisão da troca,
  e `unwritten` devolve, uma vez, as trocadas desde a última olhada e as
  novas. teste com dois cursores (`two_atlases_write_a_replacement_once_each`)
  e na GPU com dois renderers (`a_stream_image_draws_its_latest_frame_in_every_context`).
- condição 2: um cursor novo escreve o log como está, então o último
  frame (`a_new_atlas_gets_the_latest_frame`); na GPU um renderer criado
  depois e um cujo device foi destruído e recuperado desenham o último.
- condição 3: cinquenta trocas deixam `upload_count`, `image_count` e
  `atlas_size` como estavam, e a entrada guarda um frame
  (`a_stream_is_one_slot_whose_pixels_change_in_place`). a página cria o
  slot no primeiro frame e o guarda: desligar e religar reusa (no ADR).
- condição 4: o dedupe segue para as outras imagens
  (`the_dedupe_holds_beside_streams`), e nenhum teste que existia no
  `image.rs` mudou. `image_count` passa a contar os slots de fluxo
  (semântica nova, registrada); o `prepare` olha o log em vez do mapa do
  dedupe, o que é o mesmo para imagens fixas.
- condição 5, dito sem consertar: a gravação (`PLEV_RECORD=1`,
  `rust/record`) guarda um asset de imagem por handle que viu, e cada
  frame da câmera é um `id` novo, então um asset por frame gravado, só com
  o tamanho: `put_asset` escreve imagem viva sem pixels, nenhum pixel da
  câmera vai para o `.monster`. a memória do gravador segue o orçamento
  dele (os assets têm referência e saem com o último chunk). uma sessão
  lida de volta desenha esses nós como nada; um replay vivo no mesmo
  processo desenharia o que o slot tem agora, não o frame de então. o
  `--svg` (`engine::headless::svg`) marca a imagem como deixada de fora,
  um comentário com o tamanho e uma aproximação relatada, como para toda
  imagem; e o `--svg` do showcase monta a view sem captura, então a
  página sai sem imagem nenhuma.

### o trait e os estados

`rust/engine/src/runtime/platform/capture/mod.rs` (486 de produção, 609 com os
testes), `capture/ffmpeg.rs` (218), `capture/ffmpeg/args.rs` (258, 461
com os testes), `capture/ffmpeg/child.rs` (317):

- `DeviceKind::{Camera, Microphone}`; `Device { kind, name, index }`
  (`first`, `at`, `named`); `CaptureSettings { width, height, fps, rate,
  camera, microphone }` com `Default` 640x360 a 15 fps, 16 kHz, índice 0;
  `Frame { width, height, rgba, at }` e `AudioChunk { samples, rate, at }`
  (os `Debug` dão tamanhos, nunca os bytes), `rms`;
  `CaptureState::{Off, Starting, On, Denied, NoDevice, Failed(String)}`
  (`is_live`: `Starting` ou `On`); `CaptureError::{Unsupported,
  NoDevice(String), Io(String)}`.
- `trait Capture { start(kind, &settings); stop(kind); state(kind);
  latest_frame(); take_audio() }`: nada bloqueia. o frame mais novo vence
  (um slot de um: o que ninguém pegou é trocado, e o buffer dele é
  reusado na leitura seguinte); o áudio em chunks de 1/20 s num anel de 2
  s, os mais velhos saem primeiro. `stop` solta o dispositivo e descarta o
  que ninguém pegou.
- `FfmpegCapture`: um filho por dispositivo, `PLEV_FFMPEG` ou o `ffmpeg`
  do PATH, nada roda ao construir; uma thread lê o stdout, uma o stderr
  (as últimas 128 linhas), e um vigia pergunta ao filho a cada 20 ms se
  acabou: o estado vem de todas as linhas (`classify`); sem nada em 15 s
  mata pelo pid e fica `Failed("nothing from the camera in 15 s")`; uma
  câmera que recusou o padrão e listou os modos recomeça uma vez com o
  menor modo que cobre o tamanho pedido e aceita a taxa pedida
  (`-framerate` e `-video_size` só nesse caso, com o que a câmera
  listou). `stop` mata pelo pid e colhe (`wait`); `Drop` para os dois.
  um pai que morre sem parar deixa o ffmpeg escrevendo num pipe sem
  leitor: a escrita seguinte falha e ele sai. `pid(kind)` para os
  testes; `input(Input)` e `first_frame(d)` para os falsos.
- `FakeCapture` (frames de uma cor por frame, `FakeCapture::pixel(n)`; um
  seno de 440 Hz de amplitude 1/8, `FakeCapture::RMS`; o estado de cada
  start alinhável com `will_come_to`; `starts`, `stops`, `frames`),
  `NoCapture` (wasm e mobile, `Unsupported`), `default_capture()`, e
  `Capture` para `Arc<Mutex<C>>` e `Box<C>`.

### o showcase: a página, a privacidade, coberto e dormindo

`view/apps/capture/mod.rs` (607 de produção), `capture/semantics.rs`
(237), `capture/tests.rs`; `view/dispatch.rs` 517 (era 467),
`view/input.rs` 423 (era 417), `view/panes.rs` 141 (era 136),
`view/pages/table.rs` 520 (era 513), `view/pages/mod.rs` 158 (era 157),
`view/apps/mod.rs` 20 (era 17), `app.rs` 571 (era 567),
`app/capture.rs` 37 (138 com o teste), `app/sleep.rs` 55 (era 50),
`app/detached.rs` 97 (era 96).

- a página "Capture" no Apps Demo, depois de Files: dois switches
  (`capture.camera`, `capture.microphone`, `Role::Toggle`, `on` é o que a
  pessoa ligou, ação `toggle` com `PERSON_ONLY`, a política do shell,
  agora reexportada de `view::dispatch`), a caixa do frame no tamanho da
  câmera (mais estreita se o card for), o medidor (`ProgressBar`, dB sobre
  um piso de -60, `FLOOR_DB`) e o status. a região projeta
  `capture:camera` ("off", "starting", "on 640x360 at 15 fps", "permission
  denied", "no camera", "failed: ...", e "paused while the page is not
  seen" quando a pessoa ligou e a página não está à vista) e
  `capture:microphone` (o mesmo, "on 16000 Hz"); `capture.frame:showing`,
  `capture.level:level` (em décimos, para a voz não mexer a projeção a
  cada chunk). pixels e amostras nunca são projetados.
- privacidade: um agente pedindo `toggle` recebe `NotAuthorized` antes do
  `execute`, nada liga, e ele lê as linhas. a página nasce com
  `NoCapture`; só o shell, com a janela de pé, dá o `default_capture()`
  (`App::install_capture`, ao lado do `install_print`). então uma galeria
  headless, um baseline, o `--render` e todo teste de integração não
  alcançam um dispositivo.
- coberto ou dormindo: o dispositivo só roda enquanto a pessoa quer e a
  página pode ser vista (é a página ativa, uma janela que a mostra está
  acordada, o app não dorme). o shell passa a palavra das janelas no
  próprio evento (`App::capture_follows` em `covered`, `sleep`, `wake` e
  ao fechar a janela da página), porque uma janela coberta só tica quando
  algo acorda o laço; a página para o dispositivo (o filho morre, a luz
  apaga), esvazia a caixa e descarta o que não foi pego; ao mostrar ou
  acordar, liga de novo se a pessoa tinha ligado, e a caixa espera um
  frame novo. sair da página faz o mesmo (o `page_wake` diz à página, a
  cada wake, se ela é a ativa). decidido: soltar, não pausar; está no
  ADR. um dispositivo que acabou sozinho (negado, falhou) não recomeça a
  cada wake: só quando a pessoa desliga e liga, ou a página sai de vista
  e volta.
- o frame novo só entra na textura no `wake` com a página visível (nunca
  num frame, nunca dormindo); com um dispositivo vivo o `Wake` pede
  `after(1/fps)`; sem nenhum, nada.
- teclas: Tab anda pelos dois switches, Enter liga o focado, Escape solta
  (os braços em `input.rs`).

### testes

- `engine::gpu::image` (+5): um slot por fluxo e cinquenta trocas; as
  recusas e os textos; dois cursores; o cursor novo com o último frame; o
  dedupe ao lado.
- `engine::platform::capture` (+11): as settings; o falso (frames novos,
  o seno, o desligado); o falso alinhado; o nível; `NoCapture` e o
  compartilhado; a entrada de cada plataforma por índice e nome; a saída;
  os modos e o retry; o `classify` de todas as linhas; o anel; construir
  sem rodar e o programa que não existe.
- `rust/engine/tests/capture.rs` (10 e 1 ignorado; binário novo): a
  sonda (câmera, microfone, frames num fluxo e desenhados, a imagem de
  fluxo em três contextos com device recuperado), e o `FfmpegCapture`
  sobre os falsos: frames do tamanho pedido a uns 15 fps com o mais novo
  vencendo e um frame só esperando; o microfone com chunks de 800 e o anel
  em 32 000 amostras depois de 3 s sem ninguém pegar; parar mata o pid
  (câmera e microfone são filhos diferentes; o drop mata o que sobra); o
  prazo mata o mudo em 2 s; negado, sem dispositivo e falha pelas
  palavras; o retry com `-framerate 15 -video_size 640x480`. os falsos
  deixam os argumentos em `$TMPDIR/plev-capture-fakes/<pid>.args`, e cada
  teste apaga os seus.
- `view::apps::capture` (+9): a projeção em repouso e a política; o
  agente recusado, a pessoa e a tecnologia assistiva aceitas; o frame
  entra no wake e não antes (o mesmo slot, id novo, desenhado); o prazo só
  com dispositivo vivo; coberto, dormindo e fora da página solta e volta;
  as recusas como linhas sem recomeçar; o medidor; o clique e as teclas; o
  card num telefone e numa janela larga.
- `app::capture` (+1): o shell sobre janela nua, o agente recusado, a
  pessoa liga, coberto solta na hora e cinco ticks sem frame, mostrar
  religa, dormir solta, acordar religa, outra página solta, voltar religa.
- `apps/uigallery/tests/capture.rs` (1, GPU; binário novo): o frame
  desenhado antes do wake mostra a caixa vazia, o primeiro depois do wake
  mostra o frame 0 do falso dentro da caixa, o seguinte o frame 1 no mesmo
  lugar; ligado, um prazo de um período; não visto, a caixa vazia no
  frame seguinte e nada pedido; visto de novo, um frame novo.

### gate e regress

`df -h /` antes de cada gate: 18 GB livres; 18 GB depois. `./tool/tasks/fullcheck.sh`
verde no worktree com `CARGO_NET_OFFLINE=true`, com a perna wasm e a
perna urnaui (`../urna` presente), duas vezes (a segunda depois de
`de4a4d98`): antes (a base `9ea37ea9`, medida neste worktree) 2604
passed, 0 failed, 16 ignored em 126 binários; depois 2641 passed, 0
failed, 17 ignored em 128 binários (+37: 5 do `gpu::image`, 11 do
`platform::capture`, 10 de `engine/tests/capture.rs`, 9 da página, 1 de
`app::capture`, 1 de `showcase/tests/capture.rs`; +1 ignorado, o
atlas cheio; +2 binários, os dois `tests/capture.rs`). perna urnaui: 85
passed, 0 failed, 1 ignored em 6 binários, igual a antes. clippy `-D
warnings`, fmt e a checagem wasm limpos nas duas pernas; o único aviso no
log é o `block v0.1.6` de incompatibilidade futura, que já estava.

`cargo run --offline -p regress -- check`: 163
baselines, 163 iguais, 0 mudaram, sem frame desenhado e sem GPU aberta.
os 161 de antes não mudaram; os dois novos são `capture@core@1280x800` e
`capture@hoff@1280x800` (`bless --page Capture --theme core --png` e
`--theme hoff --png`), a página em repouso: os dois switches desligados,
a caixa vazia dizendo "off", o medidor em zero.

linhas de produção: acima; tudo abaixo de 639 (`file_hygiene_guard`
verde). a página está em 607: o próximo que tocá-la tira o `render` para
um `draw.rs`.

### ADR

`docs/adr/interface/0062-capture-is-a-job-the-person-turns-on.md` (proposed): o trait, o
ffmpeg como filho e por quê, a permissão como estado, só pessoa, a imagem
de fluxo, coberto ou dormindo solta o dispositivo, o que o bring-up muda.
linha para o índice (`docs/adr/README.md` está fora da lista, fica para a
integração):

```text
| 2026-09-24 | [capture-is-a-job-the-person-turns-on](capture-is-a-job-the-person-turns-on.md) | proposed | task-76 item 10: `engine::platform::capture` põe câmera e microfone atrás de um trait (`Capture`: começar e parar cada `DeviceKind` sem bloquear o laço, `CaptureState` Off, Starting, On, Denied, NoDevice, Failed, o `Frame` RGBA mais novo, um só guardado, e os `AudioChunk` mono f32 num anel de 2 s); `FfmpegCapture` roda um ffmpeg por dispositivo na entrada da plataforma (avfoundation, dshow, v4l2 e alsa), lê o pipe cru, lê todo o stderr como estado, mata pelo pid no prazo de 15 s e ao parar, e recomeça uma câmera que recusou a taxa padrão com um modo que ela listou, sem dependência; a permissão é estado, nunca falha (o texto da recusa é palpite até a prova real); o frame vai para uma imagem de fluxo (`stream_image_rgba`/`replace_image_rgba`: um slot trocado no lugar com id novo, escrito uma vez por atlas pela revisão, o último frame num atlas novo) no wake; no showcase `capture.camera` e `capture.microphone` são só de pessoa (`PERSON_ONLY`), o agente lê `capture:camera` e `capture:microphone`; coberta, dormindo ou fora da página o dispositivo é solto na hora e volta ao mostrar se a pessoa o tinha ligado; provado com falsos e lavfi, nenhum dispositivo aberto |
```

### interfaces adicionadas (só crescimento, salvo o dito)

- `engine::gpu::image::{ImageStore::stream, ImageStore::replace,
  stream_image_rgba, replace_image_rgba}`, `ImageError::{NotAStream,
  WrongSize}` (o `match` de fora não existia; nada quebrou), reexportadas
  em `engine::gpu`. `ImageStore::image_count` conta os slots de fluxo
  (semântica nova); `ImageAtlasGpu::prepare` olha o log.
- `engine::platform::capture::{DeviceKind, Device, CaptureSettings,
  Frame, AudioChunk, rms, CaptureState, CaptureError, Capture,
  FakeCapture, NoCapture, default_capture, FfmpegCapture, Input,
  FFMPEG_ENV}` e `engine::platform::capture` em `platform/mod.rs` (uma
  linha).
- showcase: `Pane::Capture`, a página (`view::apps::capture::CapturePage`),
  `ShowcaseView::{set_capture, set_capture_shown}` (em `dispatch.rs`),
  `view::dispatch::PERSON_ONLY` (`pub(crate)`, reexportado do `shell`),
  `App::{install_capture, capture_follows}` (`pub(super)`). a projeção da
  galeria ganha "Capture" nas opções de `shell.page`; nenhuma assinatura
  pública mudou; ide, urnaui, wasm e os exemplos compilam sem tocar.

### arquivos tocados fora da lista permitida, e por quê

- `rust/engine/src/drawing/gpu/image.rs` e `rust/engine/src/drawing/gpu/mod.rs`:
  liberados pelo orquestrador depois da parada (a opção 1).
- `apps/uigallery/src/views/apps/mod.rs`: a linha `pub mod capture;` (e
  o doc do grupo), sem a qual o diretório novo não é módulo.
- `apps/uigallery/src/shell/mod.rs`: `mod capture;` e a chamada
  `self.install_capture();` no `resumed`, ao lado do `install_print()`. a
  lista dizia `app/**`; o `app.rs` é a raiz desse diretório, e o
  `resumed` mora nele.
- `tool/bench/regress/baselines/capture@core@1280x800` e
  `capture@hoff@1280x800`: os baselines novos da página, que o briefing
  permitia se a sonda (c) mostrasse os outros iguais.
- `apps/uigallery/src/views/dispatcher/mod.rs` está na lista pelos braços; além
  deles recebeu a reexportação do `PERSON_ONLY` e os dois métodos de
  `ShowcaseView` que o shell chama (o `view/mod.rs` está fora).

### limitações e decisões, sem esconder

- nenhum dispositivo real foi aberto: a câmera e o microfone de verdade,
  o diálogo de permissão, a luz, o texto de uma recusa, os modos de uma
  câmera que não aceita 29,97 e o silêncio de um microfone negado não
  foram vistos. o que o `classify` lê como `Denied` é palpite; o retry lê
  o formato das strings da libavdevice, não uma saída vista. a prova real
  é do brenner (roteiro abaixo), e o que ela mostrar entra no `args.rs`.
- sem `-video_size` na entrada, uma câmera que abre em 1080p é escalada
  pelo ffmpeg a cada frame que fica (o `fps` vem antes do `scale` desde
  `de4a4d98`, então os que saem não são escalados); o custo de CPU do
  ffmpeg não foi medido com um dispositivo.
- o frame não é espelhado (a vista de selfie costuma ser); o áudio é mono
  a 16 kHz e a página só lê o nível: o buffer está no trait para quem o
  quiser.
- o slot de fluxo nunca é liberado (o atlas não despeja); um por página.
- o prazo do primeiro frame é 15 s: uma pessoa que demora mais no
  diálogo vê "failed: nothing from the camera in 15 s" e liga de novo
  depois de responder.
- no windows o índice 0 sem nome vira "failed: capture: dshow opens a
  camera by its name, and none was given" até o bring-up dar a lista.
- o shell só recalcula "visível" nos eventos de janela e no fechar a
  janela da página; abrir a janela da página com a principal coberta não
  acontece (o `D` é apertado numa janela visível).
- a projeção muda o `shell.page` (uma opção a mais) na projeção da
  galeria; a da página (`ShowcaseView::semantics`) das outras páginas não
  mudou.
- a página tem 607 linhas de produção.
- gravação e `--svg`: condição 5, acima, sem conserto.

### o que fica para o bring-up e o que o item 6 reaproveita

- bring-up: AVFoundation direto (a permissão antes do start, `Denied`
  dito pelo sistema, a lista de dispositivos e a escolha na página); um
  `.app` com `NSCameraUsageDescription` e `NSMicrophoneUsageDescription`
  no `Info.plist` (o bundle passa a ser o processo responsável, também
  pelo ffmpeg filho); windows com os nomes do dshow; linux com v4l2 e
  alsa ou pulse numa máquina real; mobile com as APIs do sistema.
- item 6 (deep links): um deep link vira `Request` pelo dispatcher com um
  ator; `capture.camera` e `capture.microphone` são `PERSON_ONLY`, então
  um deep link nunca liga câmera nem microfone, a menos que o item 6
  decida que um deep link é uma pessoa, e aí a câmera pede a confirmação
  humana da task-58 antes. o mesmo vale para `shell.print` e as janelas.

### o roteiro da prova real (do brenner, nunca de teste)

a luz da câmera acende nesta parte; nada disto foi feito aqui.

```text
RUST_LOG=engine=debug,showcase=info cargo run --offline -p uigallery -- Capture
  clicar no switch "camera" (ou Tab e Enter). no log:
    "capture: the page starts the camera"
    "capture: camera with ffmpeg -hide_banner -nostdin -loglevel error -f avfoundation -i 0:none -vf fps=15,scale=640:360:... -f rawvideo -pix_fmt rgba pipe:1"
  o macOS pergunta se o app que rodou o cargo (Terminal, iTerm, Ghostty, o terminal do Zed) pode usar a câmera:
  o pedido é do processo responsável, não do plev nem do ffmpeg.
  permitir: a luz acende, "capture: camera on" no log, a caixa mostra a câmera, a linha diz "on 640x360 at 15 fps".
  se o log disser "the camera refused its default, again at <w>x<h> and <n> fps", o retry rodou: anotar a linha e os modos.
  negar: a linha deve dizer "permission denied". as linhas do stderr aparecem como "capture: camera: ..." (engine=debug):
  se a página disser "failed: ..." em vez disso, esse texto é o da recusa real e falta no classify (capture/ffmpeg/args.rs).
  sem responder o diálogo por 15 s: "failed: nothing from the camera in 15 s"; responder e ligar de novo.
o microfone: o switch "microphone", o diálogo do microfone, falar e ver o medidor subir; a linha "on 16000 Hz".
  negado: o palpite é silêncio (o medidor parado em 0 com "on 16000 Hz"); anotar o que acontecer.
cobrir: cmd-H ou minimizar. no log "capture: the page stops the camera"; a luz apaga;
  pgrep -fl "ffmpeg.*avfoundation"   (nada). restaurar: a luz acende de novo e a caixa espera um frame novo.
  ir para outra página (Files) e voltar: o mesmo.
o agente, em outro terminal, com o app rodando com PLEV_AGENT=1:
  cargo run --offline -p socket --bin plev -- observe --scope capture       (capture:camera, capture:microphone)
  cargo run --offline -p socket --bin plev -- act capture.camera toggle     (recusado: NotAuthorized; nada liga)
desfazer: Ajustes do Sistema > Privacidade e Segurança > Câmera (e Microfone): desligar o app do terminal;
  ou tccutil reset Camera e tccutil reset Microphone (todos os apps), ou só um: tccutil reset Camera com.apple.Terminal
a lista dos dispositivos, se quiser escolher outra câmera: ffmpeg -f avfoundation -list_devices true -i ""   (também pode pedir a permissão)
```

o que olhar: a luz acesa só com a página à vista e a pessoa tendo ligado;
nenhum ffmpeg vivo depois de cobrir, dormir, sair da página ou fechar o
app; o texto real da recusa e dos modos, para o `args.rs`.

### comandos de validação

```text
export CARGO_TARGET_DIR=/Users/nn/Dev/hoff/plevdev/target/agents/one
export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_PROFILE_TEST_DEBUG=line-tables-only
cargo test --offline -p engine --lib -- gpu::image platform::capture
cargo test --offline -p engine --test capture -- --nocapture           (a sonda imprime as medidas; precisa do ffmpeg no PATH)
cargo test --offline -p engine --test capture -- --ignored --nocapture  (o atlas cheio: 244 frames, 225 MB)
cargo test --offline -p uigallery --lib -- apps::capture app::capture
cargo test --offline -p uigallery --test capture
cargo test --offline -p engine -p uigallery
./tool/tasks/fullcheck.sh
cargo run --offline -p regress -- check              (163 iguais)
cargo run --offline -p uigallery -- --render Capture --out capture.png   (a página em repouso, sem dispositivo)
```

### integração do item 10 (orquestrador, 2026-09-24)

- merge da `task/76i-capture` na `main` sem commit a mais. a revisão
  conferiu a imagem de fluxo (o crescimento do atlas copia no mesmo
  lugar, o cursor vale), a privacidade (nenhum teste chega a dispositivo,
  a página começa com `NoCapture`) e o prazo do laço (o `Starting` conta
  como vivo).
- P3: no tema core o interruptor da câmera e do microfone quase não
  aparece (baseline `capture@core`).

## handoff do item 6: deep links

branch `task/76j-links`, base `bcf6be8e` (main após o item 10). worktree
`.claude/worktrees/agent`, target `target/agents/one`. commits, em ordem:

- `e3b6c8d2` task-76: the deep link probe
- `df91e2a8` task-76: a link read by a strict grammar, and its sources
  with the fake
- `edefe39b` task-76: a link is not a person, and the macOS path recorded
  for the bring-up
- `66345cf5` task-76: the showcase follows a link as the agent link, only
  where it is open
- `e52359d0` task-76: a link is checked as an agent's, and its refusal is
  on screen
- (este) task-76: the item 6 handoff and the close of task-76

a rodada parou uma vez, depois da sonda, em duas condições do briefing ao
mesmo tempo (a seção "a parada", abaixo): o winit 0.30.13 registra o
delegado de aplicação dele no macOS e depende dele, e escrever qualquer
classe objc no engine pede dependência direta que muda o lock. o
orquestrador decidiu a opção 4 (a seção "a decisão"). os testes de cada
comportamento saíram no commit dele; `e52359d0` são os de integração. em
nenhum momento: esquema registrado, `open` de um link, `.app`, janela.

### a sonda

(a) o macOS, por leitura da fonte e por `cargo tree --offline`:

- o engine não tem `objc2`, `objc2-foundation` nem `objc2-app-kit` como
  dependência direta: `rust/engine/Cargo.toml` só os cita no comentário
  do item 5 (o caminho do UserNotifications ficou para o bundle), e a
  entrada `engine` do `Cargo.lock` não os lista. eles estão no lock pelos
  outros: `objc2 0.6.4` (arboard, block2, dispatch2, muda, rfd, tray-icon
  e as crates `objc2-*`), `objc2-foundation 0.3.2` e `objc2-app-kit 0.3.2`
  (arboard, muda, rfd, tray-icon); o winit 0.30.13 usa os seus, `objc2
  0.5.2` e `objc2-app-kit 0.2.2`.
- as features ligadas hoje, pela unificação (`cargo tree --offline -p
  engine -e features -i <crate>`): `objc2` std, alloc, default;
  `objc2-foundation 0.3.2` NSArray, NSString, NSURL, NSNotification,
  NSBundle, NSData, NSDictionary, NSError, NSThread e outras, sem
  NSAppleEventManager e sem NSAppleEventDescriptor; `objc2-app-kit 0.3.2`
  NSApplication, NSResponder, NSRunningApplication, NSEvent, NSMenu,
  NSMenuItem, NSWindow, NSView, NSStatusBar e outras. então
  `define_class!` (`objc2-0.6.4/src/macros/define_class.rs`),
  `NSApplicationDelegate` com `application:openURLs:`
  (`objc2-app-kit-0.3.2/src/generated/NSApplication.rs:1128`, feature
  NSApplication), `NSApplication::setDelegate`, `NSURL` e `NSArray`
  compilariam com as features que já estão ligadas; falta ao engine poder
  nomear as crates.
- o lock, medido numa cópia do workspace no scratchpad, nunca neste
  worktree: com `objc2 = "0.6"`, `objc2-foundation` e `objc2-app-kit` em
  `cfg(target_os = "macos")` no manifesto do engine, `cargo metadata
  --offline` resolve sem baixar nada, e o `Cargo.lock` muda em três
  linhas, todas na lista de dependências da entrada `engine`
  (`"objc2 0.6.4"`, `"objc2-app-kit 0.3.2"`, `"objc2-foundation 0.3.2"`).
  nenhum pacote novo. é dependência nova do engine e muda o lock.
- quem registra delegado: o `muda` 0.20.0 só põe delegado num `NSMenu`
  (`platform_impl/macos/mod.rs:94`, `ns_menu.setDelegate`), nunca no
  `NSApp`; `tray-icon` 0.25.1, `rfd` 0.15.4, `arboard` 3.6.1 e
  `accesskit_macos` 0.26 não chamam `setDelegate` no app e não registram
  handler de Apple Event (`NSAppleEventManager` não aparece em nenhum). o
  winit 0.30.13 registra: `platform_impl/macos/event_loop.rs:240`, dentro
  de `EventLoop::new`, põe o `WinitApplicationDelegate` no app
  (`app_state.rs:47-72`, um `declare_class!` com
  `applicationDidFinishLaunching:` e `applicationWillTerminate:`, que
  guarda o estado todo do laço). e lê de volta a cada volta:
  `ApplicationDelegate::get` (`app_state.rs:173-184`) pega
  `NSApp.delegate()` e entra em pânico com "tried to get a delegate that
  was not the one Winit has registered" quando não é o dele, e é chamado
  pelo observador do run loop em todo `kCFRunLoopAfterWaiting` e
  `kCFRunLoopBeforeWaiting` (`observer.rs:62` e `:84`) e em todo
  `sendEvent:` (o swizzle de `app.rs:45`).
- o texto de `winit-0.30.13/src/platform/macos.rs:20-80` ("Winit
  guarantees that it will not register an application delegate", com o
  exemplo do `application:openURLs:`) não vale para a 0.30.13, que é a do
  lock: o changelog da 0.30 só tira o delegado do iOS. o briefing leu esse
  texto. um delegado nosso substituiria o do winit, e o laço entraria em
  pânico no primeiro evento ou na primeira volta depois do `setDelegate`.
  não rodei isso (nenhum laço nem janela); é leitura da fonte, linha por
  linha.
- o que entrega a URL sem trocar o delegado: o evento Apple de URL
  (classe `kInternetEventClass`, id `kAEGetURL`, os dois `'GURL'`), por
  `NSAppleEventManager::sharedAppleEventManager()` e
  `setEventHandler_andSelector_forEventClass_andEventID`
  (`objc2-foundation-0.3.2/src/generated/NSAppleEventManager.rs:61` e
  `:71`, feature NSAppleEventManager), com a URL em
  `paramDescriptorForKeyword(keyDirectObject).stringValue()`
  (`NSAppleEventDescriptor.rs:338` e `:288`). é o que chama o
  `application:openURLs:` por baixo: o AppKit instala o handler dele para
  `GURL` ao lançar e repassa ao delegado. um handler nosso instalado no
  `NSApplicationWillFinishLaunchingNotification` (observado pelo
  `NSNotificationCenter`, sem delegado) toma o lugar do do AppKit e deixa
  o do winit em paz. também pede `objc2` e `objc2-foundation` diretos (o
  lock muda nas mesmas três linhas, com o `objc2-app-kit` para o nome da
  notificação), mais as features NSAppleEventManager e
  NSAppleEventDescriptor, que ninguém liga hoje (feature não muda o
  lock). um handler de Apple Event é do processo: não registra esquema no
  sistema.
- no macOS o link nunca chega como argumento: o Launch Services lança o
  app (só um `.app` com `CFBundleURLTypes` no `Info.plist`) e entrega a
  URL como evento `GURL` logo depois do lançamento; com o app aberto, o
  mesmo evento vai para a instância viva, sem processo novo. o handler
  precisa existir antes do fim do `finishLaunching` para pegar o link que
  lançou o app; o `resumed` do showcase vem depois, tarde para esse link.

(b) windows e linux, por leitura (nada registrado):

- windows: o esquema é uma chave `HKEY_CURRENT_USER\Software\Classes\<esquema>`
  com o valor `URL Protocol` e `shell\open\command` igual a `"<exe>"
  "%1"`. abrir o link cria sempre um processo novo com a URL em
  `argv[1]`, com o app aberto ou não.
- linux: um `.desktop` com `MimeType=x-scheme-handler/<esquema>;` e
  `Exec=<exe> %u`, escolhido por `xdg-mime default <app>.desktop
  x-scheme-handler/<esquema>`; o `xdg-open` do link cria um processo novo
  com a URL no lugar do `%u`.
- a segunda instância: nos dois, um link com o app aberto abre outro
  processo, com outra janela e estado novo, e a instância que já roda não
  fica sabendo. encaminhar (mutex nomeado e pipe no windows, D-Bus ou
  socket no linux, e o processo novo sai) é do bring-up. no macOS não
  existe segunda instância: o evento vai para a viva.
- o showcase hoje: `App::new` toma o primeiro argumento como título de
  página (`jump_to_page`), que não acha `plev-showcase://...` e ignora
  sem dizer nada; o segundo seria o tema.

(c) as capacidades do showcase hoje, na projeção da galeria (o shell e a
página ativa; dos goldens da todo, das silences e da files, e das
semânticas da capture e da surface):

- shell: `shell.page` `select` e `shell.theme` `select` (texto dentro das
  opções, qualquer ator); `shell.detach` `toggle` (abre janela),
  `shell.show` `activate` (traz a janela), `shell.print` `activate`
  (impressora) e `shell.save_session` `activate` (só gravando, escreve
  arquivo), os quatro `PERSON_ONLY`.
- todo: `todo.input` `set_value` (texto) e `submit`, `todo.filter[..]`
  `select`, `todo.item[n]` `toggle` e `remove` (destrutivo), qualquer
  ator.
- silences: `set_value`, `increment` e `decrement` dos dois sliders,
  `detect_silence` (roda o ffmpeg), `preview_segment`, `apply_cut`,
  `reject_cut`, `apply_all`, qualquer ator.
- files: `files.drop` `drop_files` (lista, qualquer ator); `files.open`
  `open_file` e `files.warn` `warn` (human, assistive e test: diálogo).
- capture: `capture.camera` e `capture.microphone` `toggle`
  (`PERSON_ONLY`: dispositivo).
- surface: `propose` (o claude por filho) e o cancelar.
- a proposta do briefing se sustenta: abertos a link, só `shell.page`
  `select` e `shell.theme` `select`. navegar e trocar o tema não escrevem,
  não apagam, não abrem dispositivo, diálogo, arquivo, janela nem
  impressora, e não rodam processo. o resto que um agente pode (o todo,
  as silences, o drop) fica fechado a link pela lista, mesmo aceito pela
  policy: um link vem de qualquer página web, e a lista é do app.

### a parada

problema: o caminho do macOS que o briefing descreve (um delegado nosso
com `application:openURLs:`) não existe com este winit, e qualquer
caminho objc no engine pede dependência direta que muda o lock.

1. o delegado do winit: um delegado nosso substitui o
   `WinitApplicationDelegate`, e o winit entra em pânico na volta
   seguinte do laço (sonda (a)).
2. o lock: `objc2`, `objc2-foundation` (e `objc2-app-kit`) como
   dependência direta do engine mudam três linhas do `Cargo.lock`, sem
   pacote novo.

opções e impacto:

1. o handler do evento Apple `GURL`, sem delegado, com `objc2`,
   `objc2-foundation` e `objc2-app-kit` diretos no engine em macOS, as
   features NSAppleEventManager e NSAppleEventDescriptor a mais. o lock
   muda três linhas na entrada `engine`, resolve offline; uma classe com
   `define_class!` que observa o `WillFinishLaunching`, instala o handler,
   empurra a URL no canal e chama o wake. o teste chama o método do
   handler pelo objc com um descritor montado por ele, sem esquema e sem
   janela. na tela, só recebe algo depois do bring-up do bundle, como o
   `NativeTray` compilado e não rodado.
2. o mesmo handler contra o runtime objc por `extern "C"` (libobjc:
   `objc_allocateClassPair`, `class_addMethod`, `sel_registerName`,
   `objc_msgSend` transmutado), sem crate. o lock fica intacto; entram
   umas 120 linhas de `unsafe` com `// SAFETY:` e transmutes, contra o
   "unsafe proibido por padrão", com custo de revisão alto.
3. um método `application:openURLs:` posto em runtime na classe do
   delegado do winit. depende do nome privado de uma classe de outra
   crate, quebra calado numa atualização, e ainda pede o objc2 ou o
   runtime cru. não recomendo.
4. no macOS, só a fonte do argumento de lançamento até o bring-up; o
   handler `GURL` entra com o bundle, como a notificação do item 5. o
   manifesto e o lock ficam intactos; tudo o mais do briefing entra (o
   parse, as fontes, o falso, o showcase, o ator, a lista, a gravação, os
   testes, o ADR), e o ADR descreve o handler como o caminho do bring-up.
   `cargo run -p uigallery -- 'plev-showcase://shell.page/select?value=Files'`
   exercita o caminho inteiro no macOS sem registrar nada.

recomendação: a opção 1, se as três linhas do lock forem aceitas: é o
caminho de verdade do macOS, pequeno, sem pacote novo, e testável pelo
objc sem esquema. sem o aceite, a opção 4. as opções 2 e 3 não.

decisão necessária: aceitar as três linhas do `Cargo.lock` e as três
dependências diretas do engine em macOS (opção 1), ou seguir sem
caminho do sistema no macOS até o bring-up (opção 4). o resto do item
(commits 2, 4, 5 e 6 do briefing) não depende da escolha; só o commit 3.

### a decisão

do orquestrador, depois da parada: a opção 4. no macOS o link chega só
pelo argumento de lançamento até o bring-up; o manifesto e o
`Cargo.lock` ficam intactos, e não entram delegado nem handler de Apple
Event. o motivo: o evento `GURL` só chega a um app com o esquema
registrado, e registrar exige o bundle; o código da opção 1 ficaria sem
caminho real até lá, como a notificação do item 5. o commit 3 virou o
registro do desenho para o bring-up (o ADR, seção "bring-up"), sem código
de macOS. a fonte do argumento vale nas três plataformas, e no showcase
um argumento que começa com `plev-showcase://` é link, nunca título de
página.

### a gramática e os erros

`rust/engine/src/runtime/platform/links/mod.rs` (382 linhas, todas de produção; os
testes em `links/tests.rs`) e `links/source.rs` (212 de produção, 333 com
os testes). `engine::platform::links`:

- `Link { target: SemId, action: String, value: Option<String> }`;
  `Link::parse(url, scheme)` lê `<esquema>://<sem id>/<ação>[?value=<texto>]`.
  olha nesta ordem e para no primeiro que vale: o tamanho (no máximo
  `MAX_LEN` = 2048 bytes: um link que navega tem menos de cem; 2048 deixa
  o parse e a linha da recusa limitados, abaixo dos 2083 caracteres que o
  shell do windows repassa); os caracteres (só ASCII imprimível, 0x21 a
  0x7E: espaço, controle e letra fora do ASCII viajam percent-encoded); o
  esquema (o do app, sem diferença de caixa, RFC 3986 3.1); fragmento
  (`#`); usuário ou senha (`@`) e porta (`:`) antes da primeira `/`; barra
  a mais; o id (vazio, depois o percent-decoding, depois `SemId::parse`);
  a ação (vazia, ou fora de `[a-z_]`, sem decoding); a consulta (só a
  chave `value`, uma vez; o valor com percent-decoding, `+` é mais, espaço
  é `%20`). todo decoding recusa um `%` sem dois dígitos hex, bytes que
  não são UTF-8, e texto com caractere de controle ou marca, override ou
  isolate de direção (a recusa é uma linha que a pessoa lê: um `%0A`
  forjaria uma segunda linha, um `%E2%80%AE` inverteria a leitura).
- `LinkError::{TooLong, BadChar, NotALink, OtherScheme, Fragment,
  UserInfo, Port, ExtraPath, NoTarget, BadTarget, NoAction, BadAction,
  NoValue, UnknownKey, ExtraKey, Percent, Utf8, Control}`, cada um com o
  texto que diz por quê ("a link has no fragment (#)", "the link's id is
  not an id: invalid character 'S' at byte 0", "the link has 3000 bytes,
  above the 2048 a link may have").
- `Link::param()` (o valor como `Value::Text`), `Link::is_in(open)` (a
  lista do app, pares `(id, ação)`), `Link::request(seen)`
  (`Request::observe` por `actor()`), `Link::url(scheme)` (a forma que o
  parse lê de volta: os colchetes do id crus, o resto fora de
  `[A-Za-z0-9._~-]` como `%XX`, uma linha de ASCII imprimível);
  `is_link(text, scheme)` (começa com `<esquema>://`, sem diferença de
  caixa); `excerpt(text)` (96 caracteres no máximo e reticências, o resto
  escapado); `ACTOR` = `"link"`.
- `Refused { link: excerpt, why: LinkError }` ("<link>: <por quê>"),
  `Delivered = Result<Link, Refused>`, `read(raw, scheme)`.

### o ator e a lista aberta

- o ator: `Actor::Agent { name: "link" }` (`links::actor()`), nunca
  `Actor::Human` e nunca um ator novo. o `Actor::Agent` bastou: a coluna
  `agent` da policy é o mínimo que o contrato dá a quem vem de fora, o
  `sem` não mudou, e o dispatcher checa um link como checa um agente
  (`PERSON_ONLY` é `NotAuthorized`, stale, inert, disabled, o parâmetro,
  e uma confirmação que a policy pede de agente espera uma pessoa, que o
  link não pode dar: `ConfirmByMachine`).
- a lista do showcase (`showcase::app::links::OPEN`): `shell.page`
  `select` e `shell.theme` `select`, e nada mais. fora dela, nenhum
  request é feito, mesmo quando a policy deixaria um agente (o todo, as
  silences, o drop, a surface). dentro dela, o dispatcher ainda decide.
- a recusa: nada roda, e a pessoa vê uma linha, `link refused: <link>:
  <por quê>`, como toast na tela (`Intent::Destructive`) e em
  `shell:notice` (o agente lê), e o log diz (`warn`). os porquês: o que o
  parse disse; "`<id> <ação>` is not open to links"; o `SemError` do
  `observe` quando a capacidade sumiu; do dispatcher, "`<alvo>` is not
  there now", "agent:link may not ask this", "what it read changed
  before it ran", "`<alvo>` is behind an overlay now", "`<alvo> <ação>` is
  disabled now", o motivo do `InvalidParam` (`"nope" is not one of the
  options`); uma falha é "link failed: ...". um link que espera
  confirmação diz "link waits for a person to confirm it: <link>" pelo
  mesmo caminho (toast informativo). um link que roda não diz nada além
  do que mudou (o log em `info`).

### o caminho por plataforma

- windows e linux: a fonte do argumento de lançamento (`LaunchLinks`),
  que é como o sistema entrega um esquema registrado: um processo novo com
  a URL como argumento. a segunda instância abre outra janela com o link
  aplicado nela; encaminhar é do bring-up.
- macOS: a mesma fonte, hoje (`cargo run -p uigallery -- <link>`, ou `open
  -a <app> --args <link>` depois do bundle). o sistema só entrega o
  esquema a um `.app` com `CFBundleURLTypes`, como evento `GURL`; esse
  handler é do bring-up (abaixo e no ADR). nada de delegado: o do winit
  0.30.13 entra em pânico se for trocado (sonda (a)).
- wasm, iOS, android: `NoLinks`. os shells móveis entregam links pela
  própria entrada (`application:openURL:options:`, o data da intent), do
  bring-up.
- `default_links(scheme, wake)`: o argumento em todo desktop, lido quando
  chamado; `NoLinks` no resto. o `wake` é para a fonte que entrega com o
  app rodando (o handler `GURL` do bring-up); o argumento, lido uma vez,
  nunca o chama.
- o desenho do bring-up no macOS, registrado: `objc2`, `objc2-foundation`
  (features `NSAppleEventManager`, `NSAppleEventDescriptor`,
  `NSNotification`, `NSString`) e `objc2-app-kit` (`NSApplication`) como
  dependências diretas do engine em macOS, as três linhas do lock medidas
  na sonda (`"objc2 0.6.4"`, `"objc2-app-kit 0.3.2"`, `"objc2-foundation
  0.3.2"` na entrada `engine`, sem pacote novo); uma classe pequena
  (`define_class!`) que observa `NSApplicationWillFinishLaunchingNotification`
  pelo `NSNotificationCenter` e então se instala com
  `NSAppleEventManager::sharedAppleEventManager().setEventHandler_andSelector_forEventClass_andEventID`
  para `GURL`/`GURL` (depois que o AppKit instalou o dele, antes que os
  eventos do lançamento sejam despachados, então o link que lançou o app
  é pego), lê a URL de `paramDescriptorForKeyword(keyDirectObject).stringValue()`,
  põe na fila da fonte e chama o `wake` (o showcase manda a si mesmo
  `UserEvent::Link`). o teste chama o método do handler pelo objc com um
  descritor montado por ele, sem esquema e sem janela. o delegado do
  winit fica: `EventLoop::new` põe o `WinitApplicationDelegate`
  (`platform_impl/macos/event_loop.rs:240`), e `ApplicationDelegate::get`
  (`app_state.rs:173-184`) entra em pânico com "tried to get a delegate
  that was not the one Winit has registered" a cada
  `kCFRunLoopAfterWaiting` e `kCFRunLoopBeforeWaiting` (`observer.rs:62`
  e `:84`) e a cada `sendEvent:` (`app.rs:45`) se não for o dele.

### o showcase

`apps/uigallery/src/shell/system/links/mod.rs` (208 de produção; os testes em
`app/links/tests.rs`), `app.rs` 596 (era 571).

- `pub mod links` em `app.rs`, público para o teste de integração:
  `SCHEME` (`plev-showcase`), `OPEN`, `follow(view, delivered, open,
  looked) -> Followed { receipt, told }`. `follow` recusa o que não é
  link, o que não está na lista e o que o `observe` não acha; senão avisa
  `looked` com a revisão vista (a gravação) e pede `view.act(req)`; o
  estágio do receipt decide se há linha.
- o shell: o campo `links: Option<Box<dyn LinkSource>>`;
  `install_links()` (`default_links` com o `wake` que manda
  `UserEvent::Link` pelo proxy no desktop) e `links_arrived()` no
  `resumed`, depois das outras instalações, com a janela principal de pé:
  o link de lançamento é lido e seguido então. `links_arrived` também no
  `resumed` de um app que já tinha janela (depois do `wake`) e no
  `UserEvent::Link`. dormindo (`Background::asleep()`), não faz poll: o
  link espera na fonte e é aplicado no primeiro poll acordado. cada link
  seguido, um `changed()` (estado gravado, frame pedido).
- `App::new` virou `with_view(launch_view(args))`; `launch_view` tira dos
  argumentos posicionais (página, tema, scroll) os que começam com
  `plev-showcase://` (`is_link`). antes, um link como primeiro argumento
  era tomado como título de página e ignorado calado; agora é link, ou
  recusa visível.
- a gravação (`PLEV_RECORD=1`): o `Observed` do `agent:link` sobre a
  galeria inteira na revisão vista (`Recorder::observed`, como a porta do
  agente grava) e cada estágio do receipt pelo sink do dispatcher, com
  `request.actor` = `agent:link`. nenhum `Input`: nada foi clicado nem
  digitado.
- nenhuma página, nenhum arquivo da view, nenhuma projeção mudou: a linha
  vai para o `shell:notice` que o item 5 já projetava (`set_notice`) e o
  toast é o `ToastManager` da view. `view/dispatch/shell.rs` não precisou
  mudar.

### testes

- engine, `platform/links/tests.rs` (12): os válidos (id com chave crua e
  codificada, valor com `%20`, UTF-8, `+`, vazio, esquema em outra
  caixa); 28 malformados, cada um com o seu `LinkError` (sem `://`, outro
  esquema, fragmento, usuário, senha, porta, barra a mais, sem id, sem
  ação, ação fora de `[a-z_]`, `?` vazio, `?value` sem `=`, chave
  desconhecida, chave a mais); o id inválido (caixa, segmento vazio,
  chave sem fechar, `%20`); o caractere cru fora do ASCII imprimível e o
  byte onde está; o percent inválido, o UTF-8 inválido, o controle e a
  direção (`%0A`, `%0D`, `%00`, `%7F`, `%C2%85`, `%E2%80%AE`, `%E2%81%A6`);
  2048 aceito e 2049 recusado, e o tamanho olhado antes de tudo; a forma
  de volta (`url`) e a volta dela; `is_link` sem pânico de fronteira de
  char; `excerpt`; o request por `agent:link`, nunca `Human`, sem `ask`;
  a lista por id e ação; cada erro com texto próprio numa linha.
- engine, `platform/links/source.rs` (5): os argumentos que começam como
  link, em ordem, uma vez, os outros deixados; um argumento que não é
  UTF-8 é recusado, nunca pulado; a recusa é uma linha curta (o longo, o
  `\n`, o `\u{202e}`); o falso em ordem e compartilhado pelo handle;
  `NoLinks` e o `default_links` deste processo (nenhum link nos
  argumentos do harness).
- `rust/engine/tests/links.rs` (5, binário novo), um app de teste pelo
  `Dispatcher` com o `FakeLinks`: o link roda por `agent:link` onde o app
  abriu e a lista vem antes; com qualquer lista a policy vale como para
  agente (`NotAuthorized` no só-de-pessoa, `InvalidParam` fora das opções
  e acima do `max_len`); a confirmação que a policy pede de agente fica
  `AwaitingConfirmation`, o link não confirma (`ConfirmByMachine`), a
  pessoa confirma e roda; o link que viu um estado velho é `Stale`, e o
  de uma capacidade que saiu não faz request (`UnknownAction`); o que não
  é link deste app não faz request nenhum.
- showcase, `app/links/tests.rs` (7), o shell sobre janela nua com o
  `FakeLinks` e um sink no dispatcher da galeria: um link de `shell.page`
  troca a página para Files pelo dispatcher, quatro estágios por
  `agent:link`, frame pedido, nada contado; tema e "Data%20Table" também;
  `capture.camera` (com um `FakeCapture`: 0 starts), `shell.print`,
  `files.open` `open_file`, `todo.item[1]` `remove`, um id que não existe,
  um fragmento e uma chave desconhecida são recusados sem request nenhum,
  a página igual, e a linha exata em `shell:notice` e num toast; dentro
  da lista, `shell.theme` com valor fora das opções é recusado pelo
  dispatcher (`requested`, `rejected`) e a linha diz; uma lista descuidada
  com `shell.print`, `files.open` e `capture.camera` dá `NotAuthorized`
  por `agent:link` nos três e nada roda; um link que chega dormindo não é
  lido em três polls (fica na fonte, nenhum frame) e é aplicado no
  primeiro poll depois do `wake`; o argumento de lançamento é link e não
  título (`launch_view`), com o tema depois dele no lugar, e a
  `LaunchLinks` dos mesmos argumentos leva à Files e recusa
  `plev-showcase://shell.page` na tela; a gravação tem `observed` e os
  quatro `stage` de `agent:link` e nenhum `input`; a lista é a página e o
  tema, abertos a agente, não destrutivos, sem confirmação.
- `apps/uigallery/tests/links.rs` (2, GPU, binário novo), sobre um host
  headless 1200x800: o link da câmera na Capture é recusado sem request,
  o toast com a linha é desenhado (mais da metade dos pixels do retângulo
  do toast mudou) e nenhum pixel mudou fora dele e da margem da sombra
  (48 px), `capture:camera` segue "off"; o link da Files a partir da
  Todo desenha, byte a byte, o frame de uma galeria que foi para a Files
  pelo nome da página.

a prova do roteiro, só por teste e sem janela: o link da Files pelo
argumento é `the_launch_argument_is_a_link_and_never_the_page_title` e
`a_link_to_the_files_page_draws_that_page_byte_for_byte`; o da câmera
recusado e na tela é
`what_is_not_open_to_links_is_refused_nothing_runs_and_the_person_is_told`
e `a_link_to_the_camera_is_refused_and_its_line_is_on_screen`.

### gate e regress

`df -h /` antes de cada gate: 17 GB livres (base) e 16 GB (depois); 16 GB
no fim. `./tool/tasks/fullcheck.sh` verde no worktree com `CARGO_NET_OFFLINE=true`,
com a perna wasm e a perna urnaui (`../urna` presente): antes (a base
`bcf6be8e`, medida neste worktree) 2641 passed, 0 failed, 17 ignored em
128 binários; depois 2672 passed, 0 failed, 17 ignored em 130 binários
(+31: 12 da gramática, 5 das fontes, 5 de `engine/tests/links.rs`, 7 do
shell, 2 de `showcase/tests/links.rs`; +2 binários, os dois
`tests/links.rs`). perna urnaui: 85 passed, 0 failed, 1 ignored em 6
binários, igual a antes. clippy `-D warnings`, fmt e a checagem wasm
limpos nas duas pernas; o único aviso no log é o `block v0.1.6` de
incompatibilidade futura, que já estava. `Cargo.lock`,
`apps/urnadbapp/Cargo.lock`, `rust/engine/Cargo.toml` e o `Cargo.toml`
da raiz sem mudança (`git diff bcf6be8e` vazio neles).

`cargo run --offline -p regress -- check`: 163
baselines, 163 iguais, 0 mudaram, sem frame desenhado e sem GPU aberta
(743 ms). nenhuma projeção de página mudou; nenhum golden, nenhum bless.

linhas de produção (`file_hygiene_guard` verde): engine
`platform/links.rs` 382, `platform/links/source.rs` 212,
`platform/mod.rs` +1; showcase `app.rs` 596 (era 571), `app/links.rs`
208. tudo abaixo de 639.

### ADR

`docs/adr/interface/0063-a-link-is-not-a-person.md` (proposed): a gramática, o ator, a
lista aberta, a recusa vista, as fontes, o caminho por plataforma hoje, o
shell, a gravação, e o bring-up (o `CFBundleURLTypes` no `.app` e o
handler `GURL` pelo `NSNotificationCenter` com as três dependências, a
chave do registro do windows, o `.desktop` do linux, a segunda
instância, iOS e android). linha para o índice (`docs/adr/README.md` está
fora da lista, fica para a integração):

```text
| 2026-09-24 | [a-link-is-not-a-person](a-link-is-not-a-person.md) | proposed | task-76 item 6: `engine::platform::links` lê `<esquema>://<sem id>/<ação>[?value=<texto>]` com gramática estrita (no máximo 2048 bytes de ASCII imprimível, o esquema do app, o id por `SemId::parse` depois do percent-decoding, uma `/`, ação de `[a-z_]`, só a chave `value`, texto UTF-8 sem controle nem marca de direção; sem fragmento, usuário, senha, porta ou barra a mais; `LinkError` diz por quê); o request de um link é `Request::observe` por `Actor::Agent` de nome `link`, nunca `Actor::Human` e sem ator novo no `sem`, e o app só o pede para o que lista como aberto a link (`Link::is_in`), a policy valendo por cima como para agente; `LinkSource` com `LaunchLinks` (os argumentos que começam com `<esquema>://`, em todo desktop), `FakeLinks`, `NoLinks` (wasm e mobile), `default_links`; no macOS o link chega só pelo argumento até o bring-up, porque o sistema entrega o esquema como evento `GURL` a um `.app` registrado e o delegado do winit 0.30.13 entra em pânico se for trocado: o handler `GURL` pelo `NSNotificationCenter`, com `objc2`, `objc2-foundation` e `objc2-app-kit` diretos (três linhas do lock), é do bring-up; o showcase (`plev-showcase`) abre só `shell.page` e `shell.theme` `select`, recusa o resto sem request, conta a recusa numa linha como toast e em `shell:notice`, lê o link de lançamento com a janela de pé (nunca como título de página), deixa o link dormindo na fonte até o wake e grava o link como a observação e o receipt de `agent:link`, sem input |
```

### interfaces adicionadas (só crescimento)

- `engine::platform::links::{Link, LinkError, MAX_LEN, ACTOR, actor,
  is_link, excerpt, Refused, Delivered, read, LinkSource, LaunchLinks,
  FakeLinks, NoLinks, default_links}`; `impl LinkSource for Box<L>` e
  `for Arc<Mutex<L>>`; `engine::platform::links` em `platform/mod.rs`
  (uma linha).
- showcase: `showcase::app::links::{SCHEME, OPEN, Followed, follow}`
  (público); `UserEvent::Link` (privado do app); `App::{install_links,
  set_links, links_arrived}` (`pub(super)`), `App::launch_view`
  (privada; `App::new` tem a mesma assinatura e o mesmo comportamento
  para quem não passa link). nenhuma assinatura pública existente mudou;
  ide, urnaui, wasm e os exemplos compilam sem tocar.

### arquivos tocados fora da lista permitida, e por quê

- `apps/uigallery/src/shell/system/links/tests.rs`: os testes do módulo novo
  `app/links.rs`, num arquivo à parte como `shell/tests.rs` (o guard não
  conta arquivo `tests.rs`). a lista nomeava `app/links.rs` e
  `apps/uigallery/tests/**`.
- nenhum outro. `app/entry.rs` e `view/dispatch/shell.rs` não precisaram
  mudar (o proxy já está no `Access` antes do `run_app`; o `set_notice`
  já era público).

### limitações e decisões, sem esconder

- nenhum link de verdade foi aberto nesta máquina e nenhum esquema foi
  registrado: tudo é o falso, a `LaunchLinks` com argumentos montados no
  teste, e o `follow` sobre a view. o `cargo run -- <link>` com janela é
  do brenner (roteiro abaixo).
- o macOS não recebe link do sistema até o bring-up (a decisão). o texto
  do winit que o briefing leu não vale para a 0.30.13; o pânico de um
  delegado trocado é leitura da fonte, não rodado.
- a lista vem antes da policy de propósito: um link fora da lista não
  vira request, então não deixa receipt nem estágio na gravação (que
  guarda a projeção da página, não o `shell:notice` da galeria); o log,
  o toast e o `shell:notice` dizem. a alternativa (despachar tudo e
  deixar a policy recusar) aceitaria `todo.item[n] remove`, que a policy
  deixa a um agente, vindo de qualquer página web.
- dentro da lista, uma confirmação pedida pela policy espera uma pessoa.
  o showcase não tem hoje resposta de pessoa para confirmação de agente
  (o ide tem o modal e o diálogo nativo do item 8): o request ficaria
  aberto. a lista de hoje não tem nada que peça confirmação (testado).
- o esquema é comparado sem diferença de caixa (RFC 3986): `PLEV-SHOWCASE://`
  é link. o `+` do valor é mais, não espaço: quem monta o link escreve
  `%20` (o `URLSearchParams` do navegador escreve `+`; um título com
  espaço vindo dele seria recusado como opção desconhecida, dito na
  tela).
- o `USAGE` do binário (`src/main.rs`, fora da lista) não fala do link;
  um link como argumento cai no modo janela porque não começa com `--`.
- o toast da recusa some depois da duração do `ToastManager`;
  `shell:notice` fica até a próxima coisa contada.
- `showcase::app::links` é público para o teste de integração; a outra
  forma (o teste dentro do crate) não teria o frame headless com GPU no
  mesmo estilo do `tests/sleep_wake.rs`.

### o roteiro da prova real (do brenner, nunca de teste)

hoje, pelo argumento (nada registrado, nenhum `open`):

```text
RUST_LOG=showcase=info cargo run --offline -p uigallery -- 'plev-showcase://shell.page/select?value=Files'
   a janela abre na Files. no log: "link: plev-showcase://shell.page/select?value=Files: completed".
cargo run --offline -p uigallery -- 'plev-showcase://capture.camera/toggle'
   a janela abre na página de sempre, a câmera não liga (nenhuma luz, nenhum ffmpeg:
   pgrep -fl "ffmpeg.*avfoundation" vazio), e um toast embaixo à direita diz
   "link refused: plev-showcase://capture.camera/toggle: capture.camera toggle is not open to links";
   o log diz o mesmo em warn.
cargo run --offline -p uigallery -- Todo dark 'plev-showcase://shell.theme/select?value=nope'
   a Todo no tema dark (o link não tomou o lugar de nenhum argumento), e o toast
   "link refused: ...: "nope" is not one of the options".
PLEV_AGENT=1 cargo run --offline -p uigallery -- 'plev-showcase://shell.print/activate'
   e em outro terminal: cargo run --offline -p socket --bin plev -- observe --scope shell
   shell:notice = "link refused: plev-showcase://shell.print/activate: shell.print activate is not open to links"
PLEV_RECORD=1 cargo run --offline -p uigallery -- 'plev-showcase://shell.page/select?value=Files'
   cmd-shift-s, e o plev-inspect da sessão: o observed e o receipt de agent:link, nenhum input antes dele.
```

depois do bring-up do bundle (o `.app` com `CFBundleURLTypes` de
`plev-showcase` e o handler `GURL` no engine):

```text
open 'plev-showcase://shell.page/select?value=Files'     (o app fechado: abre na Files; aberto: a janela viva vai para a Files, sem segunda instância)
open 'plev-showcase://capture.camera/toggle'              (recusado na tela, a câmera apagada)
um link numa página web (<a href="plev-showcase://shell.theme/select?value=dark">) clicado no navegador: o tema muda; o navegador pergunta antes de abrir o app
desfazer: tirar o .app de /Applications (o Launch Services esquece o esquema), ou
   /System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -u <o .app>
```

o que olhar: nenhum link liga câmera, microfone, impressora, diálogo ou
janela; toda recusa aparece na tela e em `shell:notice`; o link que
lançou o app é aplicado uma vez.

### comandos de validação

```text
export CARGO_TARGET_DIR=/Users/nn/Dev/hoff/plevdev/target/agents/one
export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_PROFILE_TEST_DEBUG=line-tables-only
export CARGO_NET_OFFLINE=true
cargo test --offline -p engine --lib platform::links
cargo test --offline -p engine --test links
cargo test --offline -p uigallery --lib app::links
cargo test --offline -p uigallery --test links
cargo test --offline -p engine -p uigallery
./tool/tasks/fullcheck.sh
cargo run --offline -p regress -- check   (163 iguais)
git diff bcf6be8e -- Cargo.lock apps/urnadbapp/Cargo.lock rust/engine/Cargo.toml Cargo.toml   (vazio)
```

## fecho da task-76

a task-76 tinha dez itens: nove estão na `main`, e o sexto é esta
branch, à espera da integração. o que cada um deixou e o que fica para o
bring-up de plataforma (bundle macOS, linux e windows como alvo, mobile),
item por item:

1. perda de device de GPU (`gpu-device-loss-recovers-in-place`). entrou:
   o callback de device lost em todo `GpuContext`, a recriação no lugar
   (adapter, device, surface, pipelines, atlas, imagens pelo log de
   uploads) antes do acquire, o frame pulado com um erro por perda,
   provado headless com `Device::destroy` e pixels iguais antes e depois.
   fica: a perda real (driver, sleep, GPU trocada) nunca provocada numa
   máquina; wasm só sinaliza, não recupera; o `recreate_surface` do
   android pelo caminho próprio.
2. múltiplas janelas (`windows-are-hosts-the-app-owns-the-state`).
   entrou: `WindowHost` e `Windows` no engine (um host por janela, rota
   por id, fechar a última sai ou não), a página do showcase numa janela
   própria (`D`, `shell.detach`) com o mesmo estado e o mesmo
   dispatcher. fica: a janela real vista pelo brenner; o `source` dos nós
   com a janela aberta (um `SnapshotBuilder::extend` no `sem`); a janela
   da página sem acessibilidade própria; wasm e mobile com uma janela.
3. diálogos de arquivo e drag and drop (`file-dialogs-and-drops-are-capabilities`).
   entrou: `engine::platform::files` (o `rfd` do engine, falso, nenhum),
   o urnaui migrado sem segunda implementação, o drop como evento do
   engine, a página Files. fica: o backend do linux (portal ou gtk3), o
   diálogo assíncrono do wasm (bytes), a posição do drop (o winit 0.30
   não dá), o painel real aberto pelo brenner.
4. menus nativos (`native-menus-are-another-view-of-capabilities`).
   entrou: o `MenuModel` puro a partir da projeção, o `muda` no desktop,
   o clique como o request do botão por `Actor::Human`, o shell projetado
   (`shell.page`, `shell.theme`, `shell.detach`, `shell.save_session`) e a
   barra do showcase. fica: a perna windows (`init_for_hwnd`,
   `TranslateAcceleratorW`) compilada e provada; o linux (backend `noop`
   do `muda`); o clique num item real visto pelo brenner.
5. notificações e tray (`tray-and-notifications-are-capabilities-out-of-the-window`).
   entrou: `TrayModel` e `NativeTray` (`tray-icon`, macOS e windows), a
   fila de cliques dividida com a barra, `Notification` com uma `Action`
   que volta como o request do botão, `shell.show`, `shell:notice`, a
   Files e a Silences contando o que aconteceu sem ninguém olhando. fica:
   a notificação real no macOS (UserNotifications quer o bundle:
   `NeedsBundle` hoje), o tray do linux, a perna windows provada, viver só
   no tray.
6. deep links (`a-link-is-not-a-person`, este). entrou: a gramática
   estrita, o ator `agent:link` sem mudar o `sem`, a lista aberta do app,
   a recusa na tela e em `shell:notice`, as fontes (argumento em todo
   desktop, falso, nenhum), o showcase (`plev-showcase`, só página e
   tema), a espera pelo wake, a gravação. fica: no macOS o `.app` com
   `CFBundleURLTypes` e o handler `GURL` pelo `NSNotificationCenter` (três
   dependências diretas, três linhas do lock); o registro do windows e o
   `.desktop` do linux; encaminhar o link da segunda instância para a que
   roda; iOS e android pela entrada deles; a prova com `open` depois do
   bundle.
7. sleep e wake (`a-covered-window-does-not-draw-and-wakes-new`). entrou:
   coberta e dormindo por host e por app, nenhum frame nem deadline
   enquanto isso, um frame novo ao mostrar ou acordar, os relógios que
   saltam, o `Background` para o trabalho ao lado dos frames (o Claude da
   Surface retido, o ffmpeg esperando). fica: o sleep do sistema no macOS
   (`NSWorkspace` will sleep e did wake, feature ainda não ligada), o
   `suspended` do android soltando as surfaces, o adiar do job novo do
   ffmpeg dormindo.
8. diálogos nativos de alerta e confirmação (`native-dialogs-answer-the-same-contract`).
   entrou: `engine::platform::alerts` (o `MessageDialog` do `rfd`, falso,
   nenhum), a confirmação humana da task-58 pelo diálogo nativo como opção
   do ide (`PLEV_NATIVE_CONFIRM=1`), o alerta só de pessoa na Files. fica:
   o `AsyncMessageDialog` (o diálogo bloqueia o laço), rótulos no windows
   (`common-controls-v6`), linux, o sheet real visto pelo brenner.
9. impressão (`printing-is-the-frame-on-paper`). entrou: o frame em PNG a
   300 dpi pelo render headless, o CUPS por filho (`lp`, `lpstat`),
   `shell.print` só de pessoa, cmd-p e o menu, o job ao lado do laço. fica:
   o windows (spooler), o painel nativo (`NSPrintPanel`), o PDF vetorial,
   paginação, o poll da fila depois do spool, a impressão real do brenner.
10. câmera e microfone (`capture-is-a-job-the-person-turns-on`). entrou:
    `engine::platform::capture` com o ffmpeg por filho, a permissão como
    estado, a imagem de fluxo no atlas, a página Capture só de pessoa, o
    dispositivo solto na hora quando coberto, dormindo ou fora da página.
    fica: AVFoundation direto e o `.app` com as `UsageDescription`, os
    nomes do dshow no windows, v4l2 e alsa numa máquina linux real, as
    APIs do mobile, a luz da câmera vista pelo brenner.

o que atravessa a task e fica junto para o bring-up: o bundle do macOS
destrava três itens de uma vez (a notificação do 5, o link do 6 e o
processo responsável pela câmera do 10); o linux precisa de uma escolha
de backend em três crates (`rfd`, `muda`, `tray-icon`); o windows
precisa compilar as pernas escritas e não compiladas (menu, tray, dshow);
e as provas na tela de cada item estão nos roteiros do brenner, nenhuma
feita por agente. os critérios de aceitação da task (por item: macOS
funcionando, a variante mobile ou web registrada, teste automático onde
dá, roteiro onde não dá; device lost sobrevive a uma perda forçada) estão
cumpridos no que o agente pode provar; o "funcionando no macOS" com
janela, dispositivo, impressora e sistema de verdade é o que os roteiros
pedem.
