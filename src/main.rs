#![feature(drain_filter)]

use bevy::prelude::*;
use rand::prelude::*;

// Collision Layers
const COLLISION_LAYER_PLAYER: u8 = 0b00000001;
const COLLISION_LAYER_ENEMY: u8 = 0b00000010;

// Stars
const STAR_SPAWN_TIME: u64 = 500;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: String::from("Bevy Shooter"),
            width: 600f32,
            height: 800f32,
            resizable: false,
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::rgb(0.06, 0.025, 0.125)))
        .add_startup_system(setup_camera)
        .add_startup_system(spawn_player)
        .add_plugins(DefaultPlugins)
        .add_event::<OnCollisionEnterEvent>()
        .add_system_to_stage(CoreStage::First, spawn_enemy_system)
        .add_system_to_stage(CoreStage::PreUpdate, entity_healthbar_added)
        .add_system_to_stage(CoreStage::PreUpdate, spawn_star_system)
        .add_system(player_movement_input)
        .add_system(move_entity)
        .add_system(lock_bounded_entity)
        .add_system(entity_shoot_projectile)
        .add_system(check_collision_entity)
        .add_system(on_projectile_collision_enter)
        .add_system(entity_lifespan_system)
        .add_system(destroy_out_of_window_system)
        .add_system(entity_health_system)
        .add_system(entity_healthbar_system)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

// Tags
#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct Projectile;

#[derive(Component)]
struct Bounded {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Component)]
struct Speed(f32);

#[derive(Component, Clone, Copy)]
struct Direction {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct Shooter {
    projectile_color: Color,
    projectile_size: Vec2,
    projectile_direction: Direction,
    projectile_speed: f32,
    projectile_lifespan: u64,
    shoot_positions: Vec<Vec2>,
}

#[derive(Component)]
struct AutoShoot {
    shoot_interval: u64,
    current_interval: u64,
}

#[derive(Component, Clone, Copy)]
struct PhysicsBody {
    self_layer_mask: u8,
    target_layer_maks: u8,
}

#[derive(Component)]
struct Collider {
    collided: Vec<Entity>,
}

#[derive(Component)]
struct Lifespan {
    lifespan: u64,
    current: u64,
}

#[derive(Component)]
struct Health {
    max: i32,
    current: i32,
}

#[derive(Component)]
struct HealthBar;

struct OnCollisionEnterEvent {
    this: Entity,
    other: Entity,
}

#[derive(Component)]
struct Star;

#[derive(Component)]
struct DestroyOutOfWindow;

fn spawn_player(windows: Res<Windows>, mut commands: Commands) {
    let window = windows.get_primary().unwrap();

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.2, 0.1, 0.65),
                ..Default::default()
            },
            transform: Transform {
                scale: Vec3::new(60.0, 60.0, 1.0),
                translation: Vec3::new(0.0, 0.0, 2.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Player)
        .insert(Speed(10.0))
        .insert(Direction { x: 0.0, y: 1.0 })
        .insert(Bounded {
            x: 0.0 - window.width() / 2.0,
            y: 0.0 - window.height() / 2.0,
            width: window.width(),
            height: window.height(),
        })
        .insert(AutoShoot {
            shoot_interval: 750,
            current_interval: 0,
        })
        .insert(Shooter {
            shoot_positions: vec![Vec2::new(28.0, 32.0), Vec2::new(-28.0, 32.0)],
            projectile_direction: Direction { x: 0.0, y: 1.0 },
            projectile_color: Color::rgb(0.0, 1.0, 0.0),
            projectile_size: Vec2::new(4.0, 12.0),
            projectile_speed: 12.0,
            projectile_lifespan: 2000,
        })
        .insert(PhysicsBody {
            self_layer_mask: COLLISION_LAYER_PLAYER,
            target_layer_maks: COLLISION_LAYER_ENEMY,
        })
        .insert(Collider { collided: vec![] });
}

fn spawn_enemy(window: &Window, mut commands: Commands) {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.4, 0.1, 0.6),
                ..Default::default()
            },
            transform: Transform {
                scale: Vec3::new(60.0, 60.0, 2.0),
                translation: Vec3::new(
                    rand::thread_rng().gen_range(-270.0..270.0f32),
                    rand::thread_rng().gen_range(460.0..=520.0f32),
                    1.0,
                ),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Bounded {
            x: 0.0 - window.width() / 2.0,
            y: 0.0 - window.height(),
            width: window.width(),
            height: window.height() * 2.0,
        })
        .insert(PhysicsBody {
            self_layer_mask: COLLISION_LAYER_ENEMY,
            target_layer_maks: COLLISION_LAYER_PLAYER,
        })
        .insert(Collider { collided: vec![] })
        .insert(Health { max: 5, current: 5 })
        .insert(HealthBar)
        .insert(Enemy)
        .insert(Speed(2.5))
        .insert(Direction { x: 0.0, y: -1.0 })
        .insert(DestroyOutOfWindow);
}

fn spawn_enemy_system(windows: Res<Windows>, time: Res<Time>, mut commands: Commands) {
    static mut CURRENT: u64 = 0;
    const SPAWN_INTERVAL: u64 = 5000;

    let mut spawn = false;

    let window = windows.get_primary().unwrap();

    unsafe {
        CURRENT += time.delta().as_millis() as u64;
        if CURRENT >= SPAWN_INTERVAL {
            spawn = true;
            CURRENT = 0;
        }
    }

    if spawn {
        spawn_enemy(window, commands);
    }
}

fn player_movement_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Direction, With<Player>>,
) {
    for mut direction in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::A) {
            direction.x = -1.0;
        } else if keyboard_input.pressed(KeyCode::D) {
            direction.x = 1.0;
        } else {
            direction.x = 0.0;
        }

        if keyboard_input.pressed(KeyCode::S) {
            direction.y = -1.0;
        } else if keyboard_input.pressed(KeyCode::W) {
            direction.y = 1.0;
        } else {
            direction.y = 0.0;
        }
    }
}

fn move_entity(mut query: Query<(&Speed, &Direction, &mut Transform)>) {
    for (speed, dir, mut transform) in query.iter_mut() {
        if dir.x == 0.0 && dir.y == 0.0 {
            continue;
        }

        let change = Vec2::new(dir.x, dir.y).normalize() * speed.0;

        transform.translation.x += change.x;
        transform.translation.y += change.y;
    }
}

fn lock_bounded_entity(mut query: Query<(&Bounded, &mut Transform)>) {
    for (bounded, mut transform) in query.iter_mut() {
        if transform.translation.x + transform.scale.x / 2.0 > bounded.x + bounded.width {
            transform.translation.x = bounded.x + bounded.width - transform.scale.x / 2.0;
        } else if transform.translation.x - transform.scale.x / 2.0 < bounded.x {
            transform.translation.x = bounded.x + transform.scale.x / 2.0;
        }

        if transform.translation.y + transform.scale.y / 2.0 > bounded.y + bounded.height {
            transform.translation.y = bounded.y + bounded.height - transform.scale.y / 2.0;
        } else if transform.translation.y - transform.scale.y / 2.0 < bounded.y {
            transform.translation.y = bounded.y + transform.scale.y / 2.0;
        }
    }
}

fn entity_shoot_projectile(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(&mut AutoShoot, &Shooter, &PhysicsBody, &Transform)>,
) {
    for (mut auto, shooter, body, transform) in query.iter_mut() {
        auto.current_interval += time.delta().as_millis() as u64;
        if auto.current_interval >= auto.shoot_interval {
            auto.current_interval = 0;

            for pos in shooter.shoot_positions.iter() {
                commands
                    .spawn_bundle(SpriteBundle {
                        sprite: Sprite {
                            color: shooter.projectile_color,
                            ..Default::default()
                        },
                        transform: Transform {
                            scale: Vec3::new(
                                shooter.projectile_size.x,
                                shooter.projectile_size.y,
                                1.0,
                            ),
                            translation: Vec3::new(
                                transform.translation.x + pos.x,
                                transform.translation.y + pos.y,
                                0.0,
                            ),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(PhysicsBody { ..*body })
                    .insert(Collider { collided: vec![] })
                    .insert(Projectile)
                    .insert(Direction {
                        ..shooter.projectile_direction
                    })
                    .insert(Lifespan {
                        lifespan: shooter.projectile_lifespan,
                        current: 0u64,
                    })
                    .insert(Speed(shooter.projectile_speed));
            }
        }
    }
}

fn rect_to_rect_collision(rect: &Transform, other: &Transform) -> bool {
    let collision_x = rect.translation.x + rect.scale.x / 2.0
        >= other.translation.x - other.scale.x / 2.0
        && other.translation.x + other.scale.x / 2.0 >= rect.translation.x - rect.scale.x / 2.0;
    let collision_y = rect.translation.y + rect.scale.y / 2.0
        >= other.translation.y - other.scale.y / 2.0
        && other.translation.y + other.scale.y / 2.0 >= rect.translation.y - rect.scale.y / 2.0;

    return collision_x && collision_y;
}

fn check_collision_entity(
    mut query: Query<(Entity, &mut Collider, &PhysicsBody, &Transform)>,
    mut collision_events: EventWriter<OnCollisionEnterEvent>,
) {
    let mut collisions: Vec<(Entity, Entity)> = vec![];

    for (entity, _, body, transform) in query.iter() {
        for (other_entity, _, other_body, other_transform) in query.iter() {
            if entity == other_entity {
                continue;
            }

            if let Ok(_) = query.get_component::<Player>(entity) {
                println!(
                    "Player position : {}, {}",
                    transform.translation, transform.scale
                );
            }
            if body.target_layer_maks & other_body.self_layer_mask == 0 {
                continue;
            }

            let collided = rect_to_rect_collision(transform, other_transform);
            let collision = (entity, other_entity);

            if collided && !collisions.contains(&collision) {
                collisions.push(collision);
            }
        }
    }

    for (e1, e2) in collisions.iter() {
        for (e, mut collider, _, _) in query.iter_mut() {
            if *e1 == e {
                if !collider.collided.contains(&e2) {
                    collider.collided.push(*e2);

                    collision_events.send(OnCollisionEnterEvent {
                        this: *e1,
                        other: *e2,
                    });
                }
            }
        }
    }

    for (e, mut collider, _, _) in query.iter_mut() {
        collider.collided = collider
            .collided
            .drain_filter(|other| collisions.contains(&(e, *other)))
            .collect::<Vec<_>>();
    }
}

fn on_projectile_collision_enter(
    mut commands: Commands,
    mut collision_events: EventReader<OnCollisionEnterEvent>,
    query: Query<(Entity, &Projectile)>,
    mut other: Query<&mut Health, With<Collider>>,
) {
    for event in collision_events.iter() {
        for (entity, _) in query.iter() {
            if entity == event.this {
                if let Ok(mut health) = other.get_component_mut::<Health>(event.other) {
                    health.as_mut().current -= 1;
                    commands.entity(event.this).despawn();
                } else {
                    commands.entity(event.other).despawn();
                }
            }
        }
    }
}

fn entity_lifespan_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Lifespan)>,
) {
    let mut to_delete: Vec<Entity> = vec![];

    for (entity, mut lifespan) in query.iter_mut() {
        lifespan.current += time.delta().as_millis() as u64;
        if lifespan.current >= lifespan.lifespan {
            to_delete.push(entity);
        }
    }

    for entity in to_delete {
        commands.entity(entity).despawn();
    }
}

fn entity_health_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Health), Changed<Health>>,
) {
    for (entity, health) in query.iter_mut() {
        if health.current <= 0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn entity_healthbar_added(mut commands: Commands, query: Query<Entity, Added<HealthBar>>) {
    for entity in query.iter() {
        let child = commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 0.0, 0.0),
                    ..Default::default()
                },
                transform: Transform {
                    scale: Vec3::new(1.0, 0.1, 1.0),
                    translation: Vec3::new(0.0, 0.6, 1.0),
                    ..Default::default()
                },
                ..Default::default()
            })
            .id();

        commands.entity(entity).add_child(child);
    }
}

fn entity_healthbar_system(
    mut child_query: Query<(&Parent, &mut Transform)>,
    parent_query: Query<(&Health, With<HealthBar>), Changed<Health>>,
) {
    if parent_query.is_empty() {
        return;
    }

    for (parent, mut transform) in child_query.iter_mut() {
        let parent_health = parent_query.get(parent.0);
        if let Ok((health, _)) = parent_health {
            let ratio = health.current as f32 / health.max as f32;

            transform.scale.x = ratio;
            transform.translation.x = ratio / 2.0 - 0.5;
        }
    }
}

fn spawn_star_system(mut commands: Commands, time: Res<Time>) {
    static mut CURRENT: u64 = 0;

    let mut spawn = false;

    unsafe {
        CURRENT += time.delta().as_millis() as u64;
        if CURRENT >= STAR_SPAWN_TIME {
            spawn = true;
            CURRENT = 0;
        }
    }

    if spawn {
        let spawn_count: i32 = rand::thread_rng().gen_range(10..=25);
        for _ in 0..spawn_count {
            let spawn_pos = Vec3::new(
                rand::thread_rng().gen_range(-300.0..=300.0),
                rand::thread_rng().gen_range(420.0..500.0),
                1.0,
            );

            let size: f32 = rand::thread_rng().gen_range(2.0..=5.0);

            let speed: f32 = rand::thread_rng().gen_range(2.0..=8.0) * (size / 5.0);

            commands
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgb(1.0, 1.0, 1.0),
                        ..Default::default()
                    },
                    transform: Transform {
                        translation: spawn_pos,
                        scale: Vec3::new(size, size, 1.0),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Star)
                .insert(Speed(speed))
                .insert(Direction { x: 0.0, y: -1.0 });
        }
    }
}

fn destroy_out_of_window_system(
    windows: Res<Windows>,
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<DestroyOutOfWindow>>,
) {
    let window = windows.get_primary().unwrap();

    for (entity, transform) in query.iter() {
        if transform.translation.y + (transform.scale.y / 2.0) <= 0.0 - (window.height() / 2.0) {
            commands.entity(entity).despawn_recursive();
        }
    }
}
