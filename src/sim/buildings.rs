use bevy::prelude::*;
use crate::sim::entities::{Building, BuildingKind, Position};
use crate::sim::resources::GameResources;
use crate::sim::grid::{WorldGrid, TileKind, TILE_SIZE};
use crate::ui::input::PaintTool;
use crate::sim::power_levels::{PowerGenerator, PowerConsumer};
use crate::sim::speed_modifiers::{SpeedModifiers, MovementSpeed, PowerLevelEffects};

#[derive(Resource, Default, Copy, Clone, Eq, PartialEq, Debug)]
pub enum BuildMode {
    #[default]
    None,
    ServerRack,
    PowerNode,
    Bot,
}

pub fn place_building_system(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    q_primary: Query<&Window, With<bevy::window::PrimaryWindow>>,
    q_cam: Query<(&Camera, &GlobalTransform), With<crate::ui::input::MainCamera>>,
    grid: Res<WorldGrid>,
    build_mode: Res<BuildMode>,
    mut resources: ResMut<GameResources>,
    core_query: Query<&crate::sim::entities::Position, With<crate::sim::entities::AICore>>,
) {
    if *build_mode == BuildMode::None {
        return;
    }

    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    if *build_mode == BuildMode::Bot {
        let cost = 50;
        if resources.spend_scrap(cost) {
            if let Ok(core_pos) = core_query.get_single() {
                commands.spawn((
                    crate::sim::entities::Bot::default(),
                    crate::sim::entities::Position {
                        x: core_pos.x,
                        y: (core_pos.y as i32 + 2) as u32,
                    },
                    PowerConsumer::new(2.0, 5.0, 0.5),
                    SpeedModifiers::default(),
                    MovementSpeed::new(1.0),
                    PowerLevelEffects::new(0.2),
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::srgb(0.9, 0.9, 0.2),
                            custom_size: Some(Vec2::new(TILE_SIZE * 0.6, TILE_SIZE * 0.6)),
                            ..default()
                        },
                        transform: Transform::from_xyz(
                            core_pos.x as f32 * TILE_SIZE,
                            (core_pos.y as i32 + 2) as f32 * TILE_SIZE,
                            2.0,
                        ),
                        ..default()
                    },
                ));
            }
        }
        return;
    }

    let (camera, cam_xform) = q_cam.single();

    let window = match camera.target {
        bevy::render::camera::RenderTarget::Window(id) => match id {
            bevy::window::WindowRef::Primary => q_primary.get_single().ok(),
            bevy::window::WindowRef::Entity(entity) => windows.get(entity).ok(),
        },
        _ => None,
    };
    let Some(window) = window else { return; };
    let Some(cursor) = window.cursor_position() else { return; };

    if let Some(ray) = camera.viewport_to_world(cam_xform, cursor) {
        let world = ray.origin.truncate();
        if world.x < 0.0 || world.y < 0.0 { return; }
        let gx = (world.x / TILE_SIZE).floor() as u32;
        let gy = (world.y / TILE_SIZE).floor() as u32;
        if gx >= grid.w || gy >= grid.h { return; }

        let cost = match *build_mode {
            BuildMode::ServerRack => 20,
            BuildMode::PowerNode => 15,
            BuildMode::Bot => return,
            BuildMode::None => return,
        };

        if resources.spend_scrap(cost) {
            let kind = match *build_mode {
                BuildMode::ServerRack => BuildingKind::ServerRack,
                BuildMode::PowerNode => BuildingKind::PowerNode,
                BuildMode::Bot => return,
                BuildMode::None => return,
            };

            let color = match kind {
                BuildingKind::ServerRack => Color::srgb(0.5, 0.2, 0.8),
                BuildingKind::PowerNode => Color::srgb(0.2, 0.8, 0.3),
            };

            let mut entity_commands = commands.spawn((
                Building::new(kind.clone()),
                Position { x: gx, y: gy },
                SpriteBundle {
                    sprite: Sprite {
                        color,
                        custom_size: Some(Vec2::new(TILE_SIZE * 0.8, TILE_SIZE * 0.8)),
                        ..default()
                    },
                    transform: Transform::from_xyz(
                        gx as f32 * TILE_SIZE,
                        gy as f32 * TILE_SIZE,
                        1.5,
                    ),
                    ..default()
                },
            ));

            // Add power components based on building type
            match kind {
                BuildingKind::ServerRack => {
                    entity_commands.insert(PowerConsumer::new(3.0, 3.0, 1.0));
                }
                BuildingKind::PowerNode => {
                    entity_commands.insert(PowerGenerator::new(5.0));
                }
            }
        }
    }
}

pub fn switch_build_mode(
    keys: Res<ButtonInput<KeyCode>>,
    mut build_mode: ResMut<BuildMode>,
) {
    if keys.just_pressed(KeyCode::Digit3) {
        *build_mode = BuildMode::ServerRack;
    }
    if keys.just_pressed(KeyCode::Digit4) {
        *build_mode = BuildMode::PowerNode;
    }
    if keys.just_pressed(KeyCode::Digit5) {
        *build_mode = BuildMode::Bot;
    }
    if keys.just_pressed(KeyCode::Escape) {
        *build_mode = BuildMode::None;
    }
}

pub fn complete_buildings(
    mut building_query: Query<(&mut Building, &Position)>,
    mut resources: ResMut<GameResources>,
    time: Res<Time>,
) {
    for (mut building, _pos) in building_query.iter_mut() {
        if building.is_complete {
            continue;
        }

        building.build_progress += time.delta_seconds() * 0.5;

        if building.build_progress >= 1.0 {
            building.is_complete = true;

            match building.kind {
                BuildingKind::ServerRack => {
                    resources.add_compute(10);
                    resources.add_power_consumption(3);
                }
                BuildingKind::PowerNode => {
                    resources.add_power_production(5);
                    resources.add_power_capacity(50);
                }
            }
        }
    }
}
