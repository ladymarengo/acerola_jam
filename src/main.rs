use bevy::{prelude::*, window::PrimaryWindow};

const WINDOW_WIDTH: f32 = 1200.0;
const WINDOW_HEIGHT: f32 = 720.0;

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
        .insert_resource(SelectedEntity(None))
        .insert_resource(CursorCoords(None))
        .add_systems(Startup, (spawn_camera, spawn_shapes))
        .add_systems(
            Update,
            (
                bevy::window::close_on_esc,
                update_cursor_coords,
                mouse_motion,
                update_cells_position,
                spawn_new_cells,
            ),
        )
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MainCamera));
}

fn update_cursor_coords(
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut cursor_coords: ResMut<CursorCoords>,
) {
    let (camera, camera_transform) = q_camera.single();
    let window = q_window.single();

    cursor_coords.0 = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate());
}

fn update_cells_position(mut query: Query<(&mut Transform, Entity), With<Cell>>) {
    let mut possible_drop = None;
    for (transform, entity) in &query {
        let mut drop = true;
        for (other_transform, other_entity) in &query {
            if transform.translation.y == -240.0
                || (entity != other_entity
                    && transform.translation.x == other_transform.translation.x
                    && transform.translation.y == other_transform.translation.y + 120.0)
            {
                drop = false;
            }
        }
        if drop {
            possible_drop = Some(entity);
            break;
        }
    }
    if let Some(entity) = possible_drop {
        if let Ok((mut transform, _entity)) = query.get_mut(entity) {
            transform.translation.y -= 20.0;
        }
    }
}

fn spawn_new_cells(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    query: Query<&Transform, With<Cell>>,
) {
    let texture = asset_server.load("test_shapes.png");
    let layout = TextureAtlasLayout::from_grid(Vec2::new(100.0, 100.0), 3, 2, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    for x in (-240..=240).step_by(120) {
        if !query.iter().any(|transform| {
            transform.translation.x == x as f32 && transform.translation.y >= 240.0
        }) {
            commands.spawn((
                SpriteSheetBundle {
                    texture: texture.clone(),
                    atlas: TextureAtlas {
                        layout: texture_atlas_layout.clone(),
                        index: 0,
                    },
                    transform: Transform::from_xyz(x as f32, 360.0, 0.0),
                    ..default()
                },
                Phase(0),
                Cell,
            ));
        }
    }
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
                Cell,
            ));
        }
    }
}

fn mouse_motion(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    cursor_coords: Res<CursorCoords>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut selected: ResMut<SelectedEntity>,
    mut query: Query<(&mut Phase, &mut TextureAtlas, &Transform, Entity)>,
    texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    if let Some(cursor_coords) = cursor_coords.0 {
        if mouse_button_input.just_pressed(MouseButton::Left) {
            for (mut phase, mut sprite, transform, entity) in &mut query {
                if (transform.translation.x - cursor_coords.x).abs() < 50.0
                    && (transform.translation.y - cursor_coords.y).abs() < 50.0
                {
                    if let Some(selection) = &selected.0 {
                        if selection.entity == entity {
                            commands.entity(selection.sprite).despawn();
                            selected.0 = None;
                        } else if selection.phase == phase.0
                            && (transform.translation.x - selection.coords.x).abs() < 130.0
                            && (transform.translation.y - selection.coords.y).abs() < 130.0
                        {
                            commands.entity(selection.entity).despawn();
                            commands.entity(selection.sprite).despawn();
                            selected.0 = None;
                            phase.0 += 1;
                            sprite.index = if phase.0 < 5 { phase.0.into() } else { 4 };
                        }
                    } else {
                        let sprite = commands
                            .spawn(SpriteBundle {
                                texture: asset_server.load("test_selection.png"),
                                transform: Transform::from_xyz(
                                    transform.translation.x,
                                    transform.translation.y,
                                    1.0,
                                ),
                                ..default()
                            })
                            .id();
                        selected.0 = Some(SelectionOptions {
                            entity,
                            sprite,
                            phase: phase.0,
                            coords: transform.translation,
                        });
                    }
                }
            }
        } else if mouse_button_input.just_pressed(MouseButton::Right) {
            for (_phase, _sprite, _transform, entity) in &query {
                commands.entity(entity).despawn();
            }
            spawn_shapes(commands, asset_server, texture_atlas_layouts);
        }
    }
}

#[derive(Component)]
struct Phase(u8);

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct Cell;

#[derive(Resource)]
struct SelectedEntity(Option<SelectionOptions>);

struct SelectionOptions {
    entity: Entity,
    sprite: Entity,
    phase: u8,
    coords: Vec3,
}

#[derive(Resource)]
struct CursorCoords(Option<Vec2>);
