use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, MaterialMesh2dBundle},
};

pub fn setup_background(
    asset_server: Res<AssetServer>,
    mut custom_materials: ResMut<Assets<CustomMaterial>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let rect = Mesh::from(shape::Quad::default());
    commands.spawn_bundle(MaterialMesh2dBundle {
        mesh: meshes.add(rect).into(),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        // material: materials.add(ColorMaterial::from(Color::PURPLE)),
        material: custom_materials.add(CustomMaterial {
            // texture: asset_server.load("sprites/eye.png"),
        }),
        ..default()
    });
}

#[derive(AsBindGroup, Debug, Clone, TypeUuid)]
#[uuid = "b62bb455-a72c-4b56-87bb-81e0554e234f"]
pub struct CustomMaterial {
    // #[texture(0)]
    // #[sampler(1)]
    // texture: Handle<Image>,
}

impl Material2d for CustomMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/mesh2d.wgsl".into()
    }
    fn fragment_shader() -> ShaderRef {
        // "shaders/screen_bg.wgsl".into()
        "shaders/mesh2d.wgsl".into()
    }
}
