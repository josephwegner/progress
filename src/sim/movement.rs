use std::cmp::max;
use bevy::prelude::*;
use crate::sim::entities::{Position, Path};
use crate::sim::grid::WorldGrid;
use crate::sim::game_state::WorldChangeTracker;
use crate::sim::pathfinding::find_path;

// ============================================================================
// COMPONENTS
// ============================================================================

/// Generic component for any entity that can pathfind
#[derive(Component, Debug)]
pub struct Pathfinder {}

impl Pathfinder {
    pub fn new() -> Self {
        Self {}
    }
}

// ============================================================================
// SYSTEMS
// ============================================================================
/// System: Remove Path component from entities whose paths cross changed tiles
/// Run in FixedUpdate, after track_world_changes
pub fn invalidate_affected_paths(
    mut commands: Commands,
    tracker: Res<WorldChangeTracker>,
    path_query: Query<(Entity, &Path)>,
) {
    if tracker.tiles_changed.is_empty() {
        return;
    }

    // Invalidate active paths that cross changed tiles
    for (entity, path) in path_query.iter() {
        for node in path.nodes.iter() {
            if tracker.tiles_changed.contains(&(node.x, node.y)) {
                commands.entity(entity).remove::<Path>();
                break;
            }
        }
    }
}
