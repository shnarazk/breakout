#import bevy_pbr::mesh_view_bindings

// @group(1) @binding(0)
// var texture: texture_2d<f32>;
// @group(1) @binding(1)
// var texture_sampler: sampler;

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    let uv = position.xy / vec2<f32>(view.width, view.height);
    // let color1 = textureSample(texture, texture_sampler, uv);
    let color = vec4<f32>(uv.x, uv.y, min(uv.x, uv.y), 0.0);
    // let color = vec4<f32>(color1[0], color2[1], color1[2], color2[2]);
    return color;
}
