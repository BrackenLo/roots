//====================================================================

use crate::tools;

//====================================================================

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy, Debug)]
pub struct GlobalLightData {
    pub ambient_color: glam::Vec3,
    pub ambient_strength: f32,
}

impl Default for GlobalLightData {
    #[inline]
    fn default() -> Self {
        Self {
            ambient_color: glam::Vec3::ONE,
            ambient_strength: 0.05,
        }
    }
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy, Debug, Default)]
pub struct LightInstance {
    position: glam::Vec4,
    direction: glam::Vec4,
    diffuse: glam::Vec4,
    specular: glam::Vec4,
}

impl LightInstance {
    const ZERO: LightInstance = LightInstance {
        position: glam::Vec4::ZERO,
        direction: glam::Vec4::ZERO,
        diffuse: glam::Vec4::ZERO,
        specular: glam::Vec4::ZERO,
    };
}

//====================================================================

pub struct LightingManager {
    globals_uniform: wgpu::Buffer,
    light_instances: wgpu::Buffer,
    light_instance_count: u32,

    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl LightingManager {
    pub fn new(device: &wgpu::Device) -> Self {
        log::debug!("Creating lighting manager");

        let globals_uniform = tools::create_buffer(
            device,
            tools::BufferType::Uniform,
            "Lighting Globals",
            &[GlobalLightData::default()],
        );

        let light_instances = tools::create_buffer(
            device,
            tools::BufferType::Storage,
            "Light instances",
            &[LightInstance::ZERO],
        );

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Light uniform bind group layout"),
            entries: &[
                tools::bgl_entry(tools::BgEntryType::Uniform, 0, wgpu::ShaderStages::FRAGMENT),
                tools::bgl_entry(tools::BgEntryType::Storage, 1, wgpu::ShaderStages::FRAGMENT),
            ],
        });

        let bind_group = Self::bind_lighting_buffers(
            device,
            &bind_group_layout,
            &globals_uniform,
            &light_instances,
        );

        Self {
            globals_uniform,
            light_instances,
            light_instance_count: 0,
            bind_group,
            bind_group_layout,
        }
    }

    fn bind_lighting_buffers(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        globals_uniform: &wgpu::Buffer,
        light_instances: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Light uniform bind group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        globals_uniform.as_entire_buffer_binding(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(
                        light_instances.as_entire_buffer_binding(),
                    ),
                },
            ],
        })
    }

    #[inline]
    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    #[inline]
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

//--------------------------------------------------

impl LightingManager {
    pub fn update_lights(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        lights: &[LightInstance],
    ) {
        match lights.is_empty() {
            true => {
                self.light_instances = tools::create_buffer(
                    device,
                    tools::BufferType::Storage,
                    "Light instances",
                    &[LightInstance::ZERO],
                );
                self.light_instance_count = 0;
                self.bind_group = Self::bind_lighting_buffers(
                    device,
                    &self.bind_group_layout,
                    &self.globals_uniform,
                    &self.light_instances,
                );
            }

            false => {
                if lights.len() <= self.light_instance_count as usize {
                    // queue.write_buffer(&self.light_instances, 0, bytemuck::cast_slice(lights));
                    // return;

                    let buffer_size = std::mem::size_of::<LightInstance> as u64
                        * self.light_instance_count as u64;

                    let mut buffer_slice = queue
                        .write_buffer_with(
                            &self.light_instances,
                            0,
                            wgpu::BufferSize::new(buffer_size).unwrap(),
                        )
                        .unwrap();

                    let (data, empty) = buffer_slice.split_at_mut(lights.len());
                    data.copy_from_slice(bytemuck::cast_slice(lights));
                    empty.fill(0);

                    return;
                }

                self.light_instance_count = lights.len() as u32;
                self.light_instances = tools::create_buffer(
                    device,
                    tools::BufferType::Storage,
                    "Light instances",
                    &[LightInstance::ZERO],
                );
            }
        }
    }

    #[inline]
    pub fn update_globals(&self, queue: &wgpu::Queue, data: GlobalLightData) {
        queue
            .write_buffer_with(
                &self.globals_uniform,
                0,
                wgpu::BufferSize::new(std::mem::size_of::<GlobalLightData>() as u64).unwrap(),
            )
            .unwrap()
            .copy_from_slice(bytemuck::cast_slice(&[data]));
    }
}

//====================================================================

//====================================================================
