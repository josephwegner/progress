mod grid;
mod world;
mod bot;
mod pathfinding;
mod rendering;
mod input;
mod power;
mod jobs;
mod ui;
mod scouts;

use bevy::prelude::*;
use bevy::asset::AssetMetaCheck;

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1)))
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            meta_check: AssetMetaCheck::Never,
            ..default()
        }).set(WindowPlugin {
            primary_window: Some(Window {
                title: "Machine Seed".into(),
                fit_canvas_to_parent: true,
                canvas: Some("#bevy".into()),
                ..default()
            }),
            ..default()
        }))
        // Resources
        .init_resource::<world::World>()
        .init_resource::<input::PaintTool>()
        .init_resource::<input::BuildMode>()
        .init_resource::<scouts::ScoutSpawnTimer>()
        // Startup
        .add_systems(Startup, (
            input::setup_camera,
            world::setup_world,
            rendering::setup_rendering,
            ui::setup_hud,
        ))
        // Fixed update (game logic at fixed timestep)
        .add_systems(FixedUpdate, (
            power::update_power_system,
            scouts::update_scouts,
            scouts::scout_jamming_system,
            bot::update_bots,
            scouts::scout_detection_system,
            world::update_game_time,
        ).chain())
        // Update (render and input at frame rate)
        .add_systems(Update, (
            input::camera_controls,
            input::handle_paint_input,
            input::handle_build_input,
            input::handle_tool_switching,
            scouts::spawn_scouts_system,
            rendering::update_tile_rendering,
            rendering::update_bot_sprites,
            ui::update_hud,
            ui::show_game_over_screen,
        ))
        .run();
}
