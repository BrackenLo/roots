//====================================================================

use roots_renderer::{
    shared::{SharedRenderResources, Vertex},
    tools,
};

//====================================================================

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy, Debug)]
pub struct LineInstance {
    color: glam::Vec4,
    pos1: glam::Vec3,
    pos2: glam::Vec3,
    thickness: f32,
    pad: [u32; 1],
}

impl Default for LineInstance {
    #[inline]
    fn default() -> Self {
        Self {
            color: glam::Vec4::ONE,
            pos1: glam::Vec3::ONE,
            pos2: glam::Vec3::ZERO,
            thickness: 2.,
            pad: [0; 1],
        }
    }
}

impl Vertex for LineInstance {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
            1 => Float32x4, // Color
            2 => Float32x3, // pos 1
            3 => Float32x3, // pos 2
            4 => Float32, // Thickness
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<LineInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &VERTEX_ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy, Debug)]
struct LineVertex(glam::Vec3);

impl Vertex for LineVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![
            0 => Float32x3,
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<LineVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &VERTEX_ATTRIBUTES,
        }
    }
}

const LINE_VERTICES: [LineVertex; 8] = [
    LineVertex(glam::vec3(-0.5, 0., 0.)),
    LineVertex(glam::vec3(0.5, 0., 0.)),
    LineVertex(glam::vec3(0., -0.5, 0.)),
    LineVertex(glam::vec3(0., 0.5, 0.)),
    //
    LineVertex(glam::vec3(-0.5, 0., 0.)),
    LineVertex(glam::vec3(0.5, 0., 0.)),
    LineVertex(glam::vec3(0., -0.5, 0.)),
    LineVertex(glam::vec3(0., 0.5, 0.)),
];

const LINE_INDICES: [u16; 12] = [0, 4, 5, 0, 5, 1, 2, 6, 7, 2, 7, 3];

//====================================================================

pub struct LineRenderer {
    pipeline: wgpu::RenderPipeline,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,

    instance_buffer: wgpu::Buffer,
    instance_count: u32,

    to_prep: Vec<LineInstance>,
}

impl LineRenderer {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        shared: &SharedRenderResources,
    ) -> Self {
        log::debug!("Creating Line Renderer");

        let pipeline = tools::create_pipeline(
            device,
            config,
            "Line Pipeline",
            &[shared.camera_bind_group_layout()],
            &[LineVertex::desc(), LineInstance::desc()],
            include_str!("shaders/line.wgsl"),
            tools::RenderPipelineDescriptor {
                fragment_targets: Some(&[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::all(),
                })]),
                ..Default::default()
            }
            .with_depth_stencil(),
        );

        let vertex_buffer =
            tools::buffer(device, tools::BufferType::Vertex, "Line", &LINE_VERTICES);

        let index_buffer = tools::buffer(device, tools::BufferType::Index, "Line", &LINE_INDICES);
        let index_count = LINE_INDICES.len() as u32;

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Line Instance Buffer"),
            size: 0,
            usage: wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let instance_count = 0;

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            index_count,
            instance_buffer,
            instance_count,
            to_prep: Vec::new(),
        }
    }

    #[inline]
    pub fn prep_lines(&mut self, line: &[LineInstance]) {
        self.to_prep.extend_from_slice(line)
    }

    #[inline]
    pub fn finish_prep(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        tools::update_instance_buffer(
            device,
            queue,
            "Line",
            &mut self.instance_buffer,
            &mut self.instance_count,
            &self.to_prep,
        );

        self.to_prep.clear();
    }

    pub fn render(&self, pass: &mut wgpu::RenderPass, camera_bind_group: &wgpu::BindGroup) {
        if self.instance_count == 0 {
            return;
        }

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, camera_bind_group, &[]);

        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

        pass.draw_indexed(0..self.index_count, 0, 0..self.instance_count);
    }
}

//====================================================================
