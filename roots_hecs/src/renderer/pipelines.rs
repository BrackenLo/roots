//====================================================================

use hecs::{Entity, World};
use roots_common::spatial::GlobalTransform;
use roots_pipelines::{
    line_renderer::LineRenderer,
    model_renderer::{ModelData, ModelRenderer},
    texture2d_renderer::{Texture2dRenderer, TextureData},
};
use roots_renderer::{camera::PerspectiveCamera, RenderPass};

use crate::{renderer::components::Camera, RendererState};

use super::components::{LineBundle, Model, Sprite};

//====================================================================

pub trait Pipeline: 'static {
    fn new(state: &RendererState) -> Self
    where
        Self: Sized;

    fn prep(&mut self, state: &RendererState, world: &mut World);
    fn resize(&mut self, state: &RendererState) {
        let _ = state;
    }

    fn render(&mut self, render_pass: &mut RenderPass, state: &RendererState, world: &mut World);
}

#[inline]
fn get_perspective_camera(world: &mut World) -> Option<(Entity, (&Camera, &PerspectiveCamera))> {
    world
        .query_mut::<(&Camera, &PerspectiveCamera)>()
        .into_iter()
        .next()
}

//====================================================================

impl Pipeline for ModelRenderer {
    #[inline]
    fn new(state: &RendererState) -> Self {
        Self::new(&state.device, &state.config, &state.shared, &state.lighting)
    }

    fn prep(&mut self, state: &RendererState, world: &mut World) {
        let data = world
            .query_mut::<(&Model, &GlobalTransform)>()
            .into_iter()
            .map(|(_, (model, global))| {
                (
                    ModelData {
                        meshes: &model.meshes,
                        color: model.color,
                        scale: model.scale,
                    },
                    global.to_matrix(),
                )
            })
            .collect::<Vec<_>>();

        self.prep(&state.device, &state.queue, data);
    }

    fn render(&mut self, render_pass: &mut RenderPass, state: &RendererState, world: &mut World) {
        if !self.has_instances_to_render() {
            return;
        }

        let camera = match get_perspective_camera(world) {
            Some(data) => data.1 .0,
            None => {
                log::warn!("Unable to render models - no camera available");
                return;
            }
        };

        self.render(
            render_pass,
            camera.bind_group(),
            state.lighting.bind_group(),
        );
    }
}

//====================================================================

impl Pipeline for Texture2dRenderer {
    #[inline]
    fn new(state: &RendererState) -> Self {
        Self::new(&state.device, &state.config, &state.shared)
    }

    fn prep(&mut self, state: &RendererState, world: &mut World) {
        let data = world
            .query_mut::<&Sprite>()
            .into_iter()
            .map(|(_, sprite)| TextureData {
                texture: &sprite.texture,
                size: sprite.size,
                pos: sprite.pos,
                color: sprite.color,
            })
            .collect::<Vec<_>>();

        self.prep(&state.device, &state.queue, data);
    }

    fn render(&mut self, render_pass: &mut RenderPass, _state: &RendererState, world: &mut World) {
        let camera = match get_perspective_camera(world) {
            Some(data) => data.1 .0,
            None => {
                log::warn!("Unable to render models - no camera available");
                return;
            }
        };

        Self::render(self, render_pass, camera.bind_group());
    }
}

//====================================================================

impl Pipeline for LineRenderer {
    #[inline]
    fn new(state: &RendererState) -> Self {
        Self::new(&state.device, &state.config, &state.shared)
    }

    fn prep(&mut self, state: &RendererState, world: &mut World) {
        let data =
            world
                .query_mut::<&LineBundle>()
                .into_iter()
                .fold(Vec::new(), |mut acc, (_, line)| {
                    acc.extend_from_slice(&line.lines);
                    acc
                });

        self.prep(&state.device, &state.queue, &data);
    }

    fn render(&mut self, render_pass: &mut RenderPass, _state: &RendererState, world: &mut World) {
        let camera = match get_perspective_camera(world) {
            Some(data) => data.1 .0,
            None => {
                log::warn!("Unable to render models - no camera available");
                return;
            }
        };

        Self::render(self, render_pass, camera.bind_group());
    }
}

//====================================================================
