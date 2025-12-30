use bevy::{color::palettes::css::DARK_GRAY, prelude::*};
use bevy_rapier2d::prelude::*;
const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;

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
        .insert_resource(ClearColor(Color::srgb(0.9, 0.9, 0.9)))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Slasher".to_string(),
                        resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                        canvas: Some("#bevy".to_owned()), // Bind to canvas included in `index.html`
                        prevent_default_event_handling: false, // Tells wasm not to override default event handling
                        resizable: false,
                        focused: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()), // prevents blurry sprites
        )
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .insert_state(Animation::Idle)
        .add_systems(Startup, setup)
        .add_systems(Update, exit_on_esc)
        .add_systems(Update, jump)
        .add_systems(Update, run)
        .add_systems(Update, slash)
        .add_systems(Update, check_falling.run_if(in_state(Animation::Jumping)))
        .add_systems(Update, check_landed.run_if(in_state(Animation::Falling)))
        .add_systems(
            Update,
            check_stopped_running.run_if(in_state(Animation::Running)),
        )
        .add_systems(
            Update,
            animate_sprite
                .before(jump)
                .before(run)
                .before(slash)
                .before(check_falling)
                .before(check_landed)
                .before(check_stopped_running),
        )
        .run();
}

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum Animation {
    Idle,
    Jumping,
    Falling,
    Running,
    Slashing,
}

impl Animation {
    fn indices(&self) -> AnimationIndices {
        match self {
            Animation::Idle => AnimationIndices { first: 0, last: 3 },
            Animation::Falling => AnimationIndices {
                first: 22,
                last: 23,
            },
            Animation::Jumping => AnimationIndices {
                first: 69,
                last: 71,
            },
            Animation::Running => AnimationIndices { first: 8, last: 13 },
            Animation::Slashing => AnimationIndices {
                first: 42,
                last: 45,
            },
        }
    }
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Component)]
struct Player;

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&mut AnimationIndices, &mut AnimationTimer, &mut Sprite), With<Player>>,
    mut animations: MessageReader<StateTransitionEvent<Animation>>,
    player_state: Res<State<Animation>>,
) {
    if let Ok((mut indices, mut timer, mut sprite)) = query.single_mut()
        && let Some(atlas) = &mut sprite.texture_atlas
    {
        timer.tick(time.delta());

        if timer.just_finished() {
            atlas.index = if atlas.index == indices.last {
                let new_indices = player_state.indices();
                indices.first = new_indices.first;
                indices.last = new_indices.last;
                indices.first
            } else {
                atlas.index + 1
            };
        }
        if let Some(animation_transition) = animations.read().last() {
            if let Some(animation) = &animation_transition.entered
                && &animation_transition.exited != &animation_transition.entered
            {
                let new_indices = animation.indices();
                indices.first = new_indices.first;
                indices.last = new_indices.last;
                atlas.index = indices.first;
                timer.reset();
            }
        };
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    // player sprite
    let texture = asset_server.load("adventurer.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::new(50, 37), 7, 11, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let animation_indices = AnimationIndices { first: 8, last: 13 }; // run animation frames
    commands
        .spawn((
            Sprite::from_atlas_image(
                texture,
                TextureAtlas {
                    layout: texture_atlas_layout,
                    index: animation_indices.first,
                },
            ),
            Transform::from_xyz(-25.0, WINDOW_HEIGHT as f32 / 2.0, 0.0)
                .with_scale(Vec3::splat(5.0)),
            animation_indices,
            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            RigidBody::Dynamic,
            Player,
            Velocity::zero(),
            LockedAxes::ROTATION_LOCKED,
        ))
        .with_children(|children| {
            children
                .spawn(Collider::cuboid(8.0, 13.0))
                .insert(Transform::from_xyz(0.0, -4.5, 0.0));
        });

    // ground
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(
            WINDOW_WIDTH as f32,
            WINDOW_HEIGHT as f32 / 3.0,
        ))),
        MeshMaterial2d(materials.add(Color::from(DARK_GRAY))),
        Transform::from_xyz(0.0, -1.0 * WINDOW_HEIGHT as f32 / 3.0, 0.0),
        Collider::cuboid(WINDOW_WIDTH as f32 / 2.0, WINDOW_HEIGHT as f32 / 6.0),
        RigidBody::Fixed,
    ));
}

fn exit_on_esc(keyboard_input: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        commands.write_message(AppExit::Success);
    }
}

fn jump(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_velocity: Query<&mut Velocity, With<Player>>,
    mut next_state: ResMut<NextState<Animation>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) || keyboard_input.just_pressed(KeyCode::ArrowUp)
    {
        let mut velocity = player_velocity.single_mut().unwrap();
        if velocity.linvel.y == 0.0 {
            velocity.linvel.y += 600.0;
            next_state.set(Animation::Jumping);
        }
    }
}

fn check_falling(
    mut player_velocity: Query<&mut Velocity, With<Player>>,
    mut next_state: ResMut<NextState<Animation>>,
) {
    let velocity = player_velocity.single_mut().unwrap();
    if velocity.linvel.y < 0.0 {
        next_state.set(Animation::Falling);
    }
}

fn check_landed(
    mut player_velocity: Query<&mut Velocity, With<Player>>,
    mut next_state: ResMut<NextState<Animation>>,
) {
    let velocity = player_velocity.single_mut().unwrap();
    if velocity.linvel.y == 0.0 {
        next_state.set(Animation::Idle);
    }
}

fn check_stopped_running(
    mut player_velocity: Query<&mut Velocity, With<Player>>,
    mut next_state: ResMut<NextState<Animation>>,
) {
    let velocity = player_velocity.single_mut().unwrap();
    if velocity.linvel.x == 0.0 {
        next_state.set(Animation::Idle);
    }
}

fn run(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player: Query<(&mut Velocity, &mut Sprite), With<Player>>,
    mut next_state: ResMut<NextState<Animation>>,
) {
    let (mut velocity, mut sprite) = player.single_mut().unwrap();
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        velocity.linvel.x = -500.0;
        sprite.flip_x = true;
        if velocity.linvel.y == 0.0 {
            next_state.set(Animation::Running);
        }
    } else if keyboard_input.pressed(KeyCode::ArrowRight) {
        velocity.linvel.x = 500.0;
        sprite.flip_x = false;
        if velocity.linvel.y == 0.0 {
            next_state.set(Animation::Running);
        }
    } else {
        // TODO: smoother deceleration, player should respond smoothly to unpressing run or unpressing jump
        velocity.linvel.x = 0.0;
    }
}

fn slash(keyboard_input: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<Animation>>) {
    if keyboard_input.just_pressed(KeyCode::KeyA) {
        next_state.set(Animation::Slashing);
    }
}
