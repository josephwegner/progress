#[allow(unused_imports)]
mod sim {
    pub mod grid;
    pub mod resources;
    pub mod entities;
    pub mod jobs;
    pub mod pathfinding;
    pub mod buildings;
    pub mod debug;
    pub mod combat;
    pub mod conditions;
    pub mod power_levels;
    pub mod speed_modifiers;
    pub mod notifications;
    pub mod game_state;
    pub mod movement;
    pub mod zones;
}
mod render { pub mod atlas; pub mod chunks; }
mod ui { pub mod input; pub mod hud; }

use bevy::prelude::*;
use bevy::asset::AssetMetaCheck;
use sim::grid::{WorldGrid, MAP_W, MAP_H};
use sim::resources::{GameResources, tick_resources};
use sim::entities::{spawn_ai_core, spawn_initial_bots, update_sprite_positions};
use sim::notifications::{NotificationHistory, setup_notification_ui, tick_notification_timers, update_notification_display};
use sim::game_state::{
    WorldChangeTracker, GameTime, GameState, PowerCollapseTimer, 
    check_win_condition, check_power_collapse, display_game_over,
    track_world_changes,
};
use sim::jobs::{assign_jobs_to_idle_entities, find_path_to_job, execute_scavenge_jobs, deliver_resources_to_core, cleanup_orphaned_jobs};
use sim::zones::{ReachabilityZones, ZonedJobQueue, build_initial_zones, scan_initial_jobs, rebuild_reachability_zones, scan_for_jobs_in_zones, update_entity_zones};
use sim::pathfinding::move_entities_along_path;
use sim::buildings::{BuildMode, place_building_system, switch_build_mode, complete_buildings, update_progress_bars, despawn_completed_progress_bars};
use sim::debug::DebugSettings;
use sim::entities::{ScoutSpawnTimer, spawn_scouts_system, scout_movement_system, scout_detection_system};
use ui::input::{setup_camera, camera_controls, paint_brush, switch_tools, PaintTool};
use ui::hud::{setup_hud, update_hud};
use render::chunks::{setup_render, spawn_chunk_entities, rebuild_dirty_chunks};
use sim::combat::combat_system;
use sim::conditions::{condition_system, apply_condition_effects_system};
use sim::power_levels::power_management_system;
use sim::speed_modifiers::{update_power_level_speed_modifiers, calculate_movement_speed};
use sim::movement::invalidate_affected_paths;

#[cfg(target_arch = "wasm32")]
use console_error_panic_hook;

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = dotenvy::dotenv();
    }

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
            .set(WindowPlugin{
                primary_window: Some(Window{
                    title: "Machine Seed".into(),
                    fit_canvas_to_parent: true,
                    canvas: Some("#bevy".into()),
                    ..default()
                }),
                ..default()
            })
        )
        .insert_resource(WorldGrid::new(MAP_W, MAP_H, 32, 32))
        .insert_resource(PaintTool::default())
        .insert_resource(GameResources::default())
        .insert_resource(GameTime::default())
        .insert_resource(GameState::default())
        .insert_resource(PowerCollapseTimer::new())
        .insert_resource(NotificationHistory::default())
        .insert_resource(ReachabilityZones::default())
        .insert_resource(ZonedJobQueue::default())
        .insert_resource(BuildMode::default())
        .insert_resource(DebugSettings::from_env())
        .insert_resource(ScoutSpawnTimer::default())
        .insert_resource(WorldChangeTracker::default())
        .add_systems(Startup, (
            setup_camera,
            setup_render,
            setup_hud,
            setup_notification_ui,
            spawn_ai_core,
        ))
        .add_systems(Startup, spawn_chunk_entities.after(setup_render))
        .add_systems(Startup, spawn_initial_bots.after(spawn_ai_core))
        .add_systems(Startup, build_initial_zones.after(spawn_chunk_entities))
        .add_systems(Startup, scan_initial_jobs.after(build_initial_zones))
        .add_systems(FixedUpdate, (
            power_management_system,
            tick_resources,
            combat_system,
            condition_system,
            apply_condition_effects_system,
            update_power_level_speed_modifiers,
            calculate_movement_speed,
            rebuild_reachability_zones,
            scan_for_jobs_in_zones,
            update_entity_zones,
            invalidate_affected_paths,
            move_entities_along_path,
        ))
        .add_systems(FixedUpdate, (
            assign_jobs_to_idle_entities,
            find_path_to_job,
            execute_scavenge_jobs,
            deliver_resources_to_core,
            cleanup_orphaned_jobs,
            complete_buildings,
        ).chain())
        .add_systems(Update, (
            paint_brush,
            place_building_system,
        ))
        .add_systems(Update, track_world_changes.after(paint_brush).after(place_building_system))
        .add_systems(Update, rebuild_dirty_chunks.after(track_world_changes))
        .add_systems(Update, (
            camera_controls,
            switch_tools,
            switch_build_mode,
            update_sprite_positions,
            spawn_scouts_system,
            scout_movement_system,
            scout_detection_system,
            tick_notification_timers,
            update_notification_display,
            check_win_condition,
            check_power_collapse,
            display_game_over,
            update_hud,
            update_progress_bars,
            despawn_completed_progress_bars,
            apply_condition_effects_system,
        ))
        .run();
}
