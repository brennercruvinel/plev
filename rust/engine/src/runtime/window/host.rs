//! One window's worth of engine, grouped: what dies with the window.
//!
//! [`WindowHost`] holds the window and the GPU context that draws into it
//! (the surface belongs to that context, and so does its device loss and
//! recovery), the compositor whose layers and bindings are that window's
//! frame, the input state (hit regions, hover, focus, the cursor), the
//! touch tracker and the pointer synth, the IME, the scale factor and the
//! safe area, the frame clock and the perf monitor, and the events
//! buffered for the next frame. What is not here is the app's: the theme,
//! the state and its dispatcher, the lifecycle, the event loop proxy.
//! [`super::Windows`] keeps several hosts by `WindowId`; the engine `App`
//! keeps one through it.
//!
//! A host is built in steps so every target fits: [`WindowHost::bare`]
//! has neither window nor device (a scene and an input that never draw:
//! tests without a GPU); [`attach_window`](WindowHost::attach_window) gives
//! it the window and [`attach_gpu`](WindowHost::attach_gpu) the device. On
//! native targets [`WindowHost::open`] does all three, blocking on the
//! adapter as the constructors always did; on wasm the shell spawns the
//! device request and attaches the result. [`WindowHost::headless`] is a
//! host without a window drawing into a texture, for the tests that prove
//! two windows apart.
//!
//! Render on demand is per host: a frame is drawn only for the window
//! whose `request_redraw` was called, and [`needs_redraw`](WindowHost::needs_redraw)
//! says whether this one has a reason (a change, an animation, a pending
//! request); a host at rest costs nothing while another animates. A host
//! that is covered, or whose app sleeps, asks for nothing and has no
//! deadline until it shows again, and then its first frame is new
//! (`sleep.rs`).

use std::sync::Arc;

use web_time::Instant;
use winit::window::{Window, WindowId};

use crate::animation::{AnimationTick, FrameClock};
use crate::compositor::Compositor;
use crate::effects::EffectProcessor;
use crate::gpu::GpuContext;
#[cfg(not(target_arch = "wasm32"))]
use crate::gpu::RenderConfig;
use crate::gpu::texture_pool::TexturePool;
use crate::input::{InputState, TouchInputState, TouchPointerSynth};
use crate::perf::{PerfHud, PerfMonitor};
use crate::platform::SafeAreaInsets;
use crate::platform::ime::ImeState;
use crate::text::TextSystem;

use super::events::{BufferedEvent, logical_projection_size};

/// What the device holds for one window: the context and the three
/// resources every frame draws with beside it. All four die with the
/// device and are rebuilt together by `gpu::recover_frame_resources`.
pub(crate) struct Gpu {
    pub(crate) gpu: GpuContext,
    pub(crate) text_system: TextSystem,
    pub(crate) effects: EffectProcessor,
    pub(crate) texture_pool: TexturePool,
}

impl Gpu {
    fn new(gpu: GpuContext) -> Self {
        let text_system = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
        let effects = EffectProcessor::new(&gpu.device, gpu.surface_format());
        Self {
            gpu,
            text_system,
            effects,
            texture_pool: TexturePool::new(),
        }
    }
}

pub struct WindowHost {
    /// `None` for a bare or headless host, and while suspended (android).
    pub(crate) window: Option<Arc<Window>>,
    /// `None` until the device is ready (wasm asks for it asynchronously).
    pub(crate) gpu: Option<Gpu>,
    /// A headless host's target, where a window has its surface.
    pub(crate) target: Option<wgpu::Texture>,
    pub(crate) compositor: Compositor,
    input: InputState,
    pub(crate) touch: TouchInputState,
    pub(crate) touch_pointer: TouchPointerSynth,
    pub(crate) ime: ImeState,
    safe_area: SafeAreaInsets,
    scale_factor: f64,
    /// The surface size in physical pixels, kept here too so a host without
    /// a device still knows its logical size.
    size: (u32, u32),
    clock: FrameClock,
    pub(crate) tick: AnimationTick,
    pub(crate) perf: PerfMonitor,
    pub(crate) perf_hud: PerfHud,
    pub(crate) events: Vec<BufferedEvent>,
    animating: bool,
    /// A frame was asked for and not drawn yet. A host with a window asks
    /// the platform too; one without keeps the flag for its caller.
    pub(crate) redraw_asked: bool,
    /// When this window next needs a frame with nothing else happening (a
    /// caret's flip); the loop sleeps until the earliest of all hosts.
    deadline: Option<Instant>,
    /// The platform says the window is not visible at all (minimized, or
    /// fully covered): `WindowEvent::Occluded(true)`.
    pub(crate) occluded: bool,
    /// The app sleeps (`Windows::sleep`, or suspended): every host of it.
    pub(crate) sleeping: bool,
    /// Frames drawn so far.
    pub(crate) frames: u64,
    #[cfg(all(feature = "accessibility", not(target_arch = "wasm32")))]
    pub(crate) a11y: Option<crate::accessibility::Bridge>,
}

impl Default for WindowHost {
    fn default() -> Self {
        Self::bare()
    }
}

impl WindowHost {
    /// A host with neither window nor device: the scene, the input and the
    /// clock of a window that never draws. A test builds frames into its
    /// compositor without a GPU; `attach_window` and `attach_gpu` turn it
    /// into a real one.
    pub fn bare() -> Self {
        Self {
            window: None,
            gpu: None,
            target: None,
            compositor: Compositor::new(),
            input: InputState::new(),
            touch: TouchInputState::new(),
            touch_pointer: TouchPointerSynth::new(),
            ime: ImeState::new(),
            safe_area: SafeAreaInsets::default(),
            scale_factor: 1.0,
            size: (0, 0),
            clock: FrameClock::new(),
            tick: AnimationTick {
                dt: 0.0,
                elapsed: 0.0,
                since_last: 0.0,
            },
            perf: PerfMonitor::new(),
            perf_hud: PerfHud::new(),
            events: Vec::with_capacity(64),
            animating: false,
            redraw_asked: false,
            deadline: None,
            occluded: false,
            sleeping: false,
            frames: 0,
            #[cfg(all(feature = "accessibility", not(target_arch = "wasm32")))]
            a11y: None,
        }
    }

    /// A host for `window`, its device still to come (`attach_gpu`).
    pub fn new(window: Arc<Window>) -> Self {
        let mut host = Self::bare();
        host.attach_window(window);
        host
    }

    /// A host for `window` with its device, blocking on the adapter and
    /// device requests as `GpuContext::new` does. Native only; the browser
    /// cannot block inside its event loop.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn open(window: Arc<Window>) -> Self {
        Self::open_with_config(window, RenderConfig::default())
    }

    /// [`open`](Self::open) with a render config.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn open_with_config(window: Arc<Window>, config: RenderConfig) -> Self {
        let mut host = Self::new(window.clone());
        let gpu = pollster::block_on(GpuContext::new_with_config(window, config));
        host.attach_gpu(gpu);
        host
    }

    /// A host without a window, drawing into a texture of `width x height`
    /// physical pixels at `scale` (the frame path `engine::headless` uses).
    /// `None` without a GPU adapter, so tests can skip.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn headless(width: u32, height: u32, scale: f64, config: RenderConfig) -> Option<Self> {
        let gpu = pollster::block_on(GpuContext::new_headless(width, height, config))?;
        let mut host = Self::bare();
        host.scale_factor = scale;
        host.size = (width.max(1), height.max(1));
        host.attach_gpu(gpu);
        Some(host)
    }

    /// Gives a bare host its window: the scale factor, the safe area and
    /// the size come from it. A device attached already is resized to it.
    pub fn attach_window(&mut self, window: Arc<Window>) {
        self.scale_factor = window.scale_factor();
        self.safe_area = SafeAreaInsets::from_window(&window);
        let size = window.inner_size();
        self.size = (size.width.max(1), size.height.max(1));
        self.window = Some(window);
        if self.gpu.is_some() {
            self.resize(self.size.0, self.size.1);
        }
    }

    /// Gives the host its device: the text system, the effects and the
    /// texture pool are built on it, the logical projection applied, and
    /// a frame asked for. A headless host gets its target texture.
    pub fn attach_gpu(&mut self, gpu: GpuContext) {
        let mut gpu = Gpu::new(gpu);
        let (w, h) = (gpu.gpu.surface_config.width, gpu.gpu.surface_config.height);
        if self.window.is_none() {
            self.size = (w, h);
        }
        let (lw, lh) = logical_projection_size(self.size.0, self.size.1, self.scale_factor);
        gpu.gpu.set_projection(lw, lh);
        if self.window.is_none() {
            self.target = Some(create_target(&gpu.gpu));
        }
        self.gpu = Some(gpu);
        // In the browser the device arrives after the canvas may have
        // been sized (a `Resized` taken while the adapter was requested,
        // with no surface to resize), so the surface configured at 0x0
        // follows the window's size now; left alone it drew the whole
        // page into one pixel stretched over the canvas.
        if self.window.is_some() && (w, h) != self.size {
            self.resize(self.size.0, self.size.1);
        }
        self.compositor.invalidate();
        self.request_redraw();
    }

    pub fn window(&self) -> Option<&Arc<Window>> {
        self.window.as_ref()
    }

    /// The platform's id of the window; `None` for a bare or headless host.
    pub fn id(&self) -> Option<WindowId> {
        self.window.as_ref().map(|w| w.id())
    }

    pub fn has_gpu(&self) -> bool {
        self.gpu.is_some()
    }

    pub fn gpu(&self) -> Option<&GpuContext> {
        self.gpu.as_ref().map(|g| &g.gpu)
    }

    pub fn gpu_mut(&mut self) -> Option<&mut GpuContext> {
        self.gpu.as_mut().map(|g| &mut g.gpu)
    }

    pub fn text_system(&self) -> Option<&TextSystem> {
        self.gpu.as_ref().map(|g| &g.text_system)
    }

    pub fn compositor(&self) -> &Compositor {
        &self.compositor
    }

    pub fn compositor_mut(&mut self) -> &mut Compositor {
        &mut self.compositor
    }

    pub fn input(&self) -> &InputState {
        &self.input
    }

    pub fn input_mut(&mut self) -> &mut InputState {
        &mut self.input
    }

    pub fn ime(&self) -> &ImeState {
        &self.ime
    }

    pub fn safe_area(&self) -> SafeAreaInsets {
        self.safe_area
    }

    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    /// The surface size in physical pixels.
    pub fn physical_size(&self) -> (u32, u32) {
        self.size
    }

    /// The size scenes are laid out in: physical over the scale factor.
    pub fn logical_size(&self) -> (f32, f32) {
        logical_projection_size(self.size.0, self.size.1, self.scale_factor)
    }

    /// The last frame's tick.
    pub fn tick(&self) -> AnimationTick {
        self.tick
    }

    /// Frames drawn so far by this host.
    pub fn frames(&self) -> u64 {
        self.frames
    }

    /// The window's surface changed size (physical pixels): the surface is
    /// reconfigured, the logical projection reapplied (`gpu.resize` resets
    /// it to physical, which on HiDPI would draw everything at half size),
    /// the scene invalidated and a frame asked for.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.size = (width.max(1), height.max(1));
        if let Some(gpu) = self.gpu.as_mut() {
            gpu.gpu.resize(width, height);
            let (lw, lh) = logical_projection_size(width, height, self.scale_factor);
            gpu.gpu.set_projection(lw, lh);
            if self.window.is_none() {
                self.target = Some(create_target(&gpu.gpu));
            }
        }
        if let Some(window) = &self.window {
            self.safe_area = SafeAreaInsets::from_window(window);
        }
        self.compositor.invalidate();
        self.request_redraw();
    }

    /// The window moved to a display with another scale: the projection
    /// follows, and a frame is asked for (a `Resized` usually follows too).
    pub fn scale_factor_changed(&mut self, scale_factor: f64) {
        self.scale_factor = scale_factor;
        if let Some(gpu) = self.gpu.as_mut() {
            let (lw, lh) = logical_projection_size(self.size.0, self.size.1, scale_factor);
            gpu.gpu.set_projection(lw, lh);
        }
        self.compositor.invalidate();
        self.request_redraw();
        log::info!("Scale factor changed: {scale_factor}");
    }

    /// Ask the platform for a frame of this window. A host without a
    /// window keeps the request for whoever drives it. A host that is not
    /// [`awake`](Self::awake) asks for nothing: the frame it needs is the
    /// one asked as it shows again.
    pub fn request_redraw(&mut self) {
        if !self.awake() {
            return;
        }
        self.redraw_asked = true;
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    /// A frame was asked for since the last one drawn.
    pub fn redraw_asked(&self) -> bool {
        self.redraw_asked
    }

    /// Whether this window has a reason to draw: something changed in its
    /// scene, a frame was asked for, it animates, or its perf HUD is on
    /// (the overlay's text changes every frame). Never while covered or
    /// asleep: the reasons wait for the window to show.
    pub fn needs_redraw(&self) -> bool {
        self.awake()
            && (self.animating
                || self.redraw_asked
                || self.compositor.needs_render()
                || self.perf_overlay_active())
    }

    /// Keep drawing this window every frame while `on`; off by default,
    /// frames come only on demand.
    pub fn set_animating(&mut self, on: bool) {
        self.animating = on;
        if on {
            self.request_redraw();
        }
    }

    pub fn is_animating(&self) -> bool {
        self.animating
    }

    pub(crate) fn perf_overlay_active(&self) -> bool {
        self.gpu.as_ref().is_some_and(|g| g.gpu.config.perf_hud)
    }

    /// When this window next needs a frame on its own (a caret's flip, a
    /// toast's dismissal); `None` when nothing is pending. The shell that
    /// paces the loop reads the earliest across windows.
    pub fn set_deadline(&mut self, at: Option<Instant>) {
        self.deadline = at;
    }

    /// `None` while the window is covered or the app sleeps: it needs no
    /// frame of its own until it shows, and the frame it shows with sets
    /// the next.
    pub fn deadline(&self) -> Option<Instant> {
        if self.awake() { self.deadline } else { None }
    }

    /// The start of a frame: the clock ticks, the pending request is
    /// taken as served, the IME's per-frame text is cleared.
    pub fn begin_frame(&mut self) -> AnimationTick {
        self.tick = self.clock.tick();
        self.redraw_asked = false;
        self.ime.begin_frame();
        self.tick
    }

    /// Present the scene already built into the compositor: the shared
    /// frame tail (`present_frame`) for a window, `draw_frame` into the
    /// target for a headless host. `None` when there is no device yet, no
    /// surface, or the frame was skipped (a lost device with no adapter
    /// answering; `present_frame`). A lost device is recovered here first.
    pub fn present(&mut self, clear_linear: [f32; 4], label: &str) -> Option<u32> {
        // `present_frame` polls and recovers a window's device itself; a
        // headless host's frame path is `draw_frame`, so the poll is here.
        let headless = self.gpu.as_ref()?.gpu.surface.is_none();
        if headless && self.device_lost() && !self.recover_or_skip(label) {
            return None;
        }
        let gpu = self.gpu.as_mut()?;
        let draws = if gpu.gpu.surface.is_some() {
            super::present_frame(
                &mut self.compositor,
                &mut gpu.gpu,
                &mut gpu.text_system,
                &mut gpu.effects,
                &mut gpu.texture_pool,
                clear_linear,
                label,
            )
        } else {
            let target = self.target.as_ref()?;
            let view = gpu.gpu.texture_render_view(target);
            Some(super::draw_frame(
                &mut self.compositor,
                &mut gpu.gpu,
                &mut gpu.text_system,
                &mut gpu.effects,
                &mut gpu.texture_pool,
                &view,
                clear_linear,
                label,
            ))
        };
        if draws.is_some() {
            self.frames += 1;
        }
        draws
    }

    /// The device is gone since the last frame (`GpuContext::poll_device_loss`);
    /// `false` without a device.
    pub fn device_lost(&self) -> bool {
        self.gpu.as_ref().is_some_and(|g| g.gpu.poll_device_loss())
    }

    /// Recover a lost device before a frame: everything on it is rebuilt
    /// (`gpu::recover_or_skip`), and a headless target with it. `false`
    /// when no adapter answers yet: skip the frame.
    pub(crate) fn recover_or_skip(&mut self, label: &str) -> bool {
        let Some(gpu) = self.gpu.as_mut() else {
            return false;
        };
        let recovered = crate::gpu::recover_or_skip(
            &mut self.compositor,
            &mut gpu.gpu,
            &mut gpu.text_system,
            &mut gpu.effects,
            &mut gpu.texture_pool,
            label,
        );
        if recovered && self.window.is_none() {
            self.target = Some(create_target(&gpu.gpu));
        }
        recovered
    }

    /// The pixels of a headless host's last frame (`engine::headless::read_texture`).
    /// `Err(DeviceLost)` while the device is gone, `Err(NoAdapter)` for a
    /// host that has no target to read.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn read(&self) -> Result<crate::headless::Frame, crate::headless::HeadlessError> {
        use crate::headless::HeadlessError;
        let (Some(gpu), Some(target)) = (self.gpu.as_ref(), self.target.as_ref()) else {
            return Err(HeadlessError::NoAdapter);
        };
        if gpu.gpu.poll_device_loss() {
            let lost = gpu.gpu.lost_device().unwrap_or(crate::gpu::LostDevice {
                reason: wgpu::DeviceLostReason::Unknown,
                message: String::new(),
            });
            return Err(HeadlessError::DeviceLost(lost));
        }
        crate::headless::read_texture(&gpu.gpu, target)
    }

    /// The app went to the background and the platform takes the window
    /// (android): the surface is dropped, the device kept, the window let
    /// go. `resume` gives it a new one.
    pub fn suspend(&mut self) {
        if let Some(gpu) = self.gpu.as_mut() {
            gpu.gpu.drop_surface();
        }
        #[cfg(all(feature = "accessibility", not(target_arch = "wasm32")))]
        {
            // the adapter belongs to the window being dropped
            self.a11y = None;
        }
        self.window = None;
        log::info!("window suspended: surface dropped");
    }

    /// Back from `suspend` with a new window: the surface is recreated on
    /// the kept device, and a frame asked for.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn resume(&mut self, window: Arc<Window>) {
        if let Some(gpu) = self.gpu.as_mut() {
            gpu.gpu.recreate_surface(window.clone());
        }
        self.attach_window(window);
        self.compositor.invalidate();
        self.request_redraw();
    }

    /// The system is short of memory: the glyph caches go.
    pub fn purge_caches(&mut self) {
        if let Some(gpu) = self.gpu.as_mut() {
            gpu.text_system.purge_caches();
        }
    }

    /// The accessibility bridge of this window, attached while the window
    /// was still hidden (`Bridge::new`). Every window event is shown to
    /// it before the host handles it, and every frame's tree comes from
    /// this host's bindings.
    #[cfg(all(feature = "accessibility", not(target_arch = "wasm32")))]
    pub fn set_accessibility(&mut self, bridge: crate::accessibility::Bridge) {
        self.a11y = Some(bridge);
    }

    #[cfg(all(feature = "accessibility", not(target_arch = "wasm32")))]
    pub fn accessibility_mut(&mut self) -> Option<&mut crate::accessibility::Bridge> {
        self.a11y.as_mut()
    }

    /// Shows a window event to the bridge before the host handles it
    /// (the focus of the window, on some platforms more).
    #[cfg(all(feature = "accessibility", not(target_arch = "wasm32")))]
    pub(crate) fn accessibility_event(&mut self, event: &winit::event::WindowEvent) {
        if let (Some(bridge), Some(window)) = (self.a11y.as_mut(), self.window.as_ref()) {
            bridge.process_event(window, event);
        }
    }
}

/// The offscreen texture a headless host draws into: the context's size
/// and format, readable back (as `engine::headless` makes its own).
fn create_target(gpu: &GpuContext) -> wgpu::Texture {
    gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("window_host_target"),
        size: wgpu::Extent3d {
            width: gpu.surface_config.width,
            height: gpu.surface_config.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: gpu.surface_config.format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}
