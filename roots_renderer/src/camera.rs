//====================================================================

use wgpu::util::DeviceExt;

//====================================================================

pub struct Camera {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl Camera {
    pub fn new<C: CameraUniform>(
        device: &wgpu::Device,
        data: &C,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        log::trace!("Creating new camera of type {}", std::any::type_name::<C>());

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera buffer"),
            contents: bytemuck::cast_slice(&[data.get_camera_uniform(&glam::Affine3A::IDENTITY)]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding()),
            }],
        });

        Self { buffer, bind_group }
    }

    #[inline]
    pub fn update_camera<C: CameraUniform>(
        &self,
        queue: &wgpu::Queue,
        data: &C,
        transform: &glam::Affine3A,
    ) {
        queue
            .write_buffer_with(
                &self.buffer,
                0,
                wgpu::BufferSize::new(std::mem::size_of::<CameraUniformRaw>() as u64).unwrap(),
            )
            .unwrap()
            .copy_from_slice(bytemuck::cast_slice(&[data.get_camera_uniform(transform)]));
    }

    #[inline]
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

//====================================================================

pub trait CameraUniform {
    fn get_projection_matrix(&self) -> glam::Mat4;
    fn get_view_matrix(&self, transform: &glam::Affine3A) -> glam::Mat4;

    #[inline]
    fn get_camera_uniform(&self, transform: &glam::Affine3A) -> CameraUniformRaw {
        CameraUniformRaw::new(
            self.get_projection_matrix() * self.get_view_matrix(transform),
            transform.translation.into(),
        )
    }
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
pub struct CameraUniformRaw {
    view_projection: glam::Mat4,
    camera_position: glam::Vec3,
    _padding: u32,
}

impl CameraUniformRaw {
    pub fn new(view_projection: glam::Mat4, camera_position: glam::Vec3) -> Self {
        Self {
            view_projection,
            camera_position,
            _padding: 0,
        }
    }
}

//--------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct OrthographicCamera {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub z_near: f32,
    pub z_far: f32,
}

impl Default for OrthographicCamera {
    fn default() -> Self {
        Self {
            left: 0.,
            right: 1920.,
            bottom: 0.,
            top: 1080.,
            z_near: 0.,
            z_far: 1000000.,
        }
    }
}

impl CameraUniform for OrthographicCamera {
    #[inline]
    fn get_projection_matrix(&self) -> glam::Mat4 {
        self.get_projection()
    }

    #[inline]
    fn get_view_matrix(&self, transform: &glam::Affine3A) -> glam::Mat4 {
        let (_, rotation, translation) = transform.to_scale_rotation_translation();

        glam::Mat4::from_rotation_translation(rotation, translation)
    }
}

impl OrthographicCamera {
    #[inline]
    fn get_projection(&self) -> glam::Mat4 {
        glam::Mat4::orthographic_lh(
            self.left,
            self.right,
            self.bottom,
            self.top,
            self.z_near,
            self.z_far,
        )
    }

    pub fn new_sized(width: f32, height: f32) -> Self {
        Self {
            left: 0.,
            right: width,
            bottom: 0.,
            top: height,
            ..Default::default()
        }
    }

    pub fn new_centered(half_width: f32, half_height: f32) -> Self {
        Self {
            left: -half_width,
            right: half_width,
            bottom: -half_height,
            top: half_height,
            ..Default::default()
        }
    }

    pub fn set_size(&mut self, width: f32, height: f32) {
        let half_width = width / 2.;
        let half_height = height / 2.;

        self.left = -half_width;
        self.right = half_width;
        self.top = half_height;
        self.bottom = -half_height;
    }
}

//--------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct PerspectiveCamera {
    pub up: glam::Vec3,
    pub aspect: f32,
    pub fovy: f32,
    pub z_near: f32,
    pub z_far: f32,
}

impl Default for PerspectiveCamera {
    fn default() -> Self {
        Self {
            up: glam::Vec3::Y,
            aspect: 1.7777777778,
            fovy: 45.,
            z_near: 0.1,
            z_far: 1000000.,
        }
    }
}

impl CameraUniform for PerspectiveCamera {
    #[inline]
    fn get_projection_matrix(&self) -> glam::Mat4 {
        self.get_projection()
    }

    fn get_view_matrix(&self, transform: &glam::Affine3A) -> glam::Mat4 {
        let forward = (transform.matrix3 * glam::Vec3::Z).normalize_or_zero();
        let translation = transform.translation.into();

        glam::Mat4::look_at_lh(translation, translation + forward, self.up)
    }
}

impl PerspectiveCamera {
    #[inline]
    fn get_projection(&self) -> glam::Mat4 {
        glam::Mat4::perspective_lh(self.fovy, self.aspect, self.z_near, self.z_far)
    }
}

//====================================================================
