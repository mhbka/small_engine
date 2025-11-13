use crate::graphics::{
    gpu::{GpuContext, bind_group::GpuBindGroup, buffer::GpuBuffer, texture::GpuTexture},
    render::{
        assets::AssetStore,
        renderable::model::{self, Material, Model},
    },
};
use std::io::{BufReader, Cursor};

#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let mut origin = location.origin().unwrap();
    if !origin.ends_with("learn-wgpu") {
        origin = format!("{}/learn-wgpu", origin);
    }
    let base = reqwest::Url::parse(&format!("{}/", origin,)).unwrap();
    base.join(file_name).unwrap()
}

pub async fn load_string(file_name: &str) -> anyhow::Result<String> {
    #[cfg(target_arch = "wasm32")]
    let txt = {
        let url = format_url(file_name);
        reqwest::get(url).await?.text().await?
    };
    #[cfg(not(target_arch = "wasm32"))]
    let txt = {
        let path = std::path::Path::new(env!("OUT_DIR"))
            .join("res")
            .join(file_name);
        std::fs::read_to_string(path)?
    };

    Ok(txt)
}

pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    #[cfg(target_arch = "wasm32")]
    let data = {
        let url = format_url(file_name);
        reqwest::get(url).await?.bytes().await?.to_vec()
    };
    #[cfg(not(target_arch = "wasm32"))]
    let data = {
        let path = std::path::Path::new(env!("OUT_DIR"))
            .join("res")
            .join(file_name);
        std::fs::read(path)?
    };

    Ok(data)
}

pub async fn load_texture(file_name: &str, gpu: &GpuContext) -> anyhow::Result<GpuTexture> {
    let data = load_binary(file_name).await?;
    GpuTexture::from_bytes(gpu, &data, file_name)
}

/// Loads a model from the given file into the asset store.
pub async fn load_model(
    file_name: &str,
    gpu: &GpuContext,
    assets: &mut AssetStore,
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
        let layout_entries =
            GpuTexture::create_diffuse_texture_bind_group_entries(&diffuse_texture);
        let bind_group =
            GpuBindGroup::create_default(file_name, gpu, &layout_entries.0, &layout_entries.1);
        materials.push(Material {
            name: m.name,
            diffuse_texture,
            bind_group,
        })
    }

    let material_ids = assets.add_materials(materials);

    let meshes = models
        .into_iter()
        .map(|m| {
            let vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| {
                    if m.mesh.normals.is_empty() {
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
                            normal: [0.0, 0.0, 0.0],
                        }
                    } else {
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
                            normal: [
                                m.mesh.normals[i * 3],
                                m.mesh.normals[i * 3 + 1],
                                m.mesh.normals[i * 3 + 2],
                            ],
                        }
                    }
                })
                .collect::<Vec<_>>();

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

    let mesh_ids = assets.add_meshes(meshes);

    Ok(model::Model {
        meshes: mesh_ids,
        materials: material_ids,
    })
}
