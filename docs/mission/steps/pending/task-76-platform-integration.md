---
project: plev
audience: [ai-agents, contributors]
status: pending
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
