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
    // pub translation: glam::Vec3,
    // pub rotation: glam::Quat,
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
            // translation: glam::Vec3::ZERO,
            // rotation: glam::Quat::IDENTITY,
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
    fn get_projection(&self) -> glam::Mat4 {
        let projection_matrix = glam::Mat4::orthographic_lh(
            self.left,
            self.right,
            self.bottom,
            self.top,
            self.z_near,
            self.z_far,
        );

        projection_matrix

        // let transform_matrix =
        //     glam::Mat4::from_rotation_translation(self.rotation, -self.translation);

        // projection_matrix * transform_matrix
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
    // pub translation: glam::Vec3,
    // pub rotation: glam::Quat,
}

impl Default for PerspectiveCamera {
    fn default() -> Self {
        Self {
            up: glam::Vec3::Y,
            aspect: 1.7777777778,
            fovy: 45.,
            z_near: 0.1,
            z_far: 1000000.,
            // translation: glam::Vec3::ZERO,
            // rotation: glam::Quat::IDENTITY,
        }
    }
}

impl CameraUniform for PerspectiveCamera {
    #[inline]
    fn get_projection_matrix(&self) -> glam::Mat4 {
        self.get_projection()
        // CameraUniformRaw::new(self.get_projection(), self.translation.into())
    }

    fn get_view_matrix(&self, transform: &glam::Affine3A) -> glam::Mat4 {
        // let forward = transform.forward();

        let forward = (transform.matrix3 * glam::Vec3::Z).normalize_or_zero();
        let translation = transform.translation.into();

        glam::Mat4::look_at_lh(translation, translation + forward, self.up)
    }
}

impl PerspectiveCamera {
    fn get_projection(&self) -> glam::Mat4 {
        // let forward = (self.rotation * glam::Vec3::Z).normalize();

        let projection_matrix =
            glam::Mat4::perspective_lh(self.fovy, self.aspect, self.z_near, self.z_far);

        // let view_matrix =
        //     glam::Mat4::look_at_lh(self.translation, self.translation + forward, self.up);

        projection_matrix
        // * view_matrix
    }

    // pub fn forward(&self) -> glam::Vec3 {
    //     let (x, _, z) = (self.rotation * glam::Vec3::Z).into();
    //     glam::Vec3::new(x, 0., z).normalize()
    // }

    // pub fn right(&self) -> glam::Vec3 {
    //     let (x, _, z) = (self.rotation * glam::Vec3::X).into();
    //     glam::Vec3::new(x, 0., z).normalize()
    // }

    // pub fn rotate_camera(&mut self, yaw: f32, pitch: f32) {
    //     let yaw_rotation = glam::Quat::from_rotation_y(yaw);
    //     let pitch_rotation = glam::Quat::from_rotation_x(pitch);

    //     self.rotation = yaw_rotation * self.rotation * pitch_rotation;
    // }
}

//====================================================================
