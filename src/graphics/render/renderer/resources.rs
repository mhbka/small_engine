use slotmap::SlotMap;
use crate::graphics::{gpu::{bind_group::GpuBindGroup, pipeline::GpuPipeline}, render::{assets::AssetStore, renderer::{BindGroupId, PipelineId, RenderError, RenderResult}}};

pub struct RendererResources {
    pipelines: SlotMap<PipelineId, GpuPipeline>,
    bind_groups: SlotMap<BindGroupId, GpuBindGroup>,
    assets: AssetStore,
}

impl RendererResources {
    /// Instantiate the resources.
    pub fn new(
        pipelines: SlotMap<PipelineId, GpuPipeline>,
        bind_groups: SlotMap<BindGroupId, GpuBindGroup>,
        assets: AssetStore
    ) -> Self {
        Self {
            pipelines,
            bind_groups,
            assets
        }
    }

    /// Add the pipelines to the renderer and get back their IDs for referencing.
    pub fn add_pipelines(&mut self, pipelines: Vec<GpuPipeline>) -> Vec<PipelineId> {
        pipelines
            .into_iter()
            .map(|p| self.pipelines.insert(p))
            .collect()
    }

    /// Add the global bind groups to the renderer and get back their IDs for referencing.
    pub fn add_bind_groups(&mut self, groups: Vec<GpuBindGroup>) -> Vec<BindGroupId> {
        groups
            .into_iter()
            .map(|g| self.bind_groups.insert(g))
            .collect()
    }
    
    /// Get the referenced pipeline.
    pub fn get_pipeline(&self, id: PipelineId, command_label: &str) -> RenderResult<&GpuPipeline> {
        self.pipelines
            .get(id)
            .ok_or(RenderError::PipelineNotFound { label: command_label.into() })
    }

    /// Get the referenced bind group.
    pub fn get_bind_group(&self, id: BindGroupId, command_label: &str) -> RenderResult<&GpuBindGroup> {
        self.bind_groups
            .get(id)
            .ok_or(RenderError::GlobalBindGroupNotFound { label: command_label.into() })
    }

    /// Get the assets store.
    pub fn get_assets_store(&self) -> &AssetStore {
        &self.assets
    }

    /// Get the assets store mutably.
    pub fn get_assets_store_mut(&mut self) -> &mut AssetStore {
        &mut self.assets
    }
}