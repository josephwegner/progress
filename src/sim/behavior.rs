use bevy::prelude::*;
use crate::sim::entities::{Bot, Position, Path, BotState, AICore};
use crate::sim::jobs::{JobQueue, JobType};
use crate::sim::resources::GameResources;
use crate::sim::grid::{WorldGrid, TileKind};
use crate::sim::pathfinding::find_path;

pub fn bot_work_system(
    mut commands: Commands,
    mut bot_query: Query<(Entity, &mut Bot, &Position), Without<Path>>,
    mut job_queue: ResMut<JobQueue>,
    mut resources: ResMut<GameResources>,
    mut grid: ResMut<WorldGrid>,
    core_query: Query<&Position, With<AICore>>,
    debug: Res<crate::sim::debug::DebugSettings>,
) {
    for (bot_entity, mut bot, bot_pos) in bot_query.iter_mut() {
        // Handle bots that are in Hauling state (they don't have a Path component)
        if bot.state == BotState::Hauling {
            if let Ok(core_pos) = core_query.get_single() {
                if bot_pos.x == core_pos.x && bot_pos.y == core_pos.y {
                    // Bot has reached the AI Core - deliver scrap
                    if debug.log_bot_behavior {
                        info!("Bot at ({},{}) delivering {} scrap to AI Core", bot_pos.x, bot_pos.y, bot.carry_scrap);
                    }
                    resources.add_scrap(bot.carry_scrap);
                    bot.carry_scrap = 0;
                    bot.current_job = None; // Clear job reference
                    bot.state = BotState::Idle;
                    if debug.log_bot_behavior {
                        info!("Haul complete, bot now idle");
                    }
                } else {
                    // Bot is hauling but not at core yet - try to find/retry path
                    if debug.log_bot_behavior {
                        info!("Hauling bot at ({},{}) has no path, trying to find path to AI Core at ({},{})",
                              bot_pos.x, bot_pos.y, core_pos.x, core_pos.y);
                    }

                    if let Some(path_nodes) = find_path(
                        bot_pos.clone(),
                        core_pos.clone(),
                        &grid,
                    ) {
                        if debug.log_pathfinding {
                            info!("Path found for hauling bot: {} steps", path_nodes.len());
                        }
                        commands.entity(bot_entity).insert(Path {
                            nodes: path_nodes,
                            current_idx: 0,
                            movement_cooldown: 0.0,
                        });
                    } else {
                        if debug.log_bot_behavior {
                            warn!("Hauling bot at ({},{}) still can't find path to AI Core, will retry next frame",
                                  bot_pos.x, bot_pos.y);
                        }
                    }
                }
            }
            // Skip further processing for hauling bots
            continue;
        }

        if bot.current_job.is_none() {
            continue;
        }

        let job_entity = bot.current_job.unwrap();
        if debug.log_bot_behavior {
            info!("bot_work_system processing bot at ({},{}) with job {:?}", bot_pos.x, bot_pos.y, job_entity);
        }

        let mut job_to_process = None;
        for job in job_queue.queue.iter() {
            if job.entity == job_entity {
                job_to_process = Some(job.clone());
                break;
            }
        }

        if job_to_process.is_none() {
            warn!("Bot at ({},{}) has job entity {:?} but job not found in queue! Clearing job and returning to Idle.",
                  bot_pos.x, bot_pos.y, job_entity);
            bot.current_job = None;
            bot.state = BotState::Idle;
            continue;
        }

        if let Some(job) = job_to_process {
            match &job.job_type {
                JobType::Scavenge { x, y } => {
                    // Bot can scavenge from adjacent tile OR from on top of it
                    let dx = (bot_pos.x as i32 - *x as i32).abs();
                    let dy = (bot_pos.y as i32 - *y as i32).abs();
                    let is_adjacent_or_on = dx + dy <= 1;

                    if is_adjacent_or_on {
                        if debug.log_bot_behavior {
                            info!("Bot at ({},{}) collecting scrap from scavenge tile", bot_pos.x, bot_pos.y);
                        }
                        let scrap_amount = 5;
                        bot.carry_scrap = scrap_amount;
                        bot.state = BotState::Hauling;

                        job_queue.remove_job(job_entity);
                        commands.entity(job_entity).despawn();
                        bot.current_job = None;

                        let idx = grid.idx(*x, *y);
                        grid.tiles[idx] = TileKind::Ground;
                        grid.mark_chunk_dirty(*x, *y);
                        if debug.log_bot_behavior {
                            info!("Scavenge tile at ({},{}) converted to Ground", x, y);
                        }

                        match core_query.get_single() {
                            Ok(core_pos) => {
                            if debug.log_bot_behavior {
                                info!("Creating haul job from ({},{}) to AI Core at ({},{})",
                                      bot_pos.x, bot_pos.y, core_pos.x, core_pos.y);
                            }

                            bot.current_job = None;
                            bot.state = BotState::Hauling;

                            if let Some(path_nodes) = find_path(
                                bot_pos.clone(),
                                core_pos.clone(),
                                &grid,
                            ) {
                                if debug.log_pathfinding {
                                    info!("Path found for haul job, {} steps", path_nodes.len());
                                }
                                commands.entity(bot_entity).insert(Path {
                                    nodes: path_nodes,
                                    current_idx: 0,
                                    movement_cooldown: 0.0,
                                });
                            } else {
                                warn!("Failed to find path for haul job!");
                            }
                            },
                            Err(e) => {
                                warn!("Failed to get AI Core position: {:?}", e);
                            }
                        }
                    }
                }
            }
        }
    }
}
