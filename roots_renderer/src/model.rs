//====================================================================

use std::sync::{atomic::AtomicU32, Arc};

use crate::{shared::Vertex, texture::LoadedTexture, tools};

//====================================================================

pub type MeshId = u32;

static CURRENT_MESH_ID: AtomicU32 = AtomicU32::new(0);

#[derive(Clone, Debug)]
pub struct LoadedMesh {
    id: MeshId,
    mesh: Arc<Mesh>,
}

impl LoadedMesh {
    #[inline]
    pub fn load_mesh(mesh: Mesh) -> Self {
        let id = CURRENT_MESH_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Self {
            id,
            mesh: Arc::new(mesh),
        }
    }

    #[inline]
    pub fn load_from_data(
        device: &wgpu::Device,
        vertices: &[ModelVertex],
        indices: &[u32],
    ) -> Self {
        Self::load_mesh(Mesh::load_mesh(device, vertices, indices))
    }

    #[inline]
    pub fn id(&self) -> MeshId {
        self.id
    }

    #[inline]
    pub fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.mesh.vertex_buffer
    }

    #[inline]
    pub fn index_buffer(&self) -> &wgpu::Buffer {
        &self.mesh.index_buffer
    }

    #[inline]
    pub fn index_count(&self) -> u32 {
        self.mesh.index_count
    }
}

//--------------------------------------------------

#[derive(Debug)]
pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

impl Mesh {
    pub fn load_mesh(device: &wgpu::Device, vertices: &[ModelVertex], indices: &[u32]) -> Self {
        let vertex_buffer = tools::buffer(device, tools::BufferType::Vertex, "Mesh", vertices);
        let index_buffer = tools::buffer(device, tools::BufferType::Index, "Mesh", indices);
        let index_count = indices.len() as u32;

        Self {
            vertex_buffer,
            index_buffer,
            index_count,
        }
    }
}

//--------------------------------------------------

#[derive(Clone)]
pub struct Model {
    pub meshes: Vec<(LoadedMesh, LoadedTexture)>,
    pub color: [f32; 4],
    pub scale: glam::Vec3,
}

impl Model {
    #[inline]
    pub fn from_mesh(mesh: LoadedMesh, texture: LoadedTexture) -> Self {
        Self {
            meshes: vec![(mesh, texture)],
            color: [1., 1., 1., 1.],
            scale: glam::Vec3::ONE,
        }
    }

    #[inline]
    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }

    #[inline]
    pub fn with_scale(mut self, scale: impl Into<glam::Vec3>) -> Self {
        self.scale = scale.into();
        self
    }
}

//====================================================================

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct ModelVertex {
    pub pos: glam::Vec3,
    pub uv: glam::Vec2,
    pub normal: glam::Vec3,
}

impl Vertex for ModelVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
            0 => Float32x3,
            1 => Float32x2,
            2 => Float32x3
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &VERTEX_ATTRIBUTES,
        }
    }
}

pub const CUBE_VERTICES: [ModelVertex; 24] = [
    // Back (-z)
    // Top Left - 0
    ModelVertex {
        pos: glam::vec3(-0.5, 0.5, -0.5),
        uv: glam::vec2(0., 0.),
        normal: glam::Vec3::NEG_Z,
    },
    // Top Right - 1
    ModelVertex {
        pos: glam::vec3(0.5, 0.5, -0.5),
        uv: glam::vec2(1., 0.),
        normal: glam::Vec3::NEG_Z,
    },
    // Bottom Left - 2
    ModelVertex {
        pos: glam::vec3(-0.5, -0.5, -0.5),
        uv: glam::vec2(0., 1.),
        normal: glam::Vec3::NEG_Z,
    },
    // Bottom Right - 3
    ModelVertex {
        pos: glam::vec3(0.5, -0.5, -0.5),
        uv: glam::vec2(1., 1.),
        normal: glam::Vec3::NEG_Z,
    },
    //
    // Right (+x)
    // Top Left - 4
    ModelVertex {
        pos: glam::vec3(0.5, 0.5, -0.5),
        uv: glam::vec2(0., 0.),
        normal: glam::Vec3::X,
    },
    // Top Right - 5
    ModelVertex {
        pos: glam::vec3(0.5, 0.5, 0.5),
        uv: glam::vec2(1., 0.),
        normal: glam::Vec3::X,
    },
    // Bottom Left - 6
    ModelVertex {
        pos: glam::vec3(0.5, -0.5, -0.5),
        uv: glam::vec2(0., 1.),
        normal: glam::Vec3::X,
    },
    // Bottom Right - 7
    ModelVertex {
        pos: glam::vec3(0.5, -0.5, 0.5),
        uv: glam::vec2(1., 1.),
        normal: glam::Vec3::X,
    },
    //
    // Front (+z)
    // Top Left - 8
    ModelVertex {
        pos: glam::vec3(0.5, 0.5, 0.5),
        uv: glam::vec2(0., 0.),
        normal: glam::Vec3::Z,
    },
    // Top Right - 9
    ModelVertex {
        pos: glam::vec3(-0.5, 0.5, 0.5),
        uv: glam::vec2(1., 0.),
        normal: glam::Vec3::Z,
    },
    // Bottom Left - 10
    ModelVertex {
        pos: glam::vec3(0.5, -0.5, 0.5),
        uv: glam::vec2(0., 1.),
        normal: glam::Vec3::Z,
    },
    // Bottom Right - 11
    ModelVertex {
        pos: glam::vec3(-0.5, -0.5, 0.5),
        uv: glam::vec2(1., 1.),
        normal: glam::Vec3::Z,
    },
    //
    // Left (-x)
    // Top Left - 12
    ModelVertex {
        pos: glam::vec3(-0.5, 0.5, 0.5),
        uv: glam::vec2(0., 0.),
        normal: glam::Vec3::NEG_X,
    },
    // Top Right - 13
    ModelVertex {
        pos: glam::vec3(-0.5, 0.5, -0.5),
        uv: glam::vec2(1., 0.),
        normal: glam::Vec3::NEG_X,
    },
    // Bottom Left - 14
    ModelVertex {
        pos: glam::vec3(-0.5, -0.5, 0.5),
        uv: glam::vec2(0., 1.),
        normal: glam::Vec3::NEG_X,
    },
    // Bottom Right - 15
    ModelVertex {
        pos: glam::vec3(-0.5, -0.5, -0.5),
        uv: glam::vec2(1., 1.),
        normal: glam::Vec3::NEG_X,
    },
    //
    // Top
    // Top Left - 16
    ModelVertex {
        pos: glam::vec3(0.5, 0.5, -0.5),
        uv: glam::vec2(0., 0.),
        normal: glam::Vec3::Y,
    },
    // Top Right - 17
    ModelVertex {
        pos: glam::vec3(-0.5, 0.5, -0.5),
        uv: glam::vec2(1., 0.),
        normal: glam::Vec3::Y,
    },
    // Bottom Left - 18
    ModelVertex {
        pos: glam::vec3(0.5, 0.5, 0.5),
        uv: glam::vec2(0., 1.),
        normal: glam::Vec3::Y,
    },
    // Bottom Right - 19
    ModelVertex {
        pos: glam::vec3(-0.5, 0.5, 0.5),
        uv: glam::vec2(1., 1.),
        normal: glam::Vec3::Y,
    },
    //
    // Bottom
    // Top Left - 20
    ModelVertex {
        pos: glam::vec3(0.5, -0.5, -0.5),
        uv: glam::vec2(0., 0.),
        normal: glam::Vec3::NEG_Y,
    },
    // Top Right - 21
    ModelVertex {
        pos: glam::vec3(-0.5, -0.5, -0.5),
        uv: glam::vec2(1., 0.),
        normal: glam::Vec3::NEG_Y,
    },
    // Bottom Left - 22
    ModelVertex {
        pos: glam::vec3(0.5, -0.5, 0.5),
        uv: glam::vec2(0., 1.),
        normal: glam::Vec3::NEG_Y,
    },
    // Bottom Right - 23
    ModelVertex {
        pos: glam::vec3(-0.5, -0.5, 0.5),
        uv: glam::vec2(1., 1.),
        normal: glam::Vec3::NEG_Y,
    },
];

pub const CUBE_INDICES: [u32; 36] = [
    0, 2, 3, 0, 3, 1, // Back
    4, 6, 7, 4, 7, 5, // Right
    8, 10, 11, 8, 11, 9, // Front
    12, 14, 15, 12, 15, 13, // Left
    16, 18, 19, 16, 19, 17, // Top
    20, 22, 23, 20, 23, 21, // Bottom
];

pub const CUBE_INDEX_COUNT: u32 = CUBE_INDICES.len() as u32;

//====================================================================
