
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) instance_id    : u32,
    @location(1) position_raw   : vec4<f32>,
};

struct FragmentOutput {
    @location(0) color0: vec4<f32>
};

struct RendererGPUData {
    data0           : vec4<f32>   // aspect_ratio, unused, unused, unused
};

struct RendererItem {
    offset_scale            : vec4<f32>,    // (x, y), (scale_x, scale_y)
    texture_transform       : vec4<f32>,    // (x, y), (scale_x, scale_y) / color (r, g, b, a)
    mask                    : vec4<f32>,    // (x, y), (scale_x, scale_y)
    border_radius           : vec4<f32>,    // (top_left, top_right, bottom_right, bottom_left)
    border_radius_mask      : vec4<f32>,    // (top_left, top_right, bottom_right, bottom_left)
    data0                   : vec4<f32>,    // (depth, rotation, unused, unused)
    meta0                   : vec4<f32>,    // (type, tex_layer, unused, unused)
};

@group(0) @binding(0) var<uniform> rendererGPUData : RendererGPUData;
@group(1) @binding(0) var<storage, read> rendererItems : array<RendererItem>;

@vertex
fn vs_main(
    @builtin(vertex_index) index: u32,
    @builtin(instance_index) instance_id: u32
) -> VertexOutput {
    var output: VertexOutput;

    let a = (index >> 2u) & 1u;
    let b = (index >> 1u) & 1u;
    let c = index & 1u;

    let tx = f32(b | (a & c));
    let ty = f32(a | (~b & c));

    // let instance_id = index / 6u;
    let item = rendererItems[instance_id];

    var position = vec2<f32>(tx, ty); 

    // scale
    let scale = item.offset_scale.zw;
    position = position * scale;

    // offset
    position = position + item.offset_scale.xy;

    let depth = item.data0.x;
    position = position * 2.0 - 1.0;

    output.position = vec4<f32>(position, depth, 1.0);
    output.position_raw = vec4<f32>(position, depth, 0.0);
    output.instance_id = instance_id;


    return output;
}

fn point_in_elipse(point: vec2<f32>, center: vec2<f32>, radius: vec2<f32>) -> f32 {
    let dx = point.x - center.x;
    let dy = point.y - center.y;
    let a = radius.x;
    let b = radius.y;
    return min(max(dx * dx / (a * a) + dy * dy / (b * b) - 1.0, 0.0) * 100.0, 1.0);
}

fn calculate_border_radius_mask(
    radius: f32,
    direction: vec2<f32>,
    start: vec2<f32>,
    size: vec2<f32>,
    position: vec2<f32>,
) -> f32 {
    let rad = radius;
    var rx = rad * min(size.x, size.y) * 0.5;
    var ry = rad * min(size.x, size.y) * 0.5;
    let aspect = rendererGPUData.data0.x;    
    let offset_mask = -direction;
    if (aspect > 1.0) {
        rx /= aspect;
    } else {
        ry *= aspect;
    }
    let center = start + vec2<f32>(rx, ry) * offset_mask;
    let current_dir = normalize(position - center);
    let dir = normalize(direction);
    let cos_angle = dot(current_dir, dir);
    let cos_factor = min(max(cos_angle - 0.7071, 0.0) * 100.0, 1.0);
    let ellipse_factor = 1.0 - point_in_elipse(position, center, vec2<f32>(rx, ry));
    return clamp(ellipse_factor * cos_factor + 1.0 - cos_factor, 0.0, 1.0);
}

@fragment
fn fs_main(
    in: VertexOutput
) -> FragmentOutput {
    var output: FragmentOutput;
    // output.color0 = vec4<f32>(1.0, 0.0, 0.0, 1.0);

    let item = rendererItems[ in.instance_id ];

    // if position is outside of mask, discard
    let position = in.position_raw.xy;

    var mask = item.mask;
    mask = vec4<f32>(
        mask.xy * 2.0 - 1.0,
        mask.zw * 2.0
    );

    
    var color = item.texture_transform;

    // exclude anything outside the mask box
    if (position.x < mask.x || position.y < mask.y || position.x > (mask.z + mask.x) || position.y > (mask.w + mask.y)) { 
        color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    // exclude anything outside the mask border radius
    var start = item.offset_scale.xy * 2.0 - 1.0;
    var size = vec3<f32>(item.offset_scale.zw * 2.0, 0.0);
    var border_radius = item.border_radius;
    color.a *= calculate_border_radius_mask(border_radius.x, vec2<f32>(-1.0, 1.0), start + size.zy, size.xy, position);
    color.a *= calculate_border_radius_mask(border_radius.y, vec2<f32>(1.0, 1.0), start + size.xy, size.xy, position);
    color.a *= calculate_border_radius_mask(border_radius.z, vec2<f32>(-1.0, -1.0), start + size.zz, size.xy, position);
    color.a *= calculate_border_radius_mask(border_radius.w, vec2<f32>(1.0, -1.0), start + size.xz, size.xy, position);

    // mask border radius
    border_radius = item.border_radius_mask;
    start = mask.xy;
    size = vec3<f32>(mask.zw, 0.0);
    color.a *= calculate_border_radius_mask(border_radius.x, vec2<f32>(-1.0, 1.0), start + size.zy, size.xy, position);
    color.a *= calculate_border_radius_mask(border_radius.y, vec2<f32>(1.0, 1.0), start + size.xy, size.xy, position);
    color.a *= calculate_border_radius_mask(border_radius.z, vec2<f32>(-1.0, -1.0), start + size.zz, size.xy, position);
    color.a *= calculate_border_radius_mask(border_radius.w, vec2<f32>(1.0, -1.0), start + size.xz, size.xy, position);
    



    output.color0 = color;

    return output;
}