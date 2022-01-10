// Import the standard 2d mesh uniforms and set their bind groups
#import bevy_sprite::mesh2d_view_bind_group
[[group(0), binding(0)]]
var<uniform> view: View;
#import bevy_sprite::mesh2d_struct
[[group(1), binding(0)]]
var<uniform> mesh: Mesh2d;

// The structure of the vertex buffer is as specified in `specialize()`
struct Vertex {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] color: vec4<f32>;
};

struct Time {
    time_since_startup: f32;
};
[[group(2), binding(0)]]
var<uniform> time: Time;

struct VertexOutput {
    // The vertex shader must set the on-screen position of the vertex
    [[builtin(position)]] clip_position: vec4<f32>;
    // We pass the vertex color to the framgent shader in location 0
    [[location(0)]] position: vec3<f32>;
};

[[stage(vertex)]]
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    let world_position = mesh.model * vec4<f32>(vertex.position, 1.0);
    // let world_position = vec4<f32>(vertex.position, 1.0);
    let position = view.view_proj * world_position;
    out.clip_position = position;
    out.position = vertex.position;
    return out;
}

fn oklab_to_linear_srgb(c: vec3<f32>) -> vec3<f32> {
    let L = c.x;
    let a = c.y;
    let b = c.z;

    let l_ = L + 0.3963377774 * a + 0.2158037573 * b;
    let m_ = L - 0.1055613458 * a - 0.0638541728 * b;
    let s_ = L - 0.0894841775 * a - 1.2914855480 * b;
    let l = l_*l_*l_;
    let m = m_*m_*m_;
    let s = s_*s_*s_;
    return vec3<f32>(
		 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
		-1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
		-0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
    );
}

struct FragmentInput {
    // The color is interpolated between vertices by default
    [[location(0)]] position: vec3<f32>;
};

[[stage(fragment)]]
fn fragment(in: FragmentInput) -> [[location(0)]] vec4<f32> {
    let speed = 0.57;
    let time_since_startup = time.time_since_startup;
    let t_1 = sin(time_since_startup * speed) * 0.5 + 0.5;
    let t_2 = cos(time_since_startup * speed);

    let pos = vec2<f32>(in.position.x, in.position.y);
    let distance_to_center = distance(pos, vec2<f32>(t_2, t_1)) * 1.2;

    // blending is done in a perceptual color space: https://bottosson.github.io/posts/oklab/
    let red = vec3<f32>(0.627955, 0.224863, 0.125846);
    let green = vec3<f32>(0.86644, -0.233887, 0.179498);
    let blue = vec3<f32>(0.701674, 0.274566, -0.169156);
    let white = vec3<f32>(1.0, 0.0, 0.0);
    let mixed = mix(mix(red, blue, t_1), mix(green, white, t_2), distance_to_center);

    return vec4<f32>(oklab_to_linear_srgb(mixed), 1.0);
    // return in.color;
}
