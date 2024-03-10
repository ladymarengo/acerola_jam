#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::{prelude::*, window::PrimaryWindow};
use rand::Rng;

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
        .add_systems(Startup, (spawn_camera, spawn_smilers))
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
            transform.translation.y -= 20.0;
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
                transform: Transform::from_xyz(x, y, 1.0).with_scale(Vec3::splat(0.5)),
                ..default()
            },
            Phase(0),
            Cell,
            Corrupted(corrupted),
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
    query: Query<&Transform, With<Cell>>,
) {
    let mut rng = rand::thread_rng();

    for x in (-240..=120).step_by(120) {
        if !query.iter().any(|transform| {
            transform.translation.x == x as f32 && transform.translation.y >= 120.0
        }) {
            let corrupted = rng.gen::<f64>() < 0.7;
            spawn_smiler(
                &mut commands,
                &asset_server,
                &mut texture_atlas_layouts,
                corrupted,
                x as f32,
                360.0,
            );
        }
    }
}

fn spawn_smilers(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for x in (-240..=120).step_by(120) {
        for y in (-240..=120).step_by(120) {
            spawn_smiler(
                &mut commands,
                &asset_server,
                &mut texture_atlas_layouts,
                false,
                x as f32,
                y as f32,
            );
        }
    }
}

fn mouse_motion(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    cursor_coords: Res<CursorCoords>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut selected: ResMut<SelectedEntity>,
    mut query: Query<
        (
            &mut Phase,
            &mut TextureAtlas,
            &Transform,
            Entity,
            &mut Corrupted,
            &Children,
        ),
        Without<SmilerColor>,
    >,
    texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut colors: Query<&mut TextureAtlas, With<SmilerColor>>,
) {
    if let Some(cursor_coords) = cursor_coords.0 {
        if mouse_button_input.just_pressed(MouseButton::Left) {
            for (mut phase, mut sprite, transform, entity, mut corrupted, children) in &mut query {
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
                            phase.0 += 1;

                            let mut rng = rand::thread_rng();
                            if (corrupted.0 || selection.corrupted)
                                && !(corrupted.0 && selection.corrupted)
                            {
                                corrupted.0 = rng.gen::<f64>() < 0.9;
                            }
                            if phase.0 < 6 {
                                let child = children.first().unwrap();
                                let mut color_sprite = colors.get_mut(*child).unwrap();
                                color_sprite.index += 1;
                            }
                            if corrupted.0 && sprite.index < 1 {
                                sprite.index += 1;
                            }
                            commands.entity(selection.entity).despawn_recursive();
                            commands.entity(selection.sprite).despawn();
                            selected.0 = None;
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
                            corrupted: corrupted.0,
                        });
                    }
                }
            }
        } else if mouse_button_input.just_pressed(MouseButton::Right) {
            for (_phase, _sprite, _transform, entity, _corrupted, _children) in &query {
                commands.entity(entity).despawn_recursive();
            }
            spawn_smilers(commands, asset_server, texture_atlas_layouts);
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
