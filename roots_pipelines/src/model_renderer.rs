//====================================================================

use std::collections::{HashMap, HashSet};

use roots_common::FastHasher;
use roots_renderer::{
    lighting::LightingManager,
    model::{LoadedMesh, MeshId, ModelVertex},
    shared::{SharedRenderResources, Vertex},
    texture::{LoadedTexture, TextureId},
    tools::{self},
};

//====================================================================

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy, Debug)]
struct ModelInstance {
    pub transform: glam::Mat4,
    pub color: glam::Vec4,
    pub normal: glam::Mat3,
    pub scale: glam::Vec3,
}

impl Vertex for ModelInstance {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 9] = wgpu::vertex_attr_array![
            3 => Float32x4, // Transform
            4 => Float32x4,
            5 => Float32x4,
            6 => Float32x4,
            7 => Float32x4, // Color
            8 => Float32x3, // Normal
            9 => Float32x3,
            10 => Float32x3,
            11 => Float32x3, // Scale
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ModelInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &VERTEX_ATTRIBUTES,
        }
    }
}

pub struct ModelData<'a> {
    pub meshes: &'a [(LoadedMesh, LoadedTexture)],
    pub color: [f32; 4],
    pub scale: glam::Vec3,
}

pub struct MeshInstance<'a> {
    pub mesh: &'a LoadedMesh,
    pub texture: &'a LoadedTexture,
}

//====================================================================

#[derive(Debug)]
pub struct ModelRenderer {
    pipeline: wgpu::RenderPipeline,

    texture_storage: HashMap<u32, LoadedTexture, FastHasher>,
    mesh_storage: HashMap<u32, LoadedMesh, FastHasher>,
    instances: HashMap<MeshId, HashMap<TextureId, tools::InstanceBuffer<ModelInstance>>>,
}

impl ModelRenderer {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        shared: &SharedRenderResources,
        lighting: &LightingManager,
    ) -> Self {
        log::debug!("Creating Model Renderer");

        let pipeline = tools::create_pipeline(
            device,
            config,
            "Model Pipeline",
            &[
                shared.camera_bind_group_layout(),
                lighting.bind_group_layout(),
                shared.texture_bind_group_layout(),
            ],
            &[ModelVertex::desc(), ModelInstance::desc()],
            include_str!("shaders/model.wgsl"),
            tools::RenderPipelineDescriptor::default()
                .with_depth_stencil()
                .with_backface_culling(),
        );

        Self {
            pipeline,
            texture_storage: HashMap::default(),
            mesh_storage: HashMap::default(),
            instances: HashMap::default(),
        }
    }

    pub fn prep<'a>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        data: impl IntoIterator<Item = (ModelData<'a>, glam::Mat4)>,
    ) {
        let mut previous = self
            .instances
            .iter()
            .flat_map(|(mesh_id, textures)| {
                textures.keys().map(|texture_id| (*mesh_id, *texture_id))
            })
            .collect::<HashSet<_>>();

        let mut meshes_used = HashSet::new();
        let mut textures_used = HashSet::new();

        let instances = data
            .into_iter()
            .fold(HashMap::new(), |mut acc, (model, transform)| {
                model.meshes.iter().for_each(|(mesh, texture)| {
                    let mesh_entry = acc.entry(mesh.id()).or_insert_with(|| {
                        if !self.mesh_storage.contains_key(&mesh.id()) {
                            self.mesh_storage.insert(mesh.id(), mesh.clone());
                        }
                        meshes_used.insert(mesh.id());

                        HashMap::new()
                    });

                    let rotation = transform.to_scale_rotation_translation().1;
                    let normal_matrix = glam::Mat3::from_quat(rotation);

                    mesh_entry
                        .entry(texture.id())
                        .or_insert_with(|| {
                            if !self.texture_storage.contains_key(&texture.id()) {
                                self.texture_storage.insert(texture.id(), texture.clone());
                            }

                            textures_used.insert(texture.id());

                            Vec::new()
                        })
                        .push(ModelInstance {
                            transform,
                            color: model.color.into(),
                            normal: normal_matrix,
                            scale: model.scale,
                        });
                });

                acc
            });

        instances.into_iter().for_each(|(mesh_id, texture_data)| {
            texture_data.into_iter().for_each(|(texture_id, raw)| {
                previous.remove(&(mesh_id, texture_id));

                self.instances
                    .entry(mesh_id)
                    .or_insert(HashMap::default())
                    .entry(texture_id)
                    .and_modify(|instance| instance.update(device, queue, &raw))
                    .or_insert_with(|| tools::InstanceBuffer::new(device, &raw));
            });
        });

        previous.into_iter().for_each(|(mesh_id, texture_id)| {
            log::trace!("Removing model instance {} - {}", mesh_id, texture_id);
            self.instances
                .get_mut(&mesh_id)
                .unwrap()
                .remove(&texture_id);
        });

        self.texture_storage
            .retain(|texture_id, _| textures_used.contains(texture_id));

        self.mesh_storage
            .retain(|mesh_id, _| meshes_used.contains(mesh_id));
    }

    pub fn render(
        &mut self,
        pass: &mut wgpu::RenderPass,
        camera_bind_group: &wgpu::BindGroup,
        lighting_bind_group: &wgpu::BindGroup,
    ) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, camera_bind_group, &[]);
        pass.set_bind_group(1, lighting_bind_group, &[]);

        self.instances.iter().for_each(|(mesh_id, instance)| {
            let mesh = self.mesh_storage.get(mesh_id).unwrap();

            pass.set_vertex_buffer(0, mesh.vertex_buffer().slice(..));
            pass.set_index_buffer(mesh.index_buffer().slice(..), wgpu::IndexFormat::Uint32);

            instance.iter().for_each(|(texture_id, instance)| {
                let texture = self.texture_storage.get(texture_id).unwrap();

                pass.set_bind_group(1, texture.bind_group(), &[]);
                pass.set_vertex_buffer(1, instance.slice(..));
                pass.draw_indexed(0..mesh.index_count(), 0, 0..instance.count());
            });
        });
    }
}

//====================================================================
