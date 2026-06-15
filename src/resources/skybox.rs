use crate::{graphics::{gpu::{GpuContext, bind_group::GpuBindGroup}, render::{assets::SkyboxId, renderable::skybox::Skybox, renderer::Renderer}, textures::cube::CubeMapTexture}, resources::{hdr::HdrLoader, utils::load_data::load_binary}};

/// Loads a skybox, which is a HDR image converted into a cubemap texture,
/// into the renderer's asset store.
/// 
/// Returns its ID used to reference it in the asset store.
pub async fn load_skybox(
    file_name: &str,
    gpu: &GpuContext,
    renderer: &mut Renderer<'_>
) -> anyhow::Result<SkyboxId> {
    let hdr_loader = HdrLoader::new(gpu);
    let image_bytes = load_binary(file_name).await?;

    let cubemap_texture = hdr_loader.from_equirect_bytes(&gpu, &image_bytes, 1080, &format!("{file_name}_cubemap"))?;

    let bind_group_entries = CubeMapTexture::bind_group_entries(&cubemap_texture);
    let bind_group = GpuBindGroup::create_default(
        &format!("{file_name}_bind_group"), 
        gpu, 
        &bind_group_entries.0, 
        &bind_group_entries.1
    );
    let bind_group_id = renderer.resources().add_bind_groups(vec![bind_group])[0];

    let skybox = Skybox::new(format!("{file_name}_skybox"), cubemap_texture, bind_group_id);

    let skybox_id = renderer
        .resources()
        .get_assets_store_mut()
        .add_skyboxes(vec![skybox])[0];

    Ok(skybox_id)
}