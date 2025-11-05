use std::{ops::Range, time::Duration};
use bytemuck::{Pod, Zeroable};
use cgmath::{Matrix4, Quaternion, Rad, Vector3, prelude::*};
use wgpu::{Buffer, BufferUsages, Device, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode, util::{BufferInitDescriptor, DeviceExt}, vertex_attr_array};

use crate::gpu::{GpuContext, buffer::GpuBuffer};

pub const NUM_INSTANCES_PER_ROW: u32 = 10;
pub const INSTANCE_DISPLACEMENT: Vector3<f32> = Vector3::new(NUM_INSTANCES_PER_ROW as f32 * 0.5, 0.0, NUM_INSTANCES_PER_ROW as f32 * 0.5);

pub const BOB_SPEED: f32 = 1.0;
pub const ROTATION_SPEED: f32 = 1.0;
pub const MAX_VERTICAL_OFFSET: f32 = 0.3;

/// Just generate some spaced instances.
pub fn generate_instances() -> Vec<Instance> {
    const SPACE_BETWEEN: f32 = 3.0;
let instances = (0..NUM_INSTANCES_PER_ROW).flat_map(|z| {
    (0..NUM_INSTANCES_PER_ROW).map(move |x| {
        let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
        let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

        let position = cgmath::Vector3 { x, y: 0.0, z };

        let rotation = if position.is_zero() {
            cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
        } else {
            cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
        };

        Instance {
            scale: Vector3::new(1.0, 1.0, 1.0), 
            position, 
            rotation,
        }
    })
}).collect::<Vec<_>>();
    
    instances
}

/// Contains data for instances.
pub struct Instances {
    instances: Vec<Instance>,
    range: Range<u32>,
    buffer: GpuBuffer
}

impl Instances {
    /// Initialize the instances.
    pub fn initialize(label: &str, gpu: &GpuContext, instances: Vec<Instance>) -> Self {
        // let instances = generate_instances();
        let instance_data = instances.iter().map(|i| i.to_raw()).collect::<Vec<_>>();
        let buffer = GpuBuffer::create_writeable_vertex(
            label, 
            gpu, 
            bytemuck::cast_slice(&instance_data)
        );
        let range = 0..instance_data.len() as u32;
        Self {
            instances,
            range,
            buffer
        }
    }

    /// Get the actual instances.
    pub fn actual(&mut self) -> &mut Vec<Instance> { &mut self.instances }

    /// Get the current range of active instances.
    pub fn range(&self) -> Range<u32> { self.range.clone() }

    /// Get the raw content to be written to the instance buffer.
    /// 
    // TODO: premature optimization but is it possible to remove the `to_vec` here?
    pub fn content(&self) -> Vec<u8> {
        let raw_instances = self.instances.iter().map(|i| i.to_raw()).collect::<Vec<_>>();
        bytemuck::cast_slice(&raw_instances).to_vec()
    }

    /// Get the buffer for the instances.
    pub fn buffer(&self) -> &GpuBuffer { &self.buffer }
}

/// An instance.
pub struct Instance {
    pub scale: Vector3<f32>,
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>
}

impl Instance {
    /// Get the uniform data for this instance.
    pub fn to_raw(&self) -> RawInstance {
        RawInstance {
            model: (Matrix4::from_translation(self.position) * Matrix4::from(self.rotation) * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)).into()
        }   
    }

    /// Update this instance's values through a callback.
    pub fn update<F>(&mut self, mut update: F)
    where 
        F: FnMut(&mut Instance) 
    {
        update(self);
    }
}

/// Contains the actual matrix for the instance.
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct RawInstance {
    model: [[f32; 4]; 4]
}

impl RawInstance {
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout { 
            array_stride: size_of::<RawInstance>() as u64,
            step_mode: VertexStepMode::Instance, 
            attributes:  &[
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We'll have to reassemble the mat4 in the shader.
                //
                // Note that we start at location 5 to reserve 2-4 for other vertex stuff.
                VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: size_of::<[f32; 4]>() as u64,
                    shader_location: 6,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: size_of::<[f32; 8]>() as u64,
                    shader_location: 7,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: size_of::<[f32; 12]>() as u64,
                    shader_location: 8,
                    format: VertexFormat::Float32x4,
                },
            ]
        }
    }
}