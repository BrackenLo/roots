//====================================================================

use std::sync::{Arc, RwLock};

use hecs::World;
use roots_common::Size;
use roots_renderer::{
    lighting::LightingManager, shared::SharedRenderResources, texture::Texture, Color, Device,
    Queue, RenderCore, RenderEncoder, RenderPassDesc, Surface, SurfaceConfig, SurfaceError,
};
use roots_runner::window::Window;

pub mod components;
pub mod pipelines;

//====================================================================

pub struct RendererState {
    pub device: Device,
    pub queue: Queue,
    pub surface: Surface<'static>,
    pub config: SurfaceConfig,

    pub shared: SharedRenderResources,
    pub lighting: LightingManager,
    depth_texture: Texture,

    pub clear_color: Color,

    managed_pipelines: Arc<RwLock<Vec<ManagedPipeline>>>,
}

impl RendererState {
    pub fn new(window: &Window) -> Self {
        log::info!("Creating renderer");
        let (device, queue, surface, config) =
            pollster::block_on(RenderCore::new(window.arc(), window.size()))
                .unwrap()
                .break_down();

        let shared = SharedRenderResources::new(&device);
        let lighting = LightingManager::new(&device);
        let depth_texture = Texture::create_depth_texture(&device, window.size(), None);

        Self {
            device,
            queue,
            surface,
            config,
            shared,
            lighting,
            depth_texture,
            clear_color: Color::new(0.2, 0.2, 0.2, 1.),
            managed_pipelines: Arc::default(),
        }
    }

    pub fn resize(&mut self, size: Size<u32>) {
        log::trace!("Resizing window with size {}", size);

        if size.width == 0 || size.height == 0 {
            log::warn!(
                "Invalid Window size. Must be non-zero. New Window size = {}",
                size
            );
            return;
        }

        self.config.width = size.width;
        self.config.height = size.height;

        self.surface.configure(&self.device, &self.config);

        self.depth_texture = Texture::create_depth_texture(&self.device, size, None);
    }

    #[inline]
    pub fn create_encoder(&self) -> Result<RenderEncoder, SurfaceError> {
        let encoder = match RenderEncoder::new(&self.device, &self.surface) {
            Ok(encoder) => encoder,
            Err(e) => {
                log::warn!("Unable to get surface this frame");
                return Err(e);
            }
        };

        Ok(encoder)
    }

    pub fn add_managed_pipeline<P: pipelines::Pipeline>(&mut self, priority: usize) {
        let pipeline = Box::new(P::new(&self));

        let mut managed_pipelines = self.managed_pipelines.write().unwrap();

        managed_pipelines.push(ManagedPipeline { priority, pipeline });
        managed_pipelines.sort_by_key(|val| val.priority);
    }

    pub fn prep_managed(&mut self, world: &mut World) {
        self.managed_pipelines
            .write()
            .unwrap()
            .iter_mut()
            .for_each(|pipeline_data| pipeline_data.pipeline.prep(self, world));
    }

    pub fn render(&mut self, world: &mut World) {
        let mut encoder = match self.create_encoder() {
            Ok(encoder) => encoder,
            Err(_) => return,
        };

        let mut render_pass = encoder.begin_render_pass(RenderPassDesc {
            use_depth: Some(&self.depth_texture.view),
            clear_color: Some(self.clear_color),
        });

        self.managed_pipelines
            .write()
            .unwrap()
            .iter_mut()
            .for_each(|pipeline_data| pipeline_data.pipeline.render(&mut render_pass, self, world));
    }
}

//====================================================================

pub struct ManagedPipeline {
    priority: usize,
    pipeline: Box<dyn pipelines::Pipeline>,
}

//====================================================================
