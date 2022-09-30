use std::f32::consts::PI;

use bevy::prelude::*;
use rand::Rng;

#[derive(Component)]
struct Bullet;

#[derive(Component)]
struct Collider;

#[derive(Default)]
struct CollisionEvent;

#[derive(Default)]
struct Player(SpriteBundle);

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct StartText;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum AppState {
    Start,
    Playing,
}

#[derive(Default)]
struct Game {
    player: Option<Entity>,
    score: u128,
    timer: Timer,
}

fn main() {
    App::new()
        .init_resource::<Game>()
        .add_plugins(DefaultPlugins)
        .add_state(AppState::Start)
        .add_startup_system(setup)
        .add_system_set(SystemSet::on_enter(AppState::Start).with_system(setup_start))
        .add_system_set(SystemSet::on_update(AppState::Start).with_system(start))
        .add_system_set(SystemSet::on_enter(AppState::Playing).with_system(clean))
        .add_system_set(
            SystemSet::on_update(AppState::Playing)
                .with_system(tick)
                .with_system(score_update.after(tick))
                .with_system(bullet_spawn.after(tick))
                .with_system(bullet_movements.after(tick))
                .with_system(player_movements.after(tick))
                .with_system(check_for_collisions.after(tick)),
        )
        .add_system(bevy::window::close_on_esc)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut game: ResMut<Game>) {
    commands.spawn_bundle(Camera2dBundle::default());

    // Backdrop
    commands.spawn_bundle(SpriteBundle {
        transform: Transform {
            scale: Vec3::new(1015., 1015., 0.1),
            ..default()
        },
        sprite: Sprite {
            color: Color::WHITE,
            ..default()
        },
        ..default()
    });
    commands.spawn_bundle(SpriteBundle {
        transform: Transform {
            scale: Vec3::new(1010., 1010., 0.2),
            ..default()
        },
        sprite: Sprite {
            color: Color::BLACK,
            ..default()
        },
        ..default()
    });

    game.player = Some(
        commands
            .spawn()
            .insert_bundle(SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(0., 0., 0.3),
                    scale: Vec3::new(10., 10., 10.),
                    ..default()
                },
                sprite: Sprite {
                    color: Color::rgb(0.9, 0., 0.),
                    ..default()
                },
                ..default()
            })
            .insert(Collider)
            .id(),
    );

    game.timer = Timer::from_seconds(0.2, true);

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                align_content: AlignContent::FlexEnd,
                justify_content: JustifyContent::FlexEnd,
                ..default()
            },
            color: UiColor(Color::NONE),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        border: UiRect::all(Val::Px(2.0)),
                        padding: UiRect::all(Val::Px(10.)),
                        ..default()
                    },
                    color: UiColor(Color::GRAY),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(
                        TextBundle::from_section(
                            // Accepts a `String` or any type that converts into a `String`, such as `&str`
                            "Score: 0",
                            TextStyle {
                                font: asset_server.load("DejaVuSans.ttf"),
                                font_size: 50.0,
                                color: Color::BLACK,
                            },
                        )
                        .with_text_alignment(TextAlignment::TOP_CENTER), // Set the alignment of the Text
                    );
                });
        });
}

fn setup_start(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                align_content: AlignContent::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            color: UiColor(Color::NONE),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(
                    TextBundle::from_section(
                        // Accepts a `String` or any type that converts into a `String`, such as `&str`
                        "Press Space to Start",
                        TextStyle {
                            font: asset_server.load("DejaVuSans.ttf"),
                            font_size: 50.0,
                            color: Color::WHITE,
                        },
                    )
                    .with_text_alignment(TextAlignment::TOP_CENTER), // Set the alignment of the Text
                )
                .insert(StartText);
        });
}

fn start(mut state: ResMut<State<AppState>>, keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.pressed(KeyCode::Space) {
        state.set(AppState::Playing).unwrap();
    }
}

fn clean(
    mut commands: Commands,
    mut game: ResMut<Game>,
    bullets: Query<Entity, With<Bullet>>,
    start_texts: Query<Entity, With<StartText>>,
) {
    for entity in &bullets {
        commands.entity(entity).despawn_recursive();
    }
    for entity in &start_texts {
        commands.entity(entity).despawn_recursive();
    }
    game.score = 0;
}

fn tick(time: Res<Time>, mut game: ResMut<Game>) {
    game.timer.tick(time.delta()).just_finished();
    game.score += time.delta().as_millis();
}

fn score_update(game: Res<Game>, mut query: Query<&mut Text>) {
    for mut text in &mut query {
        text.sections[0].value = format!("Score: {}", game.score.to_string());
    }
}

fn bullet_spawn(
    mut commands: Commands,
    game: ResMut<Game>,
    asset_server: Res<AssetServer>,
    mut transforms: Query<&mut Transform>,
) {
    if game.timer.just_finished() {
        let player_transform = transforms.get_mut(game.player.unwrap()).unwrap();
        let rand_angle = rand::thread_rng().gen_range(0..(PI * 2000.) as u32) as f32 / 1000.;
        let rand_quad = Quat::from_rotation_z(rand_angle);
        let spawn_location = rand_quad * Vec3::new(1., 0., 0.).normalize() * 700.;
        let diff = player_transform.translation - spawn_location;
        let angle = diff.y.atan2(diff.x);
        commands
            .spawn_bundle(SpriteBundle {
                texture: asset_server.load("bullet.png"),
                transform: Transform {
                    translation: spawn_location,
                    rotation: Quat::from_rotation_z(angle),
                    scale: Vec3::new(0.1, 0.1, 0.1),
                    ..default()
                },
                sprite: Sprite {
                    color: Color::rgb(0.9, 0.9, 0.2),
                    ..default()
                },
                ..default()
            })
            .insert(Bullet)
            .insert(Velocity(
                Vec2::new(diff.x, diff.y).normalize() * Vec2::new(1000., 1000.),
            ))
            .insert(Collider);
    }
}

fn bullet_movements(
    time: Res<Time>,
    mut commands: Commands,
    mut bullet_velocities: Query<(Entity, &Velocity, &mut Transform), With<Bullet>>,
) {
    for (entity, velocity, mut transform) in &mut bullet_velocities {
        transform.translation.x += velocity.0.x * time.delta_seconds();
        transform.translation.y += velocity.0.y * time.delta_seconds();

        const LIMIT: f32 = 1000.;
        if transform.translation.x > LIMIT || transform.translation.x < -LIMIT {
            commands.entity(entity).despawn_recursive();
        }
        if transform.translation.y > LIMIT || transform.translation.y < -LIMIT {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn player_movements(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    game: ResMut<Game>,
    mut transforms: Query<&mut Transform>,
) {
    const PLAYER_SPEED: f32 = 250.;
    let mut player_transform = transforms.get_mut(game.player.unwrap()).unwrap();
    let mut direction = Vec3::new(0., 0., 0.);
    if keyboard_input.pressed(KeyCode::E) {
        direction.x += 1.;
    };
    if keyboard_input.pressed(KeyCode::U) {
        direction.x -= 1.;
    };
    if keyboard_input.pressed(KeyCode::P) {
        direction.y += 1.;
    };
    if keyboard_input.pressed(KeyCode::I) {
        direction.y -= 1.;
    };
    if direction.length() > 0. {
        player_transform.translation += direction.normalize() * PLAYER_SPEED * time.delta_seconds();
    }

    player_transform.translation.x = player_transform.translation.x.clamp(-500., 500.);
    player_transform.translation.y = player_transform.translation.y.clamp(-500., 500.);
}

fn check_for_collisions(
    time: Res<Time>,
    mut state: ResMut<State<AppState>>,
    game: ResMut<Game>,
    assets: Res<Assets<Image>>,
    transforms: Query<&Transform>,
    collider_query: Query<(Entity, &Handle<Image>, &Transform, &Velocity, &Collider), With<Bullet>>,
) {
    let player_transform = transforms.get(game.player.unwrap()).unwrap();

    for (_collider_entity, texture_handle, transform, velocity, _collider) in &collider_query {
        if let Some(texture) = assets.get(texture_handle) {
            let size = texture.size() * transform.scale.truncate();
            if collide_with_rotation_multistep(
                time.clone(),
                player_transform.translation.truncate(),
                transform.translation.truncate(),
                size,
                transform.rotation,
                velocity.0,
                5,
            ) {
                println!("Score was: {}", game.score);
                state.overwrite_set(AppState::Start).unwrap();
            }
        }
    }
}

// ============== COLLISION DETECTION ==============

fn collide_with_rotation_multistep(
    time: Time,
    point: Vec2,
    rectangle_position: Vec2,
    rectangle_size: Vec2,
    rectangle_rotation: Quat,
    rectangle_velocity: Vec2,
    steps: u16,
) -> bool {
    for i in 0..steps {
        if collide_with_rotation(
            point,
            rectangle_position
                - (i as f32 / steps as f32) * rectangle_velocity * time.clone().delta_seconds(),
            rectangle_size,
            rectangle_rotation,
        ) {
            return true;
        }
    }
    false
}

fn collide_with_rotation(
    point: Vec2,
    rectangle_position: Vec2,
    rectangle_size: Vec2,
    rectangle_rotation: Quat,
) -> bool {
    let point_1 = Vec3::new(-rectangle_size.x / 2., rectangle_size.y / 2., 0.);
    let point_2 = Vec3::new(-rectangle_size.x / 2., -rectangle_size.y / 2., 0.);
    let point_3 = Vec3::new(rectangle_size.x / 2., rectangle_size.y / 2., 0.);
    let point_4 = Vec3::new(rectangle_size.x / 2., -rectangle_size.y / 2., 0.);
    is_point_inside_rectangle(
        point,
        rectangle_rotation.mul_vec3(point_1).truncate() + rectangle_position,
        rectangle_rotation.mul_vec3(point_2).truncate() + rectangle_position,
        rectangle_rotation.mul_vec3(point_3).truncate() + rectangle_position,
        rectangle_rotation.mul_vec3(point_4).truncate() + rectangle_position,
    )
}

fn is_point_inside_rectangle(t: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, p4: Vec2) -> bool {
    is_point_inside_triangle(t, p1, p2, p3) || is_point_inside_triangle(t, p2, p4, p3)
}

fn is_point_inside_triangle(t: Vec2, p1: Vec2, p2: Vec2, p3: Vec2) -> bool {
    let area_ref = area(p1, p2, p3);

    let area_1 = area(p1, p2, t);
    let area_2 = area(p1, t, p3);
    let area_3 = area(t, p2, p3);
    area_ref >= area_1 + area_2 + area_3
}

fn area(p1: Vec2, p2: Vec2, p3: Vec2) -> f32 {
    return ((p1.x * (p2.y - p3.y) + p2.x * (p3.y - p1.y) + p3.x * (p1.y - p2.y)) / 2.).abs();
}
