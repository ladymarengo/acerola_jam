#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::{prelude::*, window::PrimaryWindow};
use rand::Rng;

const WINDOW_WIDTH: f32 = 1200.0;
const WINDOW_HEIGHT: f32 = 720.0;
const CELL_SIZE: f32 = 125.0;
const CELL_INTERVAL: f32 = 25.0;
const MAP_START_X: f32 = -360.0;
const MAP_START_Y: f32 = -225.0;

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
        .insert_resource(ClearColor(Color::Rgba {
            red: 0.604,
            green: 0.749,
            blue: 0.784,
            alpha: 1.0,
        }))
        .insert_resource(AnimationIndices {
            normal_calm: Indices { first: 0, last: 21 },
            normal_worried: Indices {
                first: 22,
                last: 43,
            },
            normal_scared: Indices {
                first: 44,
                last: 59,
            },
            corrupted_calm: Indices {
                first: 60,
                last: 79,
            },
            corrupted_happy: Indices {
                first: 80,
                last: 99,
            },
        })
        .insert_resource(SelectedEntity(None))
        .insert_resource(CursorCoords(None))
        .add_systems(Startup, (spawn_camera, spawn_smilers))
        .add_systems(
            Update,
            (
                bevy::window::close_on_esc,
                update_cursor_coords,
                mouse_input,
                update_cells_position,
                spawn_new_cells,
                update_corrupted_neighbors,
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

fn update_cells_position(mut query: Query<(&mut Transform, Entity), With<Smiler>>) {
    let mut possible_drop = None;
    for (transform, entity) in &query {
        let mut drop = true;
        for (other_transform, other_entity) in &query {
            if transform.translation.y == MAP_START_Y
                || (entity != other_entity
                    && transform.translation.x == other_transform.translation.x
                    && transform.translation.y
                        == other_transform.translation.y + CELL_SIZE + CELL_INTERVAL)
            {
                drop = false;
                break;
            }
        }
        if drop {
            possible_drop = Some(entity);
            break;
        }
    }
    if let Some(entity) = possible_drop {
        if let Ok((mut transform, _entity)) = query.get_mut(entity) {
            transform.translation.y -= 25.0;
        }
    }
}

fn spawn_smiler(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    corrupted: bool,
    x: f32,
    y: f32,
) {
    let texture_expr = asset_server.load("expressions.png");
    let layout_expr = TextureAtlasLayout::from_grid(Vec2::new(200.0, 200.0), 2, 1, None, None);
    let texture_atlas_layout_expr = texture_atlas_layouts.add(layout_expr);

    let texture_color = asset_server.load("colors.png");
    let layout_color = TextureAtlasLayout::from_grid(Vec2::new(200.0, 200.0), 3, 2, None, None);
    let texture_atlas_layout_color = texture_atlas_layouts.add(layout_color);

    commands
        .spawn((
            SpriteSheetBundle {
                texture: texture_expr.clone(),
                atlas: TextureAtlas {
                    layout: texture_atlas_layout_expr.clone(),
                    index: if corrupted { 1 } else { 0 },
                },
                transform: Transform::from_xyz(x, y, 1.0).with_scale(Vec3::splat(0.625)),
                ..default()
            },
            Smiler {
                phase: 0,
                corrupted,
                corrupted_neighbors: 0,
                state: if corrupted {
                    SmilerState::CorruptedCalm
                } else {
                    SmilerState::NormalCalm
                },
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                SpriteSheetBundle {
                    texture: texture_color.clone(),
                    atlas: TextureAtlas {
                        layout: texture_atlas_layout_color.clone(),
                        index: 0,
                    },
                    transform: Transform::from_xyz(0.0, 0.0, -5.0),
                    ..default()
                },
                SmilerColor,
            ));
        });
}

fn spawn_new_cells(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    query: Query<&Transform, With<Smiler>>,
) {
    let mut rng = rand::thread_rng();

    for x_coord in (MAP_START_X as i32..=(MAP_START_X + (CELL_SIZE + CELL_INTERVAL) * 3.0) as i32)
        .step_by(CELL_SIZE as usize + CELL_INTERVAL as usize)
    {
        if !query.iter().any(|transform| {
            transform.translation.x == x_coord as f32
                && transform.translation.y >= CELL_SIZE + CELL_INTERVAL
        }) {
            let corrupted = rng.gen::<f64>() < 0.7;
            spawn_smiler(
                &mut commands,
                &asset_server,
                &mut texture_atlas_layouts,
                corrupted,
                x_coord as f32,
                MAP_START_Y + (CELL_SIZE + CELL_INTERVAL) * 4.0,
            );
        }
    }
}

fn spawn_smilers(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for x_coord in (MAP_START_X as i32..=(MAP_START_X + (CELL_SIZE + CELL_INTERVAL) * 3.0) as i32)
        .step_by(CELL_SIZE as usize + CELL_INTERVAL as usize)
    {
        for y_coord in (MAP_START_Y as i32
            ..=(MAP_START_Y + (CELL_SIZE + CELL_INTERVAL) * 3.0) as i32)
            .step_by(CELL_SIZE as usize + CELL_INTERVAL as usize)
        {
            spawn_smiler(
                &mut commands,
                &asset_server,
                &mut texture_atlas_layouts,
                false,
                x_coord as f32,
                y_coord as f32,
            );

            commands.spawn(SpriteBundle {
                texture: asset_server.load("cell.png"),
                transform: Transform::from_xyz(x_coord as f32, y_coord as f32, -10.0)
                    .with_scale(Vec3::splat(0.625)),
                ..default()
            });
        }
    }
}

fn mouse_input(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    cursor_coords: Res<CursorCoords>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut selected: ResMut<SelectedEntity>,
    mut query: Query<
        (
            &mut Smiler,
            &mut TextureAtlas,
            &Transform,
            Entity,
            &Children,
        ),
        Without<SmilerColor>,
    >,
    texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut colors: Query<&mut TextureAtlas, With<SmilerColor>>,
) {
    if let Some(cursor_coords) = cursor_coords.0 {
        if mouse_button_input.just_pressed(MouseButton::Left) {
            for (mut smiler, mut sprite, transform, entity, children) in &mut query {
                if (transform.translation.x - cursor_coords.x).abs() < 50.0
                    && (transform.translation.y - cursor_coords.y).abs() < 50.0
                {
                    if let Some(selection) = &selected.0 {
                        if selection.entity == entity {
                            commands.entity(selection.sprite).despawn();
                            selected.0 = None;
                        } else if selection.phase == smiler.phase
                            && (transform.translation.x - selection.coords.x).abs()
                                <= (CELL_SIZE + CELL_INTERVAL)
                            && (transform.translation.y - selection.coords.y).abs()
                                <= (CELL_SIZE + CELL_INTERVAL)
                        {
                            smiler.phase += 1;

                            let mut rng = rand::thread_rng();
                            if (smiler.corrupted || selection.corrupted)
                                && !(smiler.corrupted && selection.corrupted)
                            {
                                smiler.corrupted = rng.gen::<f64>() < 0.9;
                            }
                            if smiler.phase < 6 {
                                let child = children.first().unwrap();
                                let mut color_sprite = colors.get_mut(*child).unwrap();
                                color_sprite.index += 1;
                            }
                            if smiler.corrupted && sprite.index < 1 {
                                sprite.index += 1;
                            }
                            commands.entity(selection.entity).despawn_recursive();
                            commands.entity(selection.sprite).despawn();
                            selected.0 = None;
                        }
                    } else {
                        let sprite = commands
                            .spawn(SpriteBundle {
                                texture: asset_server.load("selection.png"),
                                transform: Transform::from_xyz(
                                    transform.translation.x,
                                    transform.translation.y,
                                    1.0,
                                )
                                .with_scale(Vec3::splat(0.625)),
                                ..default()
                            })
                            .id();
                        selected.0 = Some(SelectionOptions {
                            entity,
                            sprite,
                            phase: smiler.phase,
                            coords: transform.translation,
                            corrupted: smiler.corrupted,
                        });
                    }
                }
            }
        } else if mouse_button_input.just_pressed(MouseButton::Right) {
            for (_smiler, _sprite, _transform, entity, _children) in &query {
                commands.entity(entity).despawn_recursive();
            }
            spawn_smilers(commands, asset_server, texture_atlas_layouts);
        }
    }
}

fn update_corrupted_neighbors(
    neighbors: Query<(&Transform, &Corrupted, Entity), With<Cell>>,
    mut smilers: Query<(&Transform, &mut CorruptedNeighbors, Entity), With<Cell>>,
) {
    for (smiler_coords, mut corrupted_neighbors, smiler_id) in &mut smilers {
        corrupted_neighbors.0 = 0;
        for (neighbor_coords, corrupted, neighbor_id) in &neighbors {
            if smiler_id != neighbor_id
                && corrupted.0
                && (smiler_coords.translation.x - neighbor_coords.translation.x).abs() < 130.0
                && (smiler_coords.translation.y - neighbor_coords.translation.y).abs() < 130.0
            {
                corrupted_neighbors.0 += 1;
            }
        }
    }
}

#[derive(Component)]
struct Phase(u8);

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct Cell;

#[derive(Component)]
struct SmilerColor;

#[derive(Component)]
struct Corrupted(bool);

#[derive(Component)]
struct CorruptedNeighbors(usize);

#[derive(Resource)]
struct SelectedEntity(Option<SelectionOptions>);

struct SelectionOptions {
    entity: Entity,
    sprite: Entity,
    phase: u8,
    coords: Vec3,
    corrupted: bool,
}

#[derive(Resource)]
struct CursorCoords(Option<Vec2>);

#[derive(Resource)]
struct AnimationIndices {
    normal_calm: Indices,
    normal_worried: Indices,
    normal_scared: Indices,
    corrupted_calm: Indices,
    corrupted_happy: Indices,
}

struct Indices {
    first: usize,
    last: usize,
}

#[derive(Component)]
enum SmilerState {
    NormalCalm,
    NormalWorried,
    NormalScared,
    CorruptedCalm,
    CorruptedHappy,
}

#[derive(Component)]
struct Smiler {
    phase: u8,
    corrupted: bool,
    corrupted_neighbors: usize,
    state: SmilerState,
}
