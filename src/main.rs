#![feature(drain_filter)]

use bevy::prelude::*;

// Collision Layers
const COLLISION_LAYER_PLAYER: u8 = 0b00000001;
const COLLISION_LAYER_ENEMY: u8 = 0b00000010;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: String::from("Bevy Shooter"),
            width: 600f32,
            height: 800f32,
            resizable: false,
            ..Default::default()
        })
        .add_startup_system(setup_camera)
        .add_startup_system(spawn_player)
        .add_plugins(DefaultPlugins)
        .add_event::<OnCollisionEnterEvent>()
        .add_system(player_movement_input)
        .add_system(move_entity)
        .add_system(lock_bounded_entity)
        .add_system(entity_shoot_projectile)
        .add_system(check_collision_entity)
        .add_system(on_projectile_collision_enter)
        .add_system(entity_lifespan_system)
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

struct OnCollisionEnterEvent {
    this: Entity,
    other: Entity,
}

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
            shoot_interval: 1000,
            current_interval: 0,
        })
        .insert(Shooter {
            shoot_positions: vec![Vec2::new(32.0, 32.0), Vec2::new(-32.0, 32.0)],
            projectile_direction: Direction { x: 0.0, y: 1.0 },
            projectile_color: Color::rgb(0.0, 1.0, 0.0),
            projectile_size: Vec2::new(4.0, 12.0),
            projectile_speed: 12.0,
            projectile_lifespan: 2000
        })
        .insert(PhysicsBody {
            self_layer_mask: COLLISION_LAYER_PLAYER,
            target_layer_maks: COLLISION_LAYER_ENEMY,
        })
        .insert(Collider { collided: vec![] });
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
    let collision_x = rect.translation.x + rect.scale.x / 2.0 >= other.translation.x
        && other.translation.x + other.scale.x / 2.0 >= rect.translation.x;
    let collision_y = rect.translation.y + rect.scale.y / 2.0 >= other.translation.y
        && other.translation.y + other.scale.y / 2.0 >= rect.translation.y;

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
) {
    for event in collision_events.iter() {
        for (entity, _) in query.iter() {
            if entity == event.this {
                commands.entity(event.other).despawn();
                // commands.entity(event.this).despawn(); TODO: Uncomment later
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
