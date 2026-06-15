use crate::graphics::render::renderable::model::ModelVertex;

pub fn calculate_tangent_and_bitangents(vertices: &mut Vec<ModelVertex>, model: &mut tobj::Model) {
    let indices = &model.mesh.indices;
    let mut triangles_included = vec![0; vertices.len()];

    for c in indices.chunks(3) {
        let v0 = vertices[c[0] as usize];
        let v1 = vertices[c[1] as usize];
        let v2 = vertices[c[2] as usize];

        let pos0: cgmath::Vector3<_> = v0.position.into();
        let pos1: cgmath::Vector3<_> = v1.position.into();
        let pos2: cgmath::Vector3<_> = v2.position.into();

        let uv0: cgmath::Vector2<_> = v0.tex_coords.into();
        let uv1: cgmath::Vector2<_> = v1.tex_coords.into();
        let uv2: cgmath::Vector2<_> = v2.tex_coords.into();

        let delta_pos1 = pos1 - pos0;
        let delta_pos2 = pos2 - pos0;
        
        let delta_uv1 = uv1 - uv0;
        let delta_uv2 = uv2 - uv0;

        let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
        let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;

        // use negative r to enable right-handed normal (?)
        let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * -r;

        vertices[c[0] as usize].tangent =
                (tangent + cgmath::Vector3::from(vertices[c[0] as usize].tangent)).into();
            vertices[c[1] as usize].tangent =
                (tangent + cgmath::Vector3::from(vertices[c[1] as usize].tangent)).into();
            vertices[c[2] as usize].tangent =
                (tangent + cgmath::Vector3::from(vertices[c[2] as usize].tangent)).into();
            vertices[c[0] as usize].bitangent =
                (bitangent + cgmath::Vector3::from(vertices[c[0] as usize].bitangent)).into();
            vertices[c[1] as usize].bitangent =
                (bitangent + cgmath::Vector3::from(vertices[c[1] as usize].bitangent)).into();
            vertices[c[2] as usize].bitangent =
                (bitangent + cgmath::Vector3::from(vertices[c[2] as usize].bitangent)).into();

        // Used to average the tangents/bitangents
        triangles_included[c[0] as usize] += 1;
        triangles_included[c[1] as usize] += 1;
        triangles_included[c[2] as usize] += 1;
    }

     for (i, n) in triangles_included.into_iter().enumerate() {
        let denom = 1.0 / n as f32;
        let v = &mut vertices[i];
        v.tangent = (cgmath::Vector3::from(v.tangent) * denom).into();
        v.bitangent = (cgmath::Vector3::from(v.bitangent) * denom).into();
    }
}