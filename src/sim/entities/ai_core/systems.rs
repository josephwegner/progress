use bevy::prelude::*;
use crate::sim::entities::Position;
use crate::sim::entities::ai_core::AICore;
use crate::sim::power_levels::PowerGenerator;

/// Spawn the AI Core at game start
pub fn spawn_ai_core(
    mut commands: Commands,
    grid: Res<crate::sim::grid::WorldGrid>,
) {
    let core_x = grid.w / 2;
    let core_y = grid.h / 2;
    let world_x = core_x as f32 * crate::sim::grid::TILE_SIZE;
    let world_y = core_y as f32 * crate::sim::grid::TILE_SIZE;

    commands.spawn((
        AICore,
        Position { x: core_x, y: core_y },
        PowerGenerator::new(11.0),
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.2, 0.8, 1.0),
                custom_size: Some(Vec2::new(crate::sim::grid::TILE_SIZE, crate::sim::grid::TILE_SIZE)),
                ..default()
            },
            transform: Transform::from_xyz(world_x, world_y, 1.0),
            ..default()
        },
    ));
}
