use crate::graphics::gpu::GpuContext;

/// Abstraction of the bind group + its layout.
#[derive(Clone, Debug)]
pub struct GpuBindGroup {
    group: wgpu::BindGroup,
    layout: wgpu::BindGroupLayout,
}

impl GpuBindGroup {
    /// Create a bind group.
    pub fn new(group: wgpu::BindGroup, layout: wgpu::BindGroupLayout) -> Self {
        Self { group, layout }
    }

    /// Create the bind group with mostly default configs.
    pub fn create_default(
        label: &str,
        gpu: &GpuContext,
        layout_entries: &[wgpu::BindGroupLayoutEntry],
        entries: &[wgpu::BindGroupEntry],
    ) -> Self {
        let device = gpu.device();

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: layout_entries,
            label: Some(&format!("{label}_layout")),
        });
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries,
            label: Some(label),
        });

        Self { group, layout }
    }

    /// Create the bind group with your own descriptor.
    pub fn create_with_desc(
        gpu: &GpuContext,
        layout_desc: &wgpu::BindGroupLayoutDescriptor,
        desc: &wgpu::BindGroupDescriptor,
    ) -> Self {
        let device = gpu.device();
        let layout = device.create_bind_group_layout(layout_desc);
        let group = device.create_bind_group(desc);
        Self { group, layout }
    }

    /// Get the actual bind group.
    pub fn handle(&self) -> &wgpu::BindGroup {
        &self.group
    }

    /// Get the group's layout.
    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }
}
