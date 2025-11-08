use bevy::prelude::*;
use serde::{Serialize, Deserialize};

pub mod bots;
pub mod ai_core;
pub mod scouts;

// Re-export for convenience
pub use bots::*;
pub use ai_core::*;
pub use scouts::*;

// ============================================================================
// GENERIC COMPONENTS (used by multiple entity types)
// ============================================================================

#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

#[derive(Component, Clone, Debug)]
pub struct Path {
    pub nodes: Vec<Position>,
    pub current_idx: usize,
    /// Cooldown timer in seconds until next movement. When <= 0, entity moves to next node.
    /// Reset to 1.0 / current_speed after each movement.
    pub movement_cooldown: f32,
}

// ============================================================================
// BUILDING COMPONENTS (not entity-specific, used by buildings system)
// ============================================================================

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub enum BuildingKind {
    ServerRack,
    PowerNode,
}

#[derive(Component, Clone, Debug)]
pub struct Building {
    pub kind: BuildingKind,
    pub build_progress: f32,
    pub is_complete: bool,
}

impl Building {
    pub fn new(kind: BuildingKind) -> Self {
        Self {
            kind,
            build_progress: 0.0,
            is_complete: false,
        }
    }
}

#[derive(Component)]
pub struct BuildProgressBar {
    pub building_entity: Entity,
}

// ============================================================================
// GENERIC SYSTEMS
// ============================================================================

/// Update sprite positions for all entities with Position + Transform
pub fn update_sprite_positions(
    mut query: Query<(&Position, &mut Transform), Or<(With<bots::Bot>, With<ai_core::AICore>, With<scouts::Scout>)>>,
) {
    for (pos, mut transform) in query.iter_mut() {
        transform.translation.x = pos.x as f32 * crate::sim::grid::TILE_SIZE;
        transform.translation.y = pos.y as f32 * crate::sim::grid::TILE_SIZE;
    }
}
