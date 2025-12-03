mod grid;
mod renderable;
mod spawn;
mod entities;
mod reservation;
mod pathfinding;
mod movement;

use bevy::prelude::*;
use reservation::ReservationSystem;

fn init() -> Result<(), String> {
    Ok(())
}

fn main() -> Result<(), String> {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Progress".to_string(),
                resolution: (640.0, 640.0).into(),
                ..default()
            }),
            ..default()
        }))
        .init_resource::<ReservationSystem>()
        .insert_resource(Time::<Fixed>::from_hz(10.0))
        .add_systems(Startup, (setup_camera, grid::setup_grid))
        .add_systems(Startup, spawn::spawn_initial_components.after(grid::setup_grid))
        .add_systems(Update, grid::add_new_positions_as_residents)
        .add_systems(Update, grid::update_residents)
        .add_systems(Update, renderable::spawn_sprites_for_new_renderables)
        .add_systems(Update, renderable::update_sprite_positions)
        .add_systems(FixedUpdate, entities::bot::find_bot_jobs)
        .add_systems(FixedUpdate, entities::bot::work)
        .add_systems(FixedUpdate, pathfinding::pathfind)
        .add_systems(FixedUpdate, movement::move_along_path)
        .run();

    Ok(())
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}