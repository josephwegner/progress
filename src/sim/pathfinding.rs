use bevy::prelude::*;
use pathfinding::prelude::astar;
use crate::sim::grid::{WorldGrid, TileKind};
use crate::sim::entities::{Position, Path};
use crate::sim::power_levels::{PowerConsumer, PowerLevel};
use crate::sim::speed_modifiers::MovementSpeed;

/// Base time interval (in seconds) for moving 1 tile at speed 1.0
/// Lower values = faster movement. 0.002 = ~500 tiles/sec at base speed.
pub const BASE_MOVEMENT_INTERVAL: f32 = 0.005;

pub fn move_entities_along_path(
    mut commands: Commands,
    mut entity_query: Query<(Entity, &mut Position, &mut Path, &MovementSpeed, Option<&PowerConsumer>)>,
    time: Res<Time>,
    debug: Res<crate::sim::debug::DebugSettings>,
) {
    for (entity, mut pos, mut path, movement_speed, power_consumer) in entity_query.iter_mut() {
        // Don't move shutdown entities
        if let Some(pc) = power_consumer {
            if pc.power_level == PowerLevel::Shutdown {
                continue;
            }
        }

        // Handle path completion
        if path.current_idx >= path.nodes.len() {
            if debug.log_pathfinding {
                info!("Bot at ({},{}) reached end of path", pos.x, pos.y);
            }
            commands.entity(entity).remove::<Path>();
            continue;
        }

        // Don't move until cooldown is reached
        path.movement_cooldown -= time.delta_seconds();
        if path.movement_cooldown > 0.0 {
            continue;
        }

        // Ok, let's move!
        path.movement_cooldown = BASE_MOVEMENT_INTERVAL / movement_speed.current_speed;

        let target = &path.nodes[path.current_idx];

        if pos.x == target.x && pos.y == target.y {
            path.current_idx += 1;
            if path.current_idx >= path.nodes.len() {
                if debug.log_pathfinding {
                    info!("Bot at ({},{}) completed path", pos.x, pos.y);
                }
                commands.entity(entity).remove::<Path>();
            }
        } else {
            if debug.log_pathfinding {
                info!("Bot moving from ({},{}) to ({},{}), step {}/{}",
                      pos.x, pos.y, target.x, target.y, path.current_idx + 1, path.nodes.len());
            }
            pos.x = target.x;
            pos.y = target.y;
        }
    }
}

pub fn find_path(
    start: Position,
    goal: Position,
    grid: &WorldGrid,
) -> Option<Vec<Position>> {
    // Check if goal tile is walkable
    let goal_tile = grid.tiles[grid.idx(goal.x, goal.y)];
    let goal_is_walkable = goal_tile == TileKind::Ground || goal_tile == TileKind::Stockpile;

    let result = astar(
        &start,
        |pos| {
            let mut neighbors = Vec::new();
            let directions = [(0i32, 1i32), (1, 0), (0, -1), (-1, 0)];

            for (dx, dy) in directions.iter() {
                let nx = pos.x as i32 + dx;
                let ny = pos.y as i32 + dy;

                if nx >= 0 && ny >= 0 && nx < grid.w as i32 && ny < grid.h as i32 {
                    let tile = grid.tiles[grid.idx(nx as u32, ny as u32)];
                    // Only allow movement through Ground and Stockpile tiles
                    if tile == TileKind::Ground || tile == TileKind::Stockpile {
                        neighbors.push((Position { x: nx as u32, y: ny as u32 }, 1u32));
                    }
                }
            }
            neighbors
        },
        |pos| {
            ((pos.x as i32 - goal.x as i32).abs() + (pos.y as i32 - goal.y as i32).abs()) as u32
        },
        |pos| {
            if goal_is_walkable {
                // Goal is walkable: must reach exactly
                *pos == goal
            } else {
                // Goal is non-walkable (e.g., resource): succeed when adjacent
                let dx = (pos.x as i32 - goal.x as i32).abs();
                let dy = (pos.y as i32 - goal.y as i32).abs();
                dx + dy == 1
            }
        },
    );

    result.map(|(path, _cost)| path)
}
