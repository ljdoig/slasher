use bevy::prelude::*;

// redirect println! to console.log in wasm
#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(target_family = "wasm")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[cfg(target_family = "wasm")]
custom_print::define_macros!({ cprintln }, concat, unsafe fn (crate::log)(&str));

#[cfg(target_family = "wasm")]
macro_rules! println { ($($args:tt)*) => { cprintln!($($args)*); } }

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, sprite_movement)
        .run();
}

#[derive(Component)]
enum Direction {
    Left,
    Right,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    commands.spawn((
        Sprite::from_image(asset_server.load("adventurer.png")),
        Transform::from_xyz(0., 0., 0.),
        Direction::Right,
    ));
}

/// The sprite is animated by changing its translation depending on the time that has passed since
/// the last frame.
fn sprite_movement(
    time: Res<Time>,
    mut sprite_position: Query<(&mut Direction, &mut Transform)>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    for (mut logo, mut transform) in &mut sprite_position {
        match *logo {
            Direction::Right => transform.translation.x += 150. * time.delta_secs(),
            Direction::Left => transform.translation.x -= 150. * time.delta_secs(),
        }

        if buttons.just_pressed(MouseButton::Left) {
            match *logo {
                Direction::Right => *logo = Direction::Left,
                Direction::Left => *logo = Direction::Right,
            }
        }

        if transform.translation.x > 200. {
            println!("Changing direction to Left");
            *logo = Direction::Left;
        } else if transform.translation.x < -200. {
            println!("Changing direction to Right");
            *logo = Direction::Right;
        }
    }
}
