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
        .add_systems(Startup, setup)
        // .add_systems(PostStartup, setup)
        .add_systems(Update, animate_sprite)
        .add_systems(Update, exit_on_esc)
        .run();
}

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut Sprite)>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());

        if timer.just_finished()
            && let Some(atlas) = &mut sprite.texture_atlas
        {
            atlas.index = if atlas.index == indices.last {
                indices.first
            } else {
                atlas.index + 1
            };
        }
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
    commands.spawn((
        Sprite::from_atlas_image(
            texture,
            TextureAtlas {
                layout: texture_atlas_layout,
                index: animation_indices.first,
            },
        ),
        Transform::from_xyz(-25.0, 18.5, 0.0).with_scale(Vec3::splat(5.0)),
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        RigidBody::Dynamic,
        Collider::cuboid(18.0, 17.5),
    ));

    // ground
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(
            WINDOW_WIDTH as f32,
            WINDOW_HEIGHT as f32 / 3.0,
        ))),
        MeshMaterial2d(materials.add(Color::from(DARK_GRAY))),
        Transform::from_xyz(0.0, -1.0 * WINDOW_HEIGHT as f32 / 3.0, 0.0),
        Collider::cuboid(WINDOW_WIDTH as f32 / 2.0, WINDOW_HEIGHT as f32 / 6.0),
    ));
}

fn exit_on_esc(keyboard_input: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        commands.write_message(AppExit::Success);
    }
}
