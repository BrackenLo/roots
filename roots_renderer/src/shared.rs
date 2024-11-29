//====================================================================

use crate::{
    camera::{Camera, CameraUniform},
    texture::Texture,
    tools,
};

//====================================================================

pub trait Vertex: bytemuck::Pod {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}

//====================================================================

pub struct SharedRenderResources {
    texture_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group_layout: wgpu::BindGroupLayout,
}

impl SharedRenderResources {
    pub fn new(device: &wgpu::Device) -> Self {
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Shared Texture 3d Bind Group Layout"),
                entries: &[
                    tools::bgl_entry(tools::BgEntryType::Texture, 0, wgpu::ShaderStages::FRAGMENT),
                    tools::bgl_entry(tools::BgEntryType::Sampler, 1, wgpu::ShaderStages::FRAGMENT),
                ],
            });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        Self {
            texture_bind_group_layout,
            camera_bind_group_layout,
        }
    }
}

impl SharedRenderResources {
    #[inline]
    pub fn texture_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.texture_bind_group_layout
    }

    #[inline]
    pub fn camera_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.camera_bind_group_layout
    }
}

impl SharedRenderResources {
    pub fn create_texture_bind_group(
        &self,
        device: &wgpu::Device,
        texture: &Texture,
        label: Option<&str>,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label,
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
        })
    }

    #[inline]
    pub fn create_camera<C: CameraUniform>(&self, device: &wgpu::Device, data: &C) -> Camera {
        Camera::new(device, data, &self.camera_bind_group_layout)
    }
}

//====================================================================
