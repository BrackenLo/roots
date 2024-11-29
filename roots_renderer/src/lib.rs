//====================================================================

use std::ops::{Deref, DerefMut};

use roots_common::Size;
use wgpu::SurfaceTarget;

pub mod camera;
pub mod lighting;
pub mod model;
pub mod shared;
pub mod texture;
pub mod tools;

//====================================================================

pub struct Device(wgpu::Device);
impl Deref for Device {
    type Target = wgpu::Device;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Queue(wgpu::Queue);
impl Deref for Queue {
    type Target = wgpu::Queue;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Surface<'a>(wgpu::Surface<'a>);
impl<'a> Deref for Surface<'a> {
    type Target = wgpu::Surface<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct SurfaceConfig(wgpu::SurfaceConfiguration);
impl Deref for SurfaceConfig {
    type Target = wgpu::SurfaceConfiguration;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SurfaceConfig {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

//====================================================================

pub struct RenderCore<'a> {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'a>,
    pub config: wgpu::SurfaceConfiguration,
}

#[derive(thiserror::Error)]
pub enum CreateRendererError {
    UnableToRequestAdapter,
}

impl std::fmt::Debug for CreateRendererError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateRendererError::UnableToRequestAdapter => f.write_str("Unable to request adapter"),
        }
    }
}

impl std::fmt::Display for CreateRendererError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl<'a> RenderCore<'a> {
    pub async fn new(
        window: impl Into<SurfaceTarget<'a>>,
        window_size: Size<u32>,
    ) -> anyhow::Result<Self> {
        log::debug!("Creating core wgpu renderer components.");

        log::debug!("Window inner size = {:?}", window_size);

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window)?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .ok_or_else(|| CreateRendererError::UnableToRequestAdapter)?;

        log::debug!("Chosen device adapter: {:#?}", adapter.get_info());

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    #[cfg(target_arch = "wasm32")]
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                    ..Default::default()
                },
                None,
            )
            .await?;

        let surface_capabilities = surface.get_capabilities(&adapter);

        let surface_format = surface_capabilities
            .formats
            .iter()
            .find(|format| format.is_srgb())
            .copied()
            .unwrap_or(surface_capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: wgpu::PresentMode::AutoNoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        log::debug!("Successfully created core wgpu components.");

        Ok(Self {
            device,
            queue,
            surface,
            config,
        })
    }

    #[inline]
    pub fn break_down(self) -> (Device, Queue, Surface<'a>, SurfaceConfig) {
        (
            Device(self.device),
            Queue(self.queue),
            Surface(self.surface),
            SurfaceConfig(self.config),
        )
    }
}

//====================================================================

pub struct RenderPassDesc<'a> {
    pub use_depth: Option<&'a wgpu::TextureView>,
    pub clear_color: Option<[f64; 4]>,
}

impl RenderPassDesc<'_> {
    pub fn none() -> Self {
        Self {
            use_depth: None,
            clear_color: None,
        }
    }
}

impl Default for RenderPassDesc<'_> {
    fn default() -> Self {
        Self {
            use_depth: None,
            clear_color: Some([0.2, 0.2, 0.2, 1.]),
        }
    }
}

//--------------------------------------------------

pub use wgpu::SurfaceError;

pub struct RenderEncoder {
    surface_texture: wgpu::SurfaceTexture,
    surface_view: wgpu::TextureView,
    encoder: wgpu::CommandEncoder,
}

impl RenderEncoder {
    pub fn new(device: &wgpu::Device, surface: &wgpu::Surface) -> Result<Self, wgpu::SurfaceError> {
        let (surface_texture, surface_view) = match surface.get_current_texture() {
            Ok(texture) => {
                let view = texture
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                (texture, view)
            }
            Err(e) => return Err(e),
        };

        let encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Main Command Encoder"),
        });

        Ok(RenderEncoder {
            surface_texture,
            surface_view,
            encoder,
        })
    }

    pub fn finish(self, queue: &wgpu::Queue) {
        queue.submit(Some(self.encoder.finish()));
        self.surface_texture.present();
    }

    pub fn begin_render_pass(&mut self, desc: RenderPassDesc) -> wgpu::RenderPass {
        // Clear the current depth buffer and use it.
        let depth_stencil_attachment = match desc.use_depth {
            Some(view) => Some(wgpu::RenderPassDepthStencilAttachment {
                view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            None => None,
        };

        let load = match desc.clear_color {
            Some(color) => wgpu::LoadOp::Clear(wgpu::Color {
                r: color[0],
                g: color[1],
                b: color[2],
                a: color[3],
            }),
            None => wgpu::LoadOp::Load,
        };

        let render_pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Tools Basic Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.surface_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass
    }
}

//====================================================================
