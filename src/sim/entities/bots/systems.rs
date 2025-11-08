use bevy::prelude::*;
use crate::sim::entities::{Position, Path};
use crate::sim::entities::bots::{Bot, HasJob, CarryingScrap};
use crate::sim::entities::ai_core::AICore;
use crate::sim::power_levels::PowerConsumer;
use crate::sim::speed_modifiers::{SpeedModifiers, MovementSpeed, PowerLevelEffects};
use crate::sim::movement::Pathfinder;
use crate::sim::zones::InZone;
use crate::sim::resources::GameResources;
use crate::sim::grid::{WorldGrid, TileKind};

/// Spawn initial bots at game start
pub fn spawn_initial_bots(
    mut commands: Commands,
    core_query: Query<&Position, With<AICore>>,
) {
    if let Ok(core_pos) = core_query.get_single() {
        for i in 0..2 {
            let offset = if i == 0 { 2 } else { -2 };
            commands.spawn((
                Bot::default(),
                Pathfinder::new(),
                Position {
                    x: (core_pos.x as i32 + offset) as u32,
                    y: core_pos.y
                },
                InZone::default(),
                PowerConsumer::new(2.0, 5.0, 0.5),
                SpeedModifiers::default(),
                MovementSpeed::new(1.0),
                PowerLevelEffects::new(0.2),
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(0.9, 0.9, 0.2),
                        custom_size: Some(Vec2::new(
                            crate::sim::grid::TILE_SIZE * 0.6,
                            crate::sim::grid::TILE_SIZE * 0.6
                        )),
                        ..default()
                    },
                    transform: Transform::from_xyz(
                        ((core_pos.x as i32 + offset) as f32) * crate::sim::grid::TILE_SIZE,
                        core_pos.y as f32 * crate::sim::grid::TILE_SIZE,
                        2.0,
                    ),
                    ..default()
                },
            ));
        }
    }
}
