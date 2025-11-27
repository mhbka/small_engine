use cgmath::Vector3;
use crate::{core::world::{World, WorldEntityId}, graphics::gpu::{GpuContext, bind_group::GpuBindGroup, buffer::GpuBuffer}};

pub const MAX_POINT_LIGHTS: usize = 1000;

/// A collection of point lights.
pub struct PointLightCollection {
    lights: Vec<PointLight>,
    light_buffer: GpuBuffer,
    light_count_buffer: GpuBuffer
}

impl PointLightCollection {
    /// Create a new collection with a max capacity of `MAX_POINT_LIGHTS`.
    pub fn new(label: &str, lights: Vec<PointLight>, gpu: &GpuContext) -> Self {
        let light_buffer = GpuBuffer::create_storage_uninit(
            label, 
            gpu, 
            (size_of::<PointLightUniform>() * MAX_POINT_LIGHTS) as u64
        );
        let light_count_buffer = GpuBuffer::create_uniform(
            label, 
            gpu, 
            bytemuck::cast_slice(&[0 as u32])
        );
        Self {
            lights,
            light_buffer,
            light_count_buffer
        }
    }

    /// Create the bind group with this collection's buffer.
    pub fn create_bind_group(&self, label: &str, gpu: &GpuContext) -> GpuBindGroup {
        GpuBindGroup::create_default(
            label,
            gpu,
            &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None, 
                }
            ],
            &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.light_buffer.handle().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.light_count_buffer.handle().as_entire_binding(),
                },
            ],
        )
    }

    /// Add the given lights to the collection. 
    /// 
    /// Panics if exceeds buffer capacity.
    pub fn add(&mut self, mut lights: Vec<PointLight>) {
        if self.lights.len() + lights.len() > MAX_POINT_LIGHTS {
            panic!("Too many point lights in the collection");
        }
        self.lights.append(&mut lights)
    }

    /// Remove the point lights with the given entity IDs.
    pub fn remove(&mut self, lights: Vec<WorldEntityId>) {
        self.lights.retain(|l| !lights.contains(&l.entity));
    }

    /// Remove the lights with the given entity IDs from the collection.
    pub fn update_and_write_buffer(&mut self, world: &World, gpu: &GpuContext) {
        let uniform_data = self.lights
            .iter_mut()
            .map(|light| light.update_and_return_uniform(world))
            .collect::<Vec<_>>();
        gpu.queue().write_buffer(
            self.light_buffer.handle(),
            0,
            bytemuck::cast_slice(&uniform_data),
        );
        gpu.queue().write_buffer(
            self.light_count_buffer.handle(), 
            0, 
            bytemuck::cast_slice(&[self.lights.len() as u32])
        );
    }
}

/// A point light.
pub struct PointLight {
    entity: WorldEntityId,
    uniform: PointLightUniform,
}

impl PointLight {
    /// Create a new point light tied to the given entity.
    pub fn new(
        entity: WorldEntityId, 
        color: Vector3<f32>
    ) -> Self {
        let uniform = PointLightUniform::new(color.into());
        Self { 
            entity,
            uniform, 
        }
    }

    /// Update and return the light's uniform.
    pub fn update_and_return_uniform(&mut self, world: &World) -> PointLightUniform {
        let entity = world
            .entity(self.entity)
            .expect("Point light entity should exist");
        self.uniform.update(entity);
        self.uniform
    }
}

use crate::core::entity::WorldEntity;

/// Represents a colored point in space.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::NoUninit)]
pub struct PointLightUniform {
    pub position: [f32; 3],
    _padding: u32, // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here...
    pub color: [f32; 3],
    _padding2: u32, // ...And here
}

impl PointLightUniform {
    /// Create a light uniform.
    pub fn new(color: [f32; 3]) -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            _padding: 0,
            color,
            _padding2: 0,
        }
    }

    /// Update the uniform.
    pub fn update(&mut self, entity: &WorldEntity) {
        self.position = entity.transform().position.into();
    }
}