use crate::{graphics::{
    gpu::{GpuContext, bind_group::GpuBindGroup, buffer::GpuBuffer, texture::GpuTexture},
    render::{
        assets::AssetStore,
        renderable::model::{self, Material, Model, ModelVertex}, renderer::Renderer,
    }, textures::standard::StandardTexture,
}, resources::utils::{calculate_tangents_and_bitangents::calculate_tangent_and_bitangents, load_data::{load_binary, load_string}}};
use std::io::{BufReader, Cursor};

/// Load a model from the given file into the renderer's asset store.
pub async fn load_model(
    file_name: &str,
    gpu: &GpuContext,
    renderer: &mut Renderer<'_>
) -> anyhow::Result<Model> {
    let obj_text = load_string(file_name).await?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);

    let (models, obj_materials) = tobj::load_obj_buf_async(
        &mut obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| async move {
            let mat_text = load_string(&p).await.unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    )
    .await?;

    let mut materials = Vec::new();
    for m in obj_materials? {
        let diffuse_texture = load_texture(&m.diffuse_texture, gpu).await?;
        let normal_texture = load_texture(&m.normal_texture, gpu).await?;
        let layout_entries =
            StandardTexture::bind_group_entries(&diffuse_texture, &normal_texture);
        let bind_group =
            GpuBindGroup::create_default(file_name, gpu, &layout_entries.0, &layout_entries.1);
        let bind_group_id = renderer.resources().add_bind_groups(vec![bind_group])[0];
        materials.push(Material {
            name: m.name,
            diffuse_texture,
            normal_texture,
            bind_group: bind_group_id
        })
    }
    let material_ids = renderer
        .resources()
        .get_assets_store_mut()
        .add_materials(materials);
    let meshes = models
        .into_iter()
        .map(|mut m| {
            let mut vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| {
                    let normal = if m.mesh.normals.is_empty() {
                        [0.0, 0.0, 0.0]
                    } else {
                        [
                            m.mesh.normals[i * 3],
                            m.mesh.normals[i * 3 + 1],
                            m.mesh.normals[i * 3 + 2],
                        ]
                    };
                    model::ModelVertex {
                            position: [
                                m.mesh.positions[i * 3],
                                m.mesh.positions[i * 3 + 1],
                                m.mesh.positions[i * 3 + 2],
                            ],
                            tex_coords: [
                                m.mesh.texcoords[i * 2],
                                1.0 - m.mesh.texcoords[i * 2 + 1],
                            ],
                            normal,
                            tangent: [0.0; 3],
                            bitangent: [0.0; 3]
                        }
                })
                .collect::<Vec<_>>();

            calculate_tangent_and_bitangents(&mut vertices, &mut m);

            let vertex_buffer = GpuBuffer::create_vertex(
                &format!("{:?}_vertex_buffer", file_name),
                gpu,
                bytemuck::cast_slice(&vertices),
            );
            let index_buffer = GpuBuffer::create_index(
                &format!("{:?}_index_buffer", file_name),
                gpu,
                bytemuck::cast_slice(&m.mesh.indices),
            );

            let material_index = m.mesh.material_id.unwrap_or(0);
            let material_id = material_ids[material_index];

            model::Mesh {
                name: file_name.to_string(),
                vertex_buffer,
                index_buffer,
                num_elements: m.mesh.indices.len() as u32,
                material: material_id,
            }
        })
        .collect::<Vec<_>>();

    let mesh_ids = renderer
        .resources()
        .get_assets_store_mut()
        .add_meshes(meshes);

    Ok(model::Model {
        meshes: mesh_ids,
        materials: material_ids,
    })
}

/// Load a texture from an image.
async fn load_texture(file_name: &str, gpu: &GpuContext) -> anyhow::Result<StandardTexture> {
    let data = load_binary(file_name).await?;
    let img = image::load_from_memory(&data)?;
    StandardTexture::from_image(gpu, &img, Some(file_name))
}