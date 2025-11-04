use bevy::prelude::*;
use pathfinding::prelude::astar;
use crate::sim::grid::{WorldGrid, TileKind};
use crate::sim::entities::{Bot, Position, Path};
use crate::sim::power_levels::{PowerConsumer, PowerLevel};
use crate::sim::speed_modifiers::MovementSpeed;

/// Base time interval (in seconds) for moving 1 tile at speed 1.0
/// Lower values = faster movement. 0.002 = ~500 tiles/sec at base speed.
pub const BASE_MOVEMENT_INTERVAL: f32 = 0.005;

pub fn assign_jobs_to_bots(
    mut commands: Commands,
    mut bot_query: Query<(Entity, &mut Bot, &Position, &PowerConsumer), Without<Path>>,
    mut job_queue: ResMut<crate::sim::jobs::JobQueue>,
    grid: Res<WorldGrid>,
    debug: Res<crate::sim::debug::DebugSettings>,
) {
    let idle_bot_count = bot_query.iter().filter(|(_, bot, _, _)| bot.state == crate::sim::entities::BotState::Idle).count();
    let jobs_available = job_queue.queue.len();

    if debug.log_jobs && idle_bot_count > 0 && jobs_available > 0 {
        info!("assign_jobs_to_bots: {} idle bots, {} jobs available", idle_bot_count, jobs_available);
    }

    for (bot_entity, mut bot, bot_pos, power_consumer) in bot_query.iter_mut() {
        if bot.state != crate::sim::entities::BotState::Idle {
            continue;
        }

        if power_consumer.power_level == PowerLevel::Shutdown {
            continue;
        }

        if debug.log_jobs {
            info!("Idle bot at ({},{}) looking for job", bot_pos.x, bot_pos.y);
        }

        // Try up to 5 jobs to find one that's reachable
        let mut jobs_tried = 0;
        let max_attempts = 5;
        let mut found_job = false;

        while jobs_tried < max_attempts && !found_job {
            if let Some(job) = job_queue.pop() {
                jobs_tried += 1;

                let target_pos = match &job.job_type {
                    crate::sim::jobs::JobType::Scavenge { x, y } => {
                        if debug.log_jobs {
                            info!("Assigning Scavenge job at ({},{}) to bot at ({},{})", x, y, bot_pos.x, bot_pos.y);
                        }
                        Position { x: *x, y: *y }
                    },
                };

                // Check if bot is already adjacent to target (for non-walkable tiles like Scavenge)
                let dx = (bot_pos.x as i32 - target_pos.x as i32).abs();
                let dy = (bot_pos.y as i32 - target_pos.y as i32).abs();
                let is_adjacent_or_on = dx + dy <= 1; // Adjacent (=1) or on target (=0)
                let is_on_target = bot_pos.x == target_pos.x && bot_pos.y == target_pos.y;

                // Check if target is walkable
                let target_tile = grid.tiles[grid.idx(target_pos.x, target_pos.y)];
                let target_is_walkable = target_tile == TileKind::Ground || target_tile == TileKind::Stockpile;

                // If bot is already in position, assign job without pathfinding
                // For non-walkable tiles (Scavenge): bot can be adjacent OR on top
                // For walkable tiles: bot must be on target
                if (is_adjacent_or_on && !target_is_walkable) || (is_on_target && target_is_walkable) {
                    if debug.log_jobs {
                        info!("Bot already in position for job (distance: {}), assigning directly without pathfinding", dx + dy);
                    }
                    bot.current_job = Some(job.entity);
                    bot.state = crate::sim::entities::BotState::MovingToJob;
                    job_queue.queue.push(job);
                    found_job = true;
                    break;
                }

                if let Some(path_nodes) = find_path(
                    bot_pos.clone(),
                    target_pos.clone(),
                    &grid,
                ) {
                    if debug.log_pathfinding {
                        info!("Path found: {} steps", path_nodes.len());
                    }
                    bot.current_job = Some(job.entity);
                    bot.state = crate::sim::entities::BotState::MovingToJob;
                    job_queue.queue.push(job);
                    commands.entity(bot_entity).insert(Path {
                        nodes: path_nodes,
                        current_idx: 0,
                        movement_cooldown: 0.0 // Start moving immediately. MovementSystem will set this to 1.0 / current_speed
                    });
                    found_job = true;
                    break;
                } else {
                    if debug.log_jobs {
                        warn!("No path found from ({},{}) to ({},{}), trying next job (attempt {}/{})",
                              bot_pos.x, bot_pos.y, target_pos.x, target_pos.y, jobs_tried, max_attempts);
                    }
                    // Put job at the back of the queue and try the next one
                    job_queue.queue.push(job);
                }
            } else {
                // No more jobs available
                break;
            }
        }
    }
}

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
