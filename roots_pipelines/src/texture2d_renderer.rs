//====================================================================

use std::collections::{HashMap, HashSet};

use roots_renderer::{
    shared::{SharedRenderResources, Vertex},
    texture::{
        LoadedTexture, TextureId, TextureRectVertex, TEXTURE_RECT_INDEX_COUNT,
        TEXTURE_RECT_INDICES, TEXTURE_RECT_VERTICES,
    },
    tools::{self},
};

//====================================================================

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy, Debug)]
pub struct TextureInstance {
    pub color: glam::Vec4,
    pub size: glam::Vec2,
    pub pos: glam::Vec3,
    pub pad: [u32; 3],
}

impl Vertex for TextureInstance {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
            2 => Float32x4, // Color
            3 => Float32x2, // Size
            4 => Float32x3, // Pos
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &VERTEX_ATTRIBUTES,
        }
    }
}

pub struct TextureData<'a> {
    pub texture: &'a LoadedTexture,
    pub size: glam::Vec2,
    pub pos: glam::Vec3,
    pub color: glam::Vec4,
}

//====================================================================

#[derive(Debug)]
pub struct Texture2dRenderer {
    pipeline: wgpu::RenderPipeline,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,

    texture_storage: HashMap<TextureId, LoadedTexture>,
    instances: HashMap<TextureId, tools::InstanceBuffer<TextureInstance>>,
}

impl Texture2dRenderer {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        shared: &SharedRenderResources,
    ) -> Self {
        log::debug!("Creating Texture2d Renderer");

        let pipeline = tools::create_pipeline(
            device,
            config,
            "Texture Pipeline",
            &[
                shared.camera_bind_group_layout(),
                shared.texture_bind_group_layout(),
            ],
            &[TextureRectVertex::desc(), TextureInstance::desc()],
            include_str!("shaders/texture2d.wgsl"),
            tools::RenderPipelineDescriptor::default().with_depth_stencil(),
        );

        let vertex_buffer = tools::buffer(
            device,
            tools::BufferType::Vertex,
            "Texture",
            &TEXTURE_RECT_VERTICES,
        );

        let index_buffer = tools::buffer(
            device,
            tools::BufferType::Index,
            "Texture",
            &TEXTURE_RECT_INDICES,
        );
        let index_count = TEXTURE_RECT_INDEX_COUNT;

        let texture_storage = HashMap::default();
        let instances = HashMap::default();

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            index_count,
            texture_storage,
            instances,
        }
    }

    pub fn prep(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, data: &[TextureData]) {
        let mut previous = self.instances.keys().map(|id| *id).collect::<HashSet<_>>();

        let instances = data.into_iter().fold(HashMap::new(), |mut acc, data| {
            let instance = TextureInstance {
                color: data.color,
                size: data.size,
                pos: data.pos,
                pad: [0; 3],
            };

            acc.entry(data.texture.id())
                .or_insert_with(|| {
                    if !self.instances.contains_key(&data.texture.id()) {
                        self.texture_storage
                            .insert(data.texture.id(), data.texture.clone());
                    }

                    Vec::new()
                })
                .push(instance);

            acc
        });

        instances.into_iter().for_each(|(id, raw)| {
            previous.remove(&id);

            self.instances
                .entry(id)
                .and_modify(|instance| instance.update(device, queue, &raw))
                .or_insert_with(|| tools::InstanceBuffer::new(device, &raw));
        });

        previous.into_iter().for_each(|id| {
            log::trace!("Removing texture instance '{}'", id);
            self.instances.remove(&id);
            self.texture_storage.remove(&id);
        });
    }

    pub fn render(&self, pass: &mut wgpu::RenderPass, camera_bind_group: &wgpu::BindGroup) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, camera_bind_group, &[]);

        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        self.instances.iter().for_each(|(texture_id, instance)| {
            let texture = self.texture_storage.get(texture_id).unwrap();

            pass.set_bind_group(1, texture.bind_group(), &[]);
            pass.set_vertex_buffer(1, instance.slice(..));
            pass.draw_indexed(0..self.index_count, 0, 0..instance.count());
        });
    }
}

//====================================================================
