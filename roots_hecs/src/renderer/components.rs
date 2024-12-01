//====================================================================

use std::ops::{Deref, DerefMut};

use roots_common::WasmWrapper;
use roots_pipelines::line_renderer::LineInstance;
use roots_renderer::{model::LoadedMesh, texture::LoadedTexture};

//====================================================================

pub struct Model {
    pub meshes: WasmWrapper<Vec<(LoadedMesh, LoadedTexture)>>,
    pub color: [f32; 4],
    pub scale: glam::Vec3,
}

impl Model {
    #[inline]
    pub fn new(meshes: impl IntoIterator<Item = (LoadedMesh, LoadedTexture)>) -> Self {
        Self {
            meshes: WasmWrapper::new(meshes.into_iter().collect()),
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
    pub fn with_scale(mut self, scale: glam::Vec3) -> Self {
        self.scale = scale;
        self
    }
}

//====================================================================

pub struct LineBundle {
    pub lines: Vec<LineInstance>,
}

pub struct Sprite {
    pub texture: LoadedTexture,
    pub size: glam::Vec2,
    pub pos: glam::Vec3,
    pub color: glam::Vec4,
}

//====================================================================

pub struct Camera(WasmWrapper<roots_renderer::camera::Camera>);

impl Deref for Camera {
    type Target = roots_renderer::camera::Camera;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Camera {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

//====================================================================
