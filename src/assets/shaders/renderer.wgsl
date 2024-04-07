
struct VertexOutput {
    @builtin(position) position: vec4<f32>
};

struct FragmentOutput {
    @location(0) color0: vec4<f32>
};


struct RendererGPUData {
    color           : vec4<f32>
};

@group(0) @binding(0) var<uniform> rendererGPUData : RendererGPUData;

@vertex
fn vs_main(
    @builtin(vertex_index) index: u32,
) -> VertexOutput {
    var output: VertexOutput;

    let a = (index >> 2u) & 1u;
    let b = (index >> 1u) & 1u;
    let c = index & 1u;

    let tx = f32(b | (a & c));
    let ty = f32(a | (~b & c));

    output.position = vec4<f32>(tx * 2.0 - 1.0, ty * 2.0 - 1.0, 0.0, 1.0);
    output.position.y *= 0.8;
    output.position.x *= 0.7;

    return output;
}

@fragment
fn fs_main() -> FragmentOutput {
    var output: FragmentOutput;
    // output.color0 = vec4<f32>(1.0, 0.0, 0.0, 1.0);
    output.color0 = rendererGPUData.color;
    return output;
}
