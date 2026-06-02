const PI: f32 = 3.1415926535897932384626433832795;

struct Face {
    forward: vec3<f32>,
    up: vec3<f32>,
    right: vec3<f32>,
}

@group(0)
@binding(0)
var src: texture_2d<f32>;

@group(0)
@binding(1)
var dst: texture_storage_2d_array<rgba16float, write>;

@compute
@workgroup_size(16, 16, 1)
fn compute_equirect_to_cubemap(
    // This is a "3D coordinate" where:
    // - x = pixel column
    // - y = pixel row
    // - z = face of the cube
    @builtin(global_invocation_id) gid: vec3<u32>,
) {
    // If texture size is not divisible by 32, we
    // need to make sure we don't try to write to
    // pixels that don't exist.
    if gid.x <= u32(textureDimensions(dst).x) {
        var FACES: array<Face, 6> = array(
            // FACES +X
            Face(
                vec3(1.0, 0.0, 0.0),  // forward
                vec3(0.0, 1.0, 0.0),  // up
                vec3(0.0, 0.0, -1.0), // right
            ),
            // FACES -X
            Face (
                vec3(-1.0, 0.0, 0.0),
                vec3(0.0, 1.0, 0.0),
                vec3(0.0, 0.0, 1.0),
            ),
            // FACES +Y
            Face (
                vec3(0.0, -1.0, 0.0),
                vec3(0.0, 0.0, 1.0),
                vec3(1.0, 0.0, 0.0),
            ),
            // FACES -Y
            Face (
                vec3(0.0, 1.0, 0.0),
                vec3(0.0, 0.0, -1.0),
                vec3(1.0, 0.0, 0.0),
            ),
            // FACES +Z
            Face (
                vec3(0.0, 0.0, 1.0),
                vec3(0.0, 1.0, 0.0),
                vec3(1.0, 0.0, 0.0),
            ),
            // FACES -Z
            Face (
                vec3(0.0, 0.0, -1.0),
                vec3(0.0, 1.0, 0.0),
                vec3(-1.0, 0.0, 0.0),
            ),
        );

        // Get texture coords relative to cubemap face
        let dst_dimensions = vec2<f32>(textureDimensions(dst));

        // Remap the cube UV from 0 -> 1 to -1 -> 1
        let cube_uv = vec2<f32>(gid.xy) / dst_dimensions * 2.0 - 1.0;

        // Get spherical coordinate from cube_uv
        let face = FACES[gid.z];
        let spherical = normalize(face.forward + face.right * cube_uv.x + face.up * cube_uv.y);

        /// We will use the latitude ("up-down") and longitude ("around") angles for equirectangular uv coordinates
        /// 
        /// - Lat:
        /// -- Lat = "How far above/below equator?" -> angle between vector and XZ plane
        /// -- Ranges from -90 to 90 (or -pi/2 to pi/2)
        /// -- It's the (inner) angle of the triangle on the XY plane, where the hypotenuse length = 1 (vector itself)
        /// -- With trig, we know that `sin(lat) = opposite/hypotenuse = y/1`
        /// -- Thus, `lat = asin(y)`
        let lat_angle = asin(spherical.y);

        // Map from pi/2 - pi/2 to 0 - 1
        let u = (lat_angle / PI) + 0.5;

        /// - For long:
        /// -- Long = "Which direction around the equator" -> projection/flattening of vector onto XZ plane
        /// -- Long ranges from -180 to 180 (or -pi to pi)
        /// -- It's the (inner) angle of the triangle on the XZ plane
        /// -- We don't know the length of hypotenuse outright; it's the unit vector, *flattened* onto XZ plane
        /// -- Instead, with trig, we see that `tan(long) = opposite/adjacent = z/x`
        /// -- Thus, `long = atan(z/x)`
        /// --- However, `atan` blows up at x=0
        /// --- Also, if z and x have same signs, the quadrant becomes unknown
        /// --- `atan2` patches these cases, and returns -pi to pi range instead of -pi/2 to pi/2
        let long_angle = atan2(spherical.z, spherical.x);

        // Map from -pi - pi to 0 - 1
        let v = (long_angle / (2 * PI)) + 0.5;

        // Finally - get the pixel coordinate, then the pixel there, and store it in the appropriate cube face!
        let uv = vec2(u, v);
        let pixel_uv = vec2<i32>(uv * vec2<f32>(textureDimensions(src)));

        // We use textureLoad() as textureSample() is not allowed in compute shaders
        var sample = textureLoad(src, pixel_uv, 0);

        textureStore(dst, gid.xy, gid.z, sample);
    }
}