use {
    bevy::{
        core::FixedTimestep,
        prelude::*,
        sprite::collide_aabb::{collide, Collision},
    },
    breakout::background::ColoredMesh2dPlugin,
};

/// An implementation of the classic game "Breakout"
const TIME_STEP: f32 = 1.0 / 60.0;
const SPRITE_Z: f32 = 1.0;
const BALL_SIZE: f32 = 20.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Scoreboard { score: 0 })
        .insert_resource(ClearColor(Color::rgb(0.9, 0.9, 0.9)))
        .add_plugin(ColoredMesh2dPlugin)
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(paddle_movement_system)
                .with_system(ball_collision_system)
                .with_system(ball_movement_system)
                .with_system(brick_movement_system),
        )
        .add_system(scoreboard_system)
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}

#[derive(Component)]
struct Paddle {
    eye_sprite: Handle<Sprite>,
    speed: f32,
    just_bounced: Option<f32>,
}

#[derive(Component)]
struct PaddleEye {
    is_left: bool,
}

#[derive(Component)]
struct Ball {
    velocity: Vec3,
    rotation: f32,
    just_bounced: Option<f32>,
}

#[derive(Component, Default)]
struct Brick {
    velocity: Option<Vec3>,
    just_bounced: Option<f32>,
}

#[derive(Component)]
enum Collider {
    Solid,
    Scorable,
    Paddle,
}

struct Scoreboard {
    score: usize,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Add the game's entities to our world

    // cameras
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());
    // paddle
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, -215.0, SPRITE_Z),
                scale: Vec3::new(120.0, 30.0, 0.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: Color::rgb(0.5, 0.5, 1.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Paddle {
            eye_sprite: asset_server.load("sprites/eye.png"),
            speed: 500.0,
            just_bounced: None,
        })
        .insert(Collider::Paddle);
    // paddle left eye
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, -215.0, SPRITE_Z),
                scale: Vec3::new(0.25, 0.25, 0.0),
                ..Default::default()
            },
            texture: asset_server.load("sprites/eye.png"),
            ..Default::default()
        })
        .insert(PaddleEye { is_left: true });

    // ball
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform {
                scale: Vec3::new(BALL_SIZE, BALL_SIZE, 0.0),
                translation: Vec3::new(0.0, -50.0, SPRITE_Z),
                ..Default::default()
            },
            sprite: Sprite {
                color: Color::rgb(0.2, 0.3, 1.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Ball {
            velocity: 400.0 * Vec3::new(0.5, -0.5, 0.0).normalize(),
            rotation: 0.0,
            just_bounced: None,
        });
    // scoreboard
    commands.spawn_bundle(TextBundle {
        text: Text {
            sections: vec![
                TextSection {
                    value: "Score: ".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.5, 0.5, 1.0),
                    },
                },
                TextSection {
                    value: "".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(1.0, 0.5, 0.5),
                    },
                },
            ],
            ..Default::default()
        },
        style: Style {
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    });

    // Add walls
    let wall_color = Color::rgb(0.8, 0.8, 0.8);
    let wall_thickness = 30.0;
    let bounds = Vec2::new(960.0, 680.0);

    // left
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(-bounds.x / 2.0, 0.0, SPRITE_Z),
                scale: Vec3::new(wall_thickness, bounds.y + wall_thickness, 1.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: wall_color,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Collider::Solid);
    // right
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(bounds.x / 2.0, 0.0, SPRITE_Z),
                scale: Vec3::new(wall_thickness, bounds.y + wall_thickness, 1.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: wall_color,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Collider::Solid);
    // bottom
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, -bounds.y / 2.0, SPRITE_Z),
                scale: Vec3::new(bounds.x + wall_thickness, wall_thickness, 1.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: wall_color,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Collider::Solid);
    // top
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, bounds.y / 2.0, SPRITE_Z),
                scale: Vec3::new(bounds.x + wall_thickness, wall_thickness, 1.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: wall_color,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Collider::Solid);

    // Add bricks
    let brick_rows = 4;
    let brick_columns = 5;
    let brick_spacing = 20.0;
    let brick_size = Vec3::new(150.0, 30.0, 1.0);
    let bricks_width = brick_columns as f32 * (brick_size.x + brick_spacing) - brick_spacing;
    // center the bricks and move them up a bit
    let bricks_offset = Vec3::new(-(bricks_width - brick_size.x) / 2.0, 100.0, 0.0);
    let brick_color = Color::rgb(0.5, 0.5, 1.0);
    for row in 0..brick_rows {
        let y_position = row as f32 * (brick_size.y + brick_spacing);
        for column in 0..brick_columns {
            let brick_position = Vec3::new(
                column as f32 * (brick_size.x + brick_spacing),
                y_position,
                SPRITE_Z,
            ) + bricks_offset;
            // brick
            commands
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: brick_color,
                        ..Default::default()
                    },
                    transform: Transform {
                        translation: brick_position,
                        scale: brick_size,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                // .insert(Collider::Scorable)
                .insert(Brick {
                    just_bounced: None,
                    ..Default::default()
                });
        }
    }
}

fn paddle_movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Paddle, &mut Transform)>,
) {
    let (paddle, mut transform) = query.single_mut();
    let mut direction = 0.0;
    if keyboard_input.pressed(KeyCode::Left) {
        direction -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::Right) {
        direction += 1.0;
    }

    let translation = &mut transform.translation;
    // move the paddle horizontally
    translation.x += direction * paddle.speed * TIME_STEP;
    // bound the paddle within the walls
    translation.x = translation.x.min(380.0).max(-380.0);
}

fn ball_movement_system(mut ball_query: Query<(&mut Ball, &mut Transform)>) {
    let (mut ball, mut transform) = ball_query.single_mut();
    let vel = ball.velocity * TIME_STEP;
    transform.translation += vel;
    // transform.rotation = transform.rotation.add(Quat::from_rotation_x(0.01));
    ball.rotation += 8.0 * TIME_STEP;
    transform.rotation = Quat::from_rotation_z(ball.rotation);
    if let Some(ref mut t) = ball.just_bounced {
        // double speed
        transform.translation += 0.3 * vel;
        const SCALE: f32 = 0.95;
        transform.scale = Vec3::new(BALL_SIZE * (1.0 + *t), BALL_SIZE * (1.0 + *t), 0.0);
        if 1.0 - SCALE < *t {
            *t *= SCALE;
        } else {
            ball.just_bounced = None;
        }
    }
}

fn scoreboard_system(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text>) {
    let mut text = query.single_mut();
    text.sections[1].value = format!("{}", scoreboard.score);
}

fn brick_movement_system(
    mut commands: Commands,
    mut bricks: Query<(Entity, &mut Brick, &mut Transform)>,
) {
    const SCALE: f32 = 0.95;
    for (entity, mut brick, mut trans) in bricks.iter_mut() {
        if let Some(ref mut t) = &mut brick.just_bounced {
            if 1.0 - SCALE < *t {
                *t *= SCALE;
                trans.scale *= SCALE;
            } else {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn ball_collision_system(
    mut commands: Commands,
    mut scoreboard: ResMut<Scoreboard>,
    mut ball_query: Query<(&mut Ball, &Transform)>,
    mut brick_query: Query<(&mut Brick, &Transform)>,
    collider_query: Query<(Entity, &Collider, &Transform)>,
) {
    let (mut ball, ball_transform) = ball_query.single_mut();
    let ball_size = ball_transform.scale.truncate();
    let velocity = &mut ball.velocity;
    let mut collided = false;

    // check collision with walls
    for (collider_entity, collider, transform) in collider_query.iter() {
        let collision = collide(
            ball_transform.translation,
            ball_size,
            transform.translation,
            transform.scale.truncate(),
        );
        if let Some(collision) = collision {
            collided = true;
            // scorable colliders should be despawned and increment the scoreboard on collision
            if let Collider::Scorable = *collider {
                scoreboard.score += 1;
                commands.entity(collider_entity).despawn();
            }

            // reflect the ball when it collides
            let mut reflect_x = false;
            let mut reflect_y = false;

            // only reflect if the ball's velocity is going in the opposite direction of the
            // collision
            match collision {
                Collision::Left => reflect_x = velocity.x > 0.0,
                Collision::Right => reflect_x = velocity.x < 0.0,
                Collision::Top => reflect_y = velocity.y < 0.0,
                Collision::Bottom => reflect_y = velocity.y > 0.0,
            }

            // reflect velocity on the x-axis if we hit something on the x-axis
            if reflect_x {
                velocity.x = -velocity.x;
            }

            // reflect velocity on the y-axis if we hit something on the y-axis
            if reflect_y {
                velocity.y = -velocity.y;
            }

            // break if this collide is on a solid, otherwise continue check whether a solid is
            // also in collision
            if let Collider::Solid = *collider {
                break;
            }
        }
    }
    // check collision with brick
    for (mut brick, transform) in brick_query.iter_mut() {
        let collision = collide(
            ball_transform.translation,
            ball_size,
            transform.translation,
            transform.scale.truncate(),
        );
        if let Some(collision) = collision {
            collided = true;
            scoreboard.score += 1;
            // commands.entity(collider_entity).despawn();
            brick.just_bounced = Some(1.0);

            // reflect the ball when it collides
            let mut reflect_x = false;
            let mut reflect_y = false;

            // only reflect if the ball's velocity is going in the opposite direction of the
            // collision
            match collision {
                Collision::Left => reflect_x = velocity.x > 0.0,
                Collision::Right => reflect_x = velocity.x < 0.0,
                Collision::Top => reflect_y = velocity.y < 0.0,
                Collision::Bottom => reflect_y = velocity.y > 0.0,
            }

            // reflect velocity on the x-axis if we hit something on the x-axis
            if reflect_x {
                velocity.x = -velocity.x;
            }

            // reflect velocity on the y-axis if we hit something on the y-axis
            if reflect_y {
                velocity.y = -velocity.y;
            }
        }
    }
    if collided {
        ball.just_bounced = Some(1.0);
    }
}
