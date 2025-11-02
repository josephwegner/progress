use bevy::prelude::*;
use pathfinding::prelude::astar;
use crate::sim::grid::{WorldGrid, TileKind};
use crate::sim::entities::{Bot, Position, Path};

pub fn assign_jobs_to_bots(
    mut commands: Commands,
    mut bot_query: Query<(Entity, &mut Bot, &Position), Without<Path>>,
    mut job_queue: ResMut<crate::sim::jobs::JobQueue>,
    grid: Res<WorldGrid>,
    debug: Res<crate::sim::debug::DebugSettings>,
) {
    let idle_bot_count = bot_query.iter().filter(|(_, bot, _)| bot.state == crate::sim::entities::BotState::Idle).count();
    let jobs_available = job_queue.queue.len();

    if debug.log_jobs && idle_bot_count > 0 && jobs_available > 0 {
        info!("assign_jobs_to_bots: {} idle bots, {} jobs available", idle_bot_count, jobs_available);
    }

    for (bot_entity, mut bot, bot_pos) in bot_query.iter_mut() {
        if bot.state != crate::sim::entities::BotState::Idle {
            continue;
        }

        if debug.log_jobs {
            info!("Idle bot at ({},{}) looking for job", bot_pos.x, bot_pos.y);
        }

        if let Some(job) = job_queue.pop() {
            let target_pos = match &job.job_type {
                crate::sim::jobs::JobType::Scavenge { x, y } => {
                    if debug.log_jobs {
                        info!("Assigning Scavenge job at ({},{}) to bot at ({},{})", x, y, bot_pos.x, bot_pos.y);
                    }
                    (*x, *y)
                },
            };

            if let Some(path_nodes) = find_path(
                (bot_pos.x, bot_pos.y),
                target_pos,
                &grid,
            ) {
                if debug.log_pathfinding {
                    info!("Path found: {} steps", path_nodes.len());
                }
                bot.current_job = Some(job.entity);
                job_queue.queue.push(job);
                commands.entity(bot_entity).insert(Path {
                    nodes: path_nodes,
                    current_idx: 0,
                });
            } else {
                warn!("No path found from ({},{}) to ({},{}), requeueing job",
                      bot_pos.x, bot_pos.y, target_pos.0, target_pos.1);
                job_queue.queue.push(job);
            }
        }
    }
}

pub fn move_bots_along_path(
    mut commands: Commands,
    mut bot_query: Query<(Entity, &mut Position, &mut Path), With<Bot>>,
    debug: Res<crate::sim::debug::DebugSettings>,
) {
    for (entity, mut pos, mut path) in bot_query.iter_mut() {
        if path.current_idx >= path.nodes.len() {
            if debug.log_pathfinding {
                info!("Bot at ({},{}) reached end of path", pos.x, pos.y);
            }
            commands.entity(entity).remove::<Path>();
            continue;
        }

        let target = path.nodes[path.current_idx];

        if pos.x == target.0 && pos.y == target.1 {
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
                      pos.x, pos.y, target.0, target.1, path.current_idx + 1, path.nodes.len());
            }
            pos.x = target.0;
            pos.y = target.1;
        }
    }
}

pub fn find_path(
    start: (u32, u32),
    goal: (u32, u32),
    grid: &WorldGrid,
) -> Option<Vec<(u32, u32)>> {
    let result = astar(
        &start,
        |&(x, y)| {
            let mut neighbors = Vec::new();
            let directions = [(0i32, 1i32), (1, 0), (0, -1), (-1, 0)];

            for (dx, dy) in directions.iter() {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;

                if nx >= 0 && ny >= 0 && nx < grid.w as i32 && ny < grid.h as i32 {
                    neighbors.push(((nx as u32, ny as u32), 1u32));
                }
            }
            neighbors
        },
        |&(x, y)| {
            ((x as i32 - goal.0 as i32).abs() + (y as i32 - goal.1 as i32).abs()) as u32
        },
        |&pos| pos == goal,
    );

    result.map(|(path, _cost)| path)
}
