#import bevy_pbr::mesh_view_bindings

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    let uv = position.xy / vec2<f32>(view.width, view.height);
    let color = vec4<f32>(uv.x, uv.y, uv.x.min(uv.y), 0.0);
    return color;
}
