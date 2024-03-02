use bevy::{prelude::*, window::PrimaryWindow};

const WINDOW_WIDTH: f32 = 1200.0;
const WINDOW_HEIGHT: f32 = 720.0;

#[derive(Component)]
struct Phase(u8);

#[derive(Component)]
struct MainCamera;

fn main() {
    println!("Hello, jam!");
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Aberration".into(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::ALICE_BLUE))
        .add_systems(Startup, (spawn_camera, spawn_shapes))
        .add_systems(Update, (bevy::window::close_on_esc, mouse_motion))
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MainCamera));
}

fn spawn_shapes(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("test_shapes.png");
    let layout = TextureAtlasLayout::from_grid(Vec2::new(100.0, 100.0), 3, 2, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    for x in (-240..=240).step_by(120) {
        for y in (-240..=240).step_by(120) {
            commands.spawn((
                SpriteSheetBundle {
                    texture: texture.clone(),
                    atlas: TextureAtlas {
                        layout: texture_atlas_layout.clone(),
                        index: 0,
                    },
                    transform: Transform::from_xyz(x as f32, y as f32, 0.0),
                    ..default()
                },
                Phase(0),
            ));
        }
    }
}

fn mouse_motion(
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut query: Query<(&mut Phase, &mut TextureAtlas, &Transform)>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
) {
    let (camera, camera_transform) = q_camera.single();
    let window = q_window.single();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        for (mut phase, mut sprite, transform) in &mut query {
            if mouse_button_input.just_pressed(MouseButton::Left)
                && (transform.translation.x - world_position.x).abs() < 50.0
                && (transform.translation.y - world_position.y).abs() < 50.0
            {
                phase.0 += 1;
                sprite.index = if phase.0 < 5 { phase.0.into() } else { 4 };
            }
        }
    }
}
