use bevy::prelude::*;
use bevy::input::mouse::{MouseWheel, MouseScrollUnit};
use bevy::window::PrimaryWindow;
use crate::grid::{TILE_SIZE, TileKind};
use crate::world::World;

#[derive(Resource, Default, Debug, PartialEq, Eq)]
pub enum PaintTool {
    #[default]
    None,
    Scavenge,    // Key 1
    Stockpile,   // Key 2
}

#[derive(Resource, Default, Debug, PartialEq, Eq)]
pub enum BuildMode {
    #[default]
    None,
    ServerRack,  // Key 3
    PowerNode,   // Key 4
    Bot,         // Key 5
}

#[derive(Component)]
pub struct MainCamera;

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_xyz(512.0, 512.0, 0.0),
            ..default()
        },
        MainCamera,
    ));
}

pub fn camera_controls(
    mut camera: Query<&mut Transform, With<MainCamera>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut scroll_events: EventReader<MouseWheel>,
    time: Res<Time>,
) {
    let mut transform = camera.single_mut();

    // Pan with WASD
    let speed = 400.0 * time.delta_seconds();
    if keyboard.pressed(KeyCode::KeyW) {
        transform.translation.y += speed;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        transform.translation.y -= speed;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        transform.translation.x -= speed;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        transform.translation.x += speed;
    }

    // Zoom with mouse wheel
    for event in scroll_events.read() {
        let zoom_delta = match event.unit {
            MouseScrollUnit::Line => event.y * 0.1,
            MouseScrollUnit::Pixel => event.y * 0.01,
        };

        let current_scale = transform.scale.x;
        let new_scale = (current_scale - zoom_delta).clamp(0.5, 3.0);
        transform.scale = Vec3::splat(new_scale);
    }
}

pub fn handle_tool_switching(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut paint_tool: ResMut<PaintTool>,
    mut build_mode: ResMut<BuildMode>,
) {
    // Paint tools
    if keyboard.just_pressed(KeyCode::Digit1) {
        *paint_tool = PaintTool::Scavenge;
        *build_mode = BuildMode::None;
        info!("Selected Scavenge paint tool");
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        *paint_tool = PaintTool::Stockpile;
        *build_mode = BuildMode::None;
        info!("Selected Stockpile paint tool");
    }

    // Build modes
    if keyboard.just_pressed(KeyCode::Digit3) {
        *build_mode = BuildMode::ServerRack;
        *paint_tool = PaintTool::None;
        info!("Selected Server Rack build mode");
    }
    if keyboard.just_pressed(KeyCode::Digit4) {
        *build_mode = BuildMode::PowerNode;
        *paint_tool = PaintTool::None;
        info!("Selected Power Node build mode");
    }
    if keyboard.just_pressed(KeyCode::Digit5) {
        *build_mode = BuildMode::Bot;
        *paint_tool = PaintTool::None;
        info!("Selected Bot build mode");
    }

    // ESC to cancel
    if keyboard.just_pressed(KeyCode::Escape) {
        *paint_tool = PaintTool::None;
        *build_mode = BuildMode::None;
    }
}

pub fn handle_paint_input(
    mouse_button: Res<ButtonInput<MouseButton>>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    paint_tool: Res<PaintTool>,
    mut world: ResMut<World>,
    mut bots: Query<&mut crate::bot::Bot>,
) {
    if *paint_tool == PaintTool::None {
        return;
    }

    if !mouse_button.pressed(MouseButton::Left) {
        return;
    }

    let window = window.single();
    let (camera, camera_transform) = camera.single();

    if let Some(cursor_pos) = window.cursor_position() {
        if let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            // Convert world position to tile coordinates
            let tile_x = (world_pos.x / TILE_SIZE).floor() as i32;
            let tile_y = (world_pos.y / TILE_SIZE).floor() as i32;

            if tile_x >= 0 && tile_y >= 0 {
                let tile_x = tile_x as u32;
                let tile_y = tile_y as u32;

                if tile_x < world.grid.width && tile_y < world.grid.height {
                    let tile_kind = match *paint_tool {
                        PaintTool::Scavenge => TileKind::Scavenge,
                        PaintTool::Stockpile => TileKind::Stockpile,
                        PaintTool::None => return,
                    };

                    world.grid.set(tile_x, tile_y, tile_kind);

                    // Invalidate bot paths that cross this tile
                    for mut bot in bots.iter_mut() {
                        crate::bot::invalidate_bot_path_if_blocked(&mut bot, &world.grid);
                    }

                    // Rescan for jobs when painting scavenge zones
                    if *paint_tool == PaintTool::Scavenge {
                        world.scan_for_jobs();
                    }
                }
            }
        }
    }
}

pub fn handle_build_input(
    mouse_button: Res<ButtonInput<MouseButton>>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    build_mode: Res<BuildMode>,
    mut world: ResMut<World>,
    mut commands: Commands,
    mut bots: Query<&mut crate::bot::Bot>,
) {
    if *build_mode == BuildMode::None {
        return;
    }

    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    let window = window.single();
    let (camera, camera_transform) = camera.single();

    if let Some(cursor_pos) = window.cursor_position() {
        if let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            let tile_x = (world_pos.x / TILE_SIZE).floor() as i32;
            let tile_y = (world_pos.y / TILE_SIZE).floor() as i32;

            if tile_x >= 0 && tile_y >= 0 {
                let tile_x = tile_x as u32;
                let tile_y = tile_y as u32;

                if tile_x < world.grid.width && tile_y < world.grid.height {
                    match *build_mode {
                        BuildMode::ServerRack => {
                            if world.spend_scrap(50) {
                                world.grid.set(tile_x, tile_y, TileKind::ServerRack);
                                world.compute_capacity += 10;
                                world.power.add_consumption(5.0);

                                // Invalidate bot paths that cross this tile
                                for mut bot in bots.iter_mut() {
                                    crate::bot::invalidate_bot_path_if_blocked(&mut bot, &world.grid);
                                }

                                info!("Built Server Rack at ({}, {})", tile_x, tile_y);
                            } else {
                                warn!("Not enough scrap to build Server Rack (need 50)");
                            }
                        }
                        BuildMode::PowerNode => {
                            if world.spend_scrap(30) {
                                world.grid.set(tile_x, tile_y, TileKind::PowerNode);
                                world.power.add_generation(15.0);

                                // Invalidate bot paths that cross this tile
                                for mut bot in bots.iter_mut() {
                                    crate::bot::invalidate_bot_path_if_blocked(&mut bot, &world.grid);
                                }

                                info!("Built Power Node at ({}, {})", tile_x, tile_y);
                            } else {
                                warn!("Not enough scrap to build Power Node (need 30)");
                            }
                        }
                        BuildMode::Bot => {
                            if world.spend_scrap(50) {
                                let position = (tile_x, tile_y);
                                commands.spawn(crate::bot::Bot::new(position));
                                info!("Built Bot at ({}, {})", tile_x, tile_y);
                            } else {
                                warn!("Not enough scrap to build Bot (need 50)");
                            }
                        }
                        BuildMode::None => {}
                    }
                }
            }
        }
    }
}
