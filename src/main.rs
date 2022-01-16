use {
    bevy::{
        core::FixedTimestep,
        prelude::*,
        sprite::collide_aabb::{collide, Collision},
    },
    breakout::background::ColoredMesh2dPlugin,
    rand::prelude::random,
};

/// An implementation of the classic game "Breakout"
const TIME_STEP: f32 = 1.0 / 60.0;
const SPRITE_Z: f32 = 1.0;
const BALL_SIZE: f32 = 20.0;
const EYE_DIST: f32 = 30.0;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Breakout+".to_string(),
            width: 980.0,
            height: 710.0,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .insert_resource(Scoreboard {
            score: 0,
            remain_bricks: 20,
            brick_in_row: 1,
            keeping: false,
            just_changed: None,
        })
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
        .add_system(bonus_notifier_system)
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}

#[derive(Component)]
struct Paddle {
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

#[derive(Component, Eq, PartialEq)]
enum Collider {
    Solid,
    Paddle,
}

#[derive(Component, Default)]
struct TextScoreBoard;

#[derive(Component, Default)]
struct TextBonus {
    show: Option<f32>,
    row: usize,
}

struct Scoreboard {
    score: usize,
    remain_bricks: usize,
    brick_in_row: usize,
    keeping: bool,
    just_changed: Option<f32>,
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
                translation: Vec3::new(0.0, -230.0, SPRITE_Z),
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
            speed: 500.0,
            just_bounced: None,
        })
        .insert(Collider::Paddle);
    // paddle left eye
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(-EYE_DIST, -230.0, SPRITE_Z + 0.1),
                scale: Vec3::new(0.25, 0.25, 0.0),
                ..Default::default()
            },
            texture: asset_server.load("sprites/eye.png"),
            ..Default::default()
        })
        .insert(PaddleEye { is_left: true });

    // paddle left black eye
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(-EYE_DIST, -230.0, SPRITE_Z + 0.2),
                scale: Vec3::new(0.25, 0.25, 0.0),
                ..Default::default()
            },
            texture: asset_server.load("sprites/black-eye.png"),
            ..Default::default()
        })
        .insert(PaddleEye { is_left: true });

    // paddle right eye
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(EYE_DIST, -230.0, SPRITE_Z),
                scale: Vec3::new(0.25, 0.25, 0.0),
                ..Default::default()
            },
            texture: asset_server.load("sprites/eye.png"),
            ..Default::default()
        })
        .insert(PaddleEye { is_left: false });

    // paddle right black eye
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(EYE_DIST, -230.0, SPRITE_Z + 0.2),
                scale: Vec3::new(0.25, 0.25, 0.0),
                ..Default::default()
            },
            texture: asset_server.load("sprites/black-eye.png"),
            ..Default::default()
        })
        .insert(PaddleEye { is_left: false });
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
    commands
        .spawn_bundle(TextBundle {
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
                    top: Val::Percent(60.0),
                    left: Val::Percent(45.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(TextScoreBoard::default());

    // bonus notifier
    commands
        .spawn_bundle(TextBundle {
            text: Text::with_section(
                "+1".to_string(),
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 100.0,
                    color: Color::rgba(1.0, 0.2, 0.0, 0.8),
                },
                TextAlignment {
                    horizontal: HorizontalAlign::Center,
                    ..Default::default()
                },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Percent(35.0),
                    left: Val::Percent(40.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(TextBonus::default());

    // Add walls
    let wall_color = Color::rgb(0.8, 0.8, 0.8);
    let wall_thickness = 35.0;
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

#[allow(clippy::type_complexity)]
fn paddle_movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut queries: QuerySet<(
        QueryState<(&mut Paddle, &mut Transform)>,
        QueryState<(&mut PaddleEye, &mut Transform)>,
    )>,
) {
    let just_bounced: Option<f32>;
    let p_pos: f32;
    let mut paddle = queries.q0();
    let (mut paddle, mut transform) = paddle.single_mut();
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
    p_pos = translation.x;
    // bound the paddle within the walls
    translation.x = translation.x.clamp(-400.0, 400.0);
    just_bounced = paddle.just_bounced;
    if let Some(ref mut t) = paddle.just_bounced {
        if 0.1 < *t {
            *t *= 0.8;
        } else {
            paddle.just_bounced = None;
        }
    }

    // move eyes
    let mut eyes = queries.q1();
    for (eye, mut trans) in eyes.iter_mut() {
        if eye.is_left {
            trans.translation.x = p_pos - EYE_DIST;
        } else {
            trans.translation.x = p_pos + EYE_DIST;
        }
        if let Some(ref t) = just_bounced {
            if 0.1 < *t {
                trans.scale.x = 0.25 + 0.5 * *t;
                trans.scale.y = 0.25 + 0.5 * *t;
            } else {
                trans.scale.x = 0.25;
                trans.scale.y = 0.25;
            }
        }
    }
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

fn scoreboard_system(
    mut scoreboard: ResMut<Scoreboard>,
    mut query: Query<(&mut Text, &mut Style), With<TextScoreBoard>>,
) {
    let (mut text, mut style) = query.single_mut();
    let remains = scoreboard.remain_bricks;
    let score = scoreboard.score;
    if let Some(ref mut t) = scoreboard.just_changed {
        style.display = Display::Flex;
        if 0.1 < *t {
            text.sections[1].value = format!("{}", score);
            if 0 < remains {
                *t *= 0.9;
            }
        } else {
            scoreboard.just_changed = None;
        }
    } else {
        style.display = Display::None;
    }
}

fn bonus_notifier_system(mut bonus_query: Query<(&mut Text, &mut Style, &mut TextBonus)>) {
    let (mut text, mut style, mut bonus) = bonus_query.single_mut();
    let point = bonus.row;
    if let Some(ref mut t) = bonus.show {
        style.display = Display::Flex;
        if 0.1 < *t {
            text.sections[0].value = format!("+{}", point);
            text.sections[0].style.font_size = 100.0 * (2.0 - *t);
            text.sections[0].style.color = Color::rgba(1.0, 0.2, 0.0, *t);
            *t *= 0.9;
        } else {
            bonus.show = None;
        }
    } else {
        style.display = Display::None;
    }
}

fn brick_movement_system(
    mut commands: Commands,
    mut bricks: Query<(Entity, &mut Brick, &mut Transform)>,
) {
    const SCALE: f32 = 0.94;
    for (entity, mut brick, mut trans) in bricks.iter_mut() {
        let velocity = brick.velocity;
        if let Some(ref mut t) = &mut brick.just_bounced {
            if 1.0 - SCALE < *t {
                *t *= SCALE;
                if let Some(v) = velocity {
                    trans.translation += *t * 0.6 * TIME_STEP * v;
                }
                trans.rotation = Quat::from_rotation_z(0.4 * random::<f32>());
                trans.scale *= 0.99; // SCALE;
            } else {
                // scorable colliders should be despawned and increment the scoreboard on collision
                commands.entity(entity).despawn();
            }
        }
    }
}

fn ball_collision_system(
    mut scoreboard: ResMut<Scoreboard>,
    mut paddle_query: Query<&mut Paddle>,
    mut ball_query: Query<(&mut Ball, &Transform)>,
    mut brick_query: Query<(&mut Brick, &Transform)>,
    mut bonus_query: Query<&mut TextBonus>,
    collider_query: Query<(&Collider, &Transform)>,
) {
    let (mut ball, ball_transform) = ball_query.single_mut();
    let ball_size = ball_transform.scale.truncate();
    let velocity = &mut ball.velocity;
    let mut penalty = 0;
    let mut collided = false;
    let mut collided_with_paddle = false;
    let mut score_changed = false;

    // check collision with walls
    for (collider, transform) in collider_query.iter() {
        let collision = collide(
            ball_transform.translation,
            ball_size,
            transform.translation,
            transform.scale.truncate(),
        );
        if let Some(collision) = collision {
            collided = true;
            scoreboard.keeping = false;

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

            if let Collider::Paddle = *collider {
                if matches!(collision, Collision::Bottom) {
                    penalty = 2;
                }
                collided_with_paddle = true;
            } else if let Collider::Solid = *collider {
                // break if this collide is on a solid, otherwise continue check
                // whether a solid is also in collision
                if matches!(collision, Collision::Top) && reflect_y {
                    scoreboard.brick_in_row = 1;
                    scoreboard.keeping = false;
                    penalty = penalty.max(1);
                }
                break;
            }
        }
    }
    match penalty {
        _ if 0 == scoreboard.remain_bricks => (),
        2 => {
            scoreboard.score /= 2;
            score_changed = true;
        }
        1 if 0 < scoreboard.score => {
            scoreboard.score -= 1;
            score_changed = true;
        }
        _ => (),
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
            if brick.just_bounced.is_some() {
                continue;
            }
            collided = true;
            if 0 < scoreboard.remain_bricks {
                if scoreboard.keeping {
                    scoreboard.brick_in_row += 1;
                    if 1 < scoreboard.brick_in_row {
                        let mut bonus = bonus_query.single_mut();
                        bonus.row = scoreboard.brick_in_row;
                        bonus.show = Some(2.0);
                    }
                }
                scoreboard.keeping = true;
                scoreboard.score += scoreboard.brick_in_row;
                scoreboard.remain_bricks -= 1;
                if 0 == scoreboard.remain_bricks {
                    scoreboard.just_changed = Some(100.0);
                }
                score_changed = true;
            }
            // commands.entity(collider_entity).despawn();
            if brick.just_bounced.is_none() {
                brick.velocity = Some(*velocity);
                brick.just_bounced = Some(1.0);
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
        }
    }
    if collided {
        ball.just_bounced = Some(1.0);
    }
    // check collision with paddle
    if collided_with_paddle {
        for mut paddle in paddle_query.iter_mut() {
            paddle.just_bounced = Some(1.0);
        }
    }
    if score_changed && scoreboard.just_changed.is_none() {
        scoreboard.just_changed = Some(4.0);
    }
}
