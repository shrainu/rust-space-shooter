use bevy::prelude::*;

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
        .add_system(player_movement_input)
        .add_system(move_entity)
        .add_system(lock_bounded_entity)
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
struct Bounded {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Component)]
struct Speed(f32);

#[derive(Component)]
struct Direction {
    x: f32,
    y: f32,
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
        });
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
