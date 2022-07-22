use bevy::{
    core::FloatOrd,
    core_pipeline::Transparent2d,
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    prelude::*,
    render::{
        mesh::{Indices, MeshVertexBufferLayout},
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, EntityRenderCommand, RenderCommandResult, RenderPhase,
            SetItemPipeline, TrackedRenderPass,
        },
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        // texture::BevyDefault,
        view::VisibleEntities,
        RenderApp,
        RenderStage,
    },
    sprite::{
        DrawMesh2d, Mesh2dHandle, Mesh2dPipeline, Mesh2dPipelineKey, Mesh2dUniform,
        SetMesh2dBindGroup, SetMesh2dViewBindGroup,
    },
};

pub fn setup_background(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mut rect = Mesh::new(PrimitiveTopology::TriangleList);
    let v_pos: Vec<[f32; 3]> = vec![
        [-1.0, -1.0, 0.0],
        [-1.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        [1.0, -1.0, 0.0],
    ];
    rect.insert_attribute(Mesh::ATTRIBUTE_POSITION, v_pos);
    let mut v_color = vec![[0.0, 0.0, 0.0, 1.0]];
    v_color.extend_from_slice(&[[1.0, 1.0, 0.0, 1.0]; 3]);
    rect.insert_attribute(Mesh::ATTRIBUTE_COLOR, v_color);
    let mut indices = vec![0, 1, 4];
    for i in 2..=4 {
        indices.extend_from_slice(&[0, i, i - 1]);
    }
    rect.set_indices(Some(Indices::U32(indices)));
    commands.spawn_bundle((
        ColoredMesh2d::default(),
        Mesh2dHandle(meshes.add(rect)),
        // Transform::default(),
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)).with_scale(Vec3::splat(700.0)),
        // Transform::default().with_scale(Vec3::splat(128.)),
        GlobalTransform::default(),
        Visibility::default(),
        ComputedVisibility::default(),
    ));
}

/// A marker component for colored 2d meshes
#[derive(Component, Default)]
pub struct ColoredMesh2d;

/// Custom pipeline for 2d meshes with vertex colors
pub struct ColoredMesh2dPipeline {
    shader: Handle<Shader>,
    /// this pipeline wraps the standard [`Mesh2dPipeline`]
    mesh2d_pipeline: Mesh2dPipeline,
    time_bind_group_layout: BindGroupLayout,
}

impl FromWorld for ColoredMesh2dPipeline {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let shader = asset_server.load("shaders/background.wgsl");
        let render_device = world.resource_mut::<RenderDevice>();
        let time_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("time bind group"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(std::mem::size_of::<f32>() as u64),
                    },
                    count: None,
                }],
            });
        let mesh2d_pipeline = world.resource::<Mesh2dPipeline>().clone();

        Self {
            shader,
            mesh2d_pipeline,
            time_bind_group_layout,
        }
    }
}

// We implement `SpecializedPipeline` to customize the default rendering from `Mesh2dPipeline`
impl SpecializedMeshPipeline for ColoredMesh2dPipeline {
    type Key = Mesh2dPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayout,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh2d_pipeline.specialize(key, layout)?;
        descriptor.vertex.shader = self.shader.clone();
        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();
        descriptor.layout = Some(vec![
            self.mesh2d_pipeline.view_layout.clone(),
            self.mesh2d_pipeline.mesh_layout.clone(),
            self.time_bind_group_layout.clone(),
        ]);
        Ok(descriptor)
    }
}

// This specifies how to render a colored 2d mesh
type DrawColoredMesh2d = (
    // Set the pipeline
    SetItemPipeline,
    // Set the view uniform as bind group 0
    SetMesh2dViewBindGroup<0>,
    // Set the mesh uniform as bind group 1
    SetMesh2dBindGroup<1>,
    // Set the time
    SetTimeBindGroup<2>,
    // Draw the mesh
    DrawMesh2d,
);

/// Plugin that renders [`ColoredMesh2d`]s
pub struct ColoredMesh2dPlugin;

impl Plugin for ColoredMesh2dPlugin {
    fn build(&self, app: &mut App) {
        let render_device = app.world.resource::<RenderDevice>();
        let buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("time uniform buffer"),
            size: std::mem::size_of::<f32>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Register our custom draw function and pipeline, and add our render systems
        let render_app = app.get_sub_app_mut(RenderApp).unwrap();
        render_app
            .add_render_command::<Transparent2d, DrawColoredMesh2d>()
            .insert_resource(TimeMeta {
                buffer,
                bind_group: None,
            })
            .init_resource::<ColoredMesh2dPipeline>()
            .init_resource::<SpecializedMeshPipelines<ColoredMesh2dPipeline>>()
            .add_system_to_stage(RenderStage::Extract, extract_time)
            .add_system_to_stage(RenderStage::Extract, extract_colored_mesh2d)
            .add_system_to_stage(RenderStage::Prepare, prepare_time)
            .add_system_to_stage(RenderStage::Queue, queue_colored_mesh2d)
            .add_system_to_stage(RenderStage::Queue, queue_time_bind_group);
        app.add_startup_system(setup_background);
    }
}

/// Extract the [`ColoredMesh2d`] marker component into the render app
pub fn extract_colored_mesh2d(
    mut commands: Commands,
    mut previous_len: Local<usize>,
    query: Query<(Entity, &ComputedVisibility), With<ColoredMesh2d>>,
) {
    let mut values = Vec::with_capacity(*previous_len);
    for (entity, computed_visibility) in query.iter() {
        if !computed_visibility.is_visible {
            continue;
        }
        values.push((entity, (ColoredMesh2d,)));
    }
    *previous_len = values.len();
    commands.insert_or_spawn_batch(values);
}

/// Queue the 2d meshes marked with [`ColoredMesh2d`] using our custom pipeline and draw function
#[allow(clippy::too_many_arguments)]
pub fn queue_colored_mesh2d(
    transparent_draw_functions: Res<DrawFunctions<Transparent2d>>,
    colored_mesh2d_pipeline: Res<ColoredMesh2dPipeline>,
    mut pipelines: ResMut<SpecializedMeshPipelines<ColoredMesh2dPipeline>>,
    mut pipeline_cache: ResMut<PipelineCache>,
    msaa: Res<Msaa>,
    render_meshes: Res<RenderAssets<Mesh>>,
    colored_mesh2d: Query<(&Mesh2dHandle, &Mesh2dUniform), With<ColoredMesh2d>>,
    mut views: Query<(&VisibleEntities, &mut RenderPhase<Transparent2d>)>,
) {
    if colored_mesh2d.is_empty() {
        return;
    }
    // Iterate each view (a camera is a view)
    for (visible_entities, mut transparent_phase) in views.iter_mut() {
        let draw_colored_mesh2d = transparent_draw_functions
            .read()
            .get_id::<DrawColoredMesh2d>()
            .unwrap();

        let mesh_key = Mesh2dPipelineKey::from_msaa_samples(msaa.samples);

        // Queue all entities visible to that view
        for visible_entity in &visible_entities.entities {
            if let Ok((mesh2d_handle, mesh2d_uniform)) = colored_mesh2d.get(*visible_entity) {
                // Get our specialized pipeline
                let mut mesh2d_key = mesh_key;
                if let Some(mesh) = render_meshes.get(&mesh2d_handle.0) {
                    mesh2d_key |=
                        Mesh2dPipelineKey::from_primitive_topology(mesh.primitive_topology);

                    let pipeline = pipelines
                        .specialize(
                            &mut pipeline_cache,
                            &colored_mesh2d_pipeline,
                            mesh2d_key,
                            &mesh.layout,
                        )
                        .unwrap();

                    let mesh_z = mesh2d_uniform.transform.w_axis.z;
                    transparent_phase.add(Transparent2d {
                        entity: *visible_entity,
                        draw_function: draw_colored_mesh2d,
                        pipeline,
                        // The 2d render items are sorted according to their z value before rendering,
                        // in order to get correct transparency
                        sort_key: FloatOrd(mesh_z),
                        // This material is not batched
                        batch_range: None,
                    });
                }
            }
        }
    }
}

#[derive(Default)]
struct ExtractedTime {
    seconds_since_startup: f32,
}

// extract the passed time into a resource in the render world
fn extract_time(mut commands: Commands, time: Res<Time>) {
    commands.insert_resource(ExtractedTime {
        seconds_since_startup: time.seconds_since_startup() as f32,
    });
}

struct TimeMeta {
    buffer: Buffer,
    bind_group: Option<BindGroup>,
}

// write the extracted time into the corresponding uniform buffer
fn prepare_time(
    time: Res<ExtractedTime>,
    time_meta: ResMut<TimeMeta>,
    render_queue: Res<RenderQueue>,
) {
    render_queue.write_buffer(
        &time_meta.buffer,
        0,
        bevy::core::cast_slice(&[time.seconds_since_startup]),
    );
}

// create a bind group for the time uniform buffer
fn queue_time_bind_group(
    render_device: Res<RenderDevice>,
    mut time_meta: ResMut<TimeMeta>,
    pipeline: Res<ColoredMesh2dPipeline>,
) {
    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &pipeline.time_bind_group_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: time_meta.buffer.as_entire_binding(),
        }],
    });
    time_meta.bind_group = Some(bind_group);
}

struct SetTimeBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetTimeBindGroup<I> {
    type Param = SRes<TimeMeta>;

    fn render<'w>(
        _view: Entity,
        _item: Entity,
        time_meta: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let time_bind_group = time_meta.into_inner().bind_group.as_ref().unwrap();
        pass.set_bind_group(I, time_bind_group, &[]);

        RenderCommandResult::Success
    }
}
