use std::sync::Arc;

use winit::window::Window;

use crate::effects::EffectProcessor;
use crate::gpu::GpuContext;
use crate::gpu::texture_pool::TexturePool;
use crate::text::TextSystem;

#[cfg(target_arch = "wasm32")]
use super::AppEvent;

pub(crate) enum GpuState {
    Uninitialized,
    Ready {
        gpu: GpuContext,
        text_system: TextSystem,
        effect_processor: EffectProcessor,
        texture_pool: TexturePool,
    },
    Suspended {
        gpu: GpuContext,
        text_system: TextSystem,
        effect_processor: EffectProcessor,
        texture_pool: TexturePool,
    },
}

impl super::App {
    pub(crate) fn init_gpu(&mut self, window: Arc<Window>) {
        self.scale_factor = window.scale_factor();
        self.safe_area = crate::platform::SafeAreaInsets::from_window(&window);

        #[cfg(not(target_arch = "wasm32"))]
        {
            let gpu = pollster::block_on(GpuContext::new(window));
            let text_system = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
            let effect_processor = EffectProcessor::new(&gpu.device, gpu.surface_format());
            let texture_pool = TexturePool::new();
            self.state = GpuState::Ready {
                gpu,
                text_system,
                effect_processor,
                texture_pool,
            };
        }

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(proxy) = self.event_loop_proxy.take() {
                let window_clone = window;
                wasm_bindgen_futures::spawn_local(async move {
                    let gpu = GpuContext::new(window_clone).await;
                    let text_system = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
                    let effect_processor = EffectProcessor::new(&gpu.device, gpu.surface_format());
                    let texture_pool = TexturePool::new();
                    let _ = proxy.send_event(AppEvent::GpuReady {
                        gpu,
                        text_system,
                        effect_processor,
                        texture_pool,
                    });
                });
            }
        }
    }
}
