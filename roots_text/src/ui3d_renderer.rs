//====================================================================

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use cosmic_text::{Metrics, Wrap};
use roots_renderer::{
    shared::{SharedRenderResources, Vertex},
    texture::Texture,
    tools,
};

use crate::{
    atlas::TextAtlas,
    shared::{TextBuffer, TextBufferDescriptor, TextResources, TextVertex},
};

//====================================================================

#[derive(Debug, Clone)]
pub struct Ui3d {
    pub menu_color: [f32; 4],
    pub selection_color: [f32; 4],

    pub options: Vec<String>,
    pub selected: u8,
    pub font_size: f32,
}

impl Default for Ui3d {
    fn default() -> Self {
        Self {
            menu_color: [0.5, 0.5, 0.5, 0.7],
            selection_color: [0.7, 0.7, 0.7, 0.8],
            options: Vec::new(),
            selected: 0,
            font_size: 30.,
        }
    }
}

#[derive(Debug)]
struct Ui3dData {
    ui_uniform_buffer: wgpu::Buffer,
    ui_uniform_bind_group: wgpu::BindGroup,

    ui_position_uniform_buffer: wgpu::Buffer,
    ui_position_uniform_bind_group: wgpu::BindGroup,
    size: [f32; 2],

    text_buffer: TextBuffer,
}

//====================================================================

pub struct Ui3dRenderer<ID> {
    ui_pipeline: wgpu::RenderPipeline,
    text_pipeline: wgpu::RenderPipeline,

    ui_uniform_bind_group_layout: wgpu::BindGroupLayout,
    ui_position_uniform_bind_group_layout: wgpu::BindGroupLayout,

    instances: HashMap<ID, Ui3dData>,
    previous: HashSet<ID>,
}

impl<ID> Ui3dRenderer<ID>
where
    ID: Hash + PartialEq + Eq + Clone,
{
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        shared: &mut SharedRenderResources,
        text_shared: &mut TextResources,
    ) -> Self {
        let ui_position_uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Ui Instance Buffer Bind Group Layout"),
                entries: &[tools::bgl_entry(
                    tools::BgEntryType::Uniform,
                    0,
                    wgpu::ShaderStages::VERTEX,
                )],
            });

        let ui_uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Ui Instance Buffer Bind Group Layout"),
                entries: &[tools::bgl_entry(
                    tools::BgEntryType::Uniform,
                    0,
                    wgpu::ShaderStages::VERTEX,
                )],
            });

        let ui_pipeline = tools::create_pipeline(
            device,
            config,
            "Ui Renderer",
            &[
                shared.camera_bind_group_layout(),
                &ui_uniform_bind_group_layout,
                &ui_position_uniform_bind_group_layout,
            ],
            &[],
            include_str!("shaders/ui3d.wgsl"),
            tools::RenderPipelineDescriptor {
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    cull_mode: Some(wgpu::Face::Back),
                    ..Default::default()
                },
                fragment_targets: Some(&[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::all(),
                })]),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: Texture::DEPTH_FORMAT,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                ..Default::default()
            },
        );

        let text_pipeline = tools::create_pipeline(
            device,
            config,
            "Ui Text Renderer",
            &[
                shared.camera_bind_group_layout(),
                text_shared.text_atlas.bind_group_layout(),
                &ui_position_uniform_bind_group_layout,
            ],
            &[TextVertex::desc()],
            include_str!("shaders/text.wgsl"),
            tools::RenderPipelineDescriptor {
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    cull_mode: Some(wgpu::Face::Back),
                    ..Default::default()
                },
                fragment_targets: Some(&[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::all(),
                })]),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: Texture::DEPTH_FORMAT,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                ..Default::default()
            },
        );

        Self {
            ui_pipeline,
            text_pipeline,
            ui_uniform_bind_group_layout,
            ui_position_uniform_bind_group_layout,
            instances: HashMap::default(),
            previous: HashSet::default(),
        }
    }

    pub fn prep_text(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        text_atlas: &mut TextAtlas,
        font_system: &mut cosmic_text::FontSystem,
        swash_cache: &mut cosmic_text::SwashCache,

        id: ID,
        ui_data: &Ui3d,
        transform: glam::Mat4,
    ) {
        //--------------------------------------------------

        self.previous.remove(&id);

        //--------------------------------------------------
        // Insert new text data

        if !self.instances.contains_key(&id) {
            log::trace!("Inserting new ui3d data");

            let ui_uniform_buffer = tools::create_buffer(
                device,
                tools::BufferType::Uniform,
                "Ui",
                &[UiUniformRaw {
                    size: glam::Vec2::ONE,
                    pad: [0.; 2],
                    menu_color: glam::Vec4::ONE,
                    selection_color: glam::Vec4::ONE,
                    selection_range_y: glam::Vec2::ZERO,
                    pad2: [0.; 2],
                }],
            );

            let ui_uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Ui Bind Group"),
                layout: &self.ui_uniform_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        ui_uniform_buffer.as_entire_buffer_binding(),
                    ),
                }],
            });

            let ui_position_uniform_buffer = tools::create_buffer(
                device,
                tools::BufferType::Uniform,
                "Ui Position",
                &[UiPositionUniformRaw {
                    transform: glam::Mat4::default(),
                }],
            );

            let ui_position_uniform_bind_group =
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Ui Position Bind Group"),
                    layout: &self.ui_position_uniform_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(
                            ui_position_uniform_buffer.as_entire_buffer_binding(),
                        ),
                    }],
                });

            let text = ui_data
                .options
                .iter()
                .cloned()
                .reduce(|a, b| format!("{}\n{}", a, b))
                .unwrap_or(String::new());

            let text_buffer = TextBuffer::new(
                device,
                font_system,
                &TextBufferDescriptor {
                    metrics: Metrics::new(10., 10.),
                    word_wrap: Wrap::None,
                    text: &text,
                    ..Default::default()
                },
            );

            self.instances.insert(
                id.clone(),
                Ui3dData {
                    ui_uniform_buffer,
                    ui_uniform_bind_group,
                    ui_position_uniform_buffer,
                    ui_position_uniform_bind_group,
                    size: [1., 1.],
                    text_buffer,
                },
            );
        }

        // Get Text Data
        let data = match self.instances.get_mut(&id) {
            Some(data) => data,
            None => return,
        };

        //--------------------------------------------------
        // Build Text

        if let Some(rebuild) = crate::shared::prep(
            device,
            queue,
            text_atlas,
            font_system,
            swash_cache,
            &mut data.text_buffer,
        ) {
            data.text_buffer.update_buffer(device, queue, &rebuild);
        }

        //--------------------------------------------------
        // Build Transform

        let position_raw = UiPositionUniformRaw { transform };

        queue
            .write_buffer_with(
                &data.ui_position_uniform_buffer,
                0,
                wgpu::BufferSize::new(std::mem::size_of::<UiPositionUniformRaw>() as u64).unwrap(),
            )
            .unwrap()
            .copy_from_slice(bytemuck::cast_slice(&[position_raw]));

        //--------------------------------------------------
        // Build UI Background

        let longest_line = ui_data
            .options
            .iter()
            .reduce(|a, b| match a.len() < b.len() {
                true => a,
                false => b,
            });

        let longest_line = match longest_line {
            Some(val) => val,
            None => return,
        };

        let selected = ui_data.selected.clamp(0, ui_data.options.len() as u8) as f32;

        let option_count = ui_data.options.len() as f32;
        let option_range = 1. / option_count;

        let ui_size = glam::vec2(
            ui_data.font_size * longest_line.len() as f32,
            ui_data.font_size * option_count,
        );

        data.size = ui_size.to_array();
        data.text_buffer.set_metrics(
            font_system,
            Metrics::new(ui_data.font_size, ui_data.font_size),
        );

        let ui_raw = UiUniformRaw {
            size: ui_size,
            menu_color: ui_data.menu_color.into(),
            selection_color: ui_data.selection_color.into(),
            selection_range_y: glam::vec2(option_range * selected, option_range * (selected + 1.)),

            pad: [0.; 2],
            pad2: [0.; 2],
        };

        queue
            .write_buffer_with(
                &data.ui_uniform_buffer,
                0,
                wgpu::BufferSize::new(std::mem::size_of::<UiUniformRaw>() as u64).unwrap(),
            )
            .unwrap()
            .copy_from_slice(bytemuck::cast_slice(&[ui_raw]));

        //--------------------------------------------------
    }

    #[inline]
    pub fn finish_prep(&mut self) {
        self.previous.drain().into_iter().for_each(|to_remove| {
            self.instances.remove(&to_remove);
        });

        self.previous = self.instances.keys().map(|id| id.clone()).collect();
    }

    pub fn render(
        &mut self,
        render_pass: &mut wgpu::RenderPass,
        text_atlas: &TextAtlas,
        camera_bind_group: &wgpu::BindGroup,
    ) {
        // Set camera (both pipelines)
        render_pass.set_bind_group(0, camera_bind_group, &[]);

        // Draw UI background
        render_pass.set_pipeline(&self.ui_pipeline);

        self.instances.values().into_iter().for_each(|instance| {
            render_pass.set_bind_group(1, &instance.ui_uniform_bind_group, &[]);
            render_pass.set_bind_group(2, &instance.ui_position_uniform_bind_group, &[]);
            render_pass.draw(0..4, 0..1);
        });

        // // Draw Text
        render_pass.set_pipeline(&self.text_pipeline);
        render_pass.set_bind_group(1, text_atlas.bind_group(), &[]);

        self.instances.values().into_iter().for_each(|instance| {
            render_pass.set_vertex_buffer(0, instance.text_buffer.vertex_buffer().slice(..));
            render_pass.set_bind_group(2, &instance.ui_position_uniform_bind_group, &[]);
            render_pass.draw(0..4, 0..instance.text_buffer.vertex_count());
        });
    }
}

//====================================================================

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy, Debug)]
struct UiPositionUniformRaw {
    transform: glam::Mat4,
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy, Debug)]
struct UiUniformRaw {
    pub size: glam::Vec2,
    pub pad: [f32; 2],

    pub menu_color: glam::Vec4,
    pub selection_color: glam::Vec4,
    pub selection_range_y: glam::Vec2,
    pub pad2: [f32; 2],
}

//====================================================================
