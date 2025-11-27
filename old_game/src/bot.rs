use bevy::prelude::*;
use crate::pathfinding::{Position, find_path};
use crate::world::World;
use crate::grid::TILE_SIZE;

const BASE_MOVE_TIME: f32 = 0.075;  // Time to move one tile at normal speed (2x faster)
const HARVEST_TIME: f32 = 2.0;     // Time to harvest a resource

#[derive(Debug, Clone)]
pub enum BotState {
    Idle,
    MovingToJob {
        job_id: u32,
        path_index: usize,
    },
    Harvesting {
        job_id: u32,
        progress: f32,
    },
    ReturningToCore {
        scrap: u32,
        path_index: usize,
    },
}

#[derive(Component, Debug)]
pub struct Bot {
    pub state: BotState,
    pub position: Position,
    pub path: Vec<Position>,
    pub movement_cooldown: f32,
    pub jammed: bool,
}

impl Bot {
    pub fn new(position: Position) -> (Self, SpriteBundle) {
        let bot = Self {
            state: BotState::Idle,
            position,
            path: Vec::new(),
            movement_cooldown: 0.0,
            jammed: false,
        };

        let sprite = SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.9, 0.9, 0.2),
                custom_size: Some(Vec2::new(TILE_SIZE * 0.6, TILE_SIZE * 0.6)),
                ..default()
            },
            transform: Transform::from_xyz(
                position.0 as f32 * TILE_SIZE,
                position.1 as f32 * TILE_SIZE,
                2.0,
            ),
            ..default()
        };

        (bot, sprite)
    }

    pub fn is_active(&self) -> bool {
        !matches!(self.state, BotState::Idle)
    }

    fn get_speed_multiplier(&self, world: &World) -> f32 {
        let power_mult = world.power.get_speed_multiplier();
        let jammed_mult = if self.jammed { 0.2 } else { 1.0 };
        power_mult * jammed_mult
    }
}

// Helper function to invalidate a single bot's path if blocked
pub fn invalidate_bot_path_if_blocked(bot: &mut Bot, grid: &crate::grid::Grid) {
    if bot.path.is_empty() {
        return;
    }

    // Check if any tile in the path is now non-walkable
    let path_blocked = bot.path.iter().any(|&pos| !grid.is_walkable(pos.0, pos.1));

    if path_blocked {
        info!("Bot path blocked by tile change, replanning");

        // Clear the path - bot will replan on next update
        bot.path.clear();

        // Reset state to idle to force replanning
        match bot.state {
            BotState::MovingToJob { .. } => {
                bot.state = BotState::Idle;
            }
            BotState::ReturningToCore { .. } => {
                bot.state = BotState::Idle;
            }
            _ => {}
        }
    }
}

pub fn update_bots(
    mut bots: Query<(Entity, &mut Bot)>,
    mut world: ResMut<World>,
    time: Res<Time>,
) {
    if world.game_over.is_some() {
        return;  // Game is over, bots stop working
    }

    let dt = time.delta_seconds();

    for (entity, mut bot) in bots.iter_mut() {
        // Update movement cooldown
        if bot.movement_cooldown > 0.0 {
            bot.movement_cooldown -= dt;
            continue;
        }

        // Handle current state
        match bot.state.clone() {
            BotState::Idle => {
                // Look for work
                if let Some(job_id) = world.claim_nearest_job(bot.position) {
                    let job_position = world.get_job(job_id).unwrap().position;

                    if let Some(path) = find_path(bot.position, job_position, &world.grid) {
                        world.claim_job(job_id, entity);
                        bot.path = path;
                        bot.state = BotState::MovingToJob {
                            job_id,
                            path_index: 0,
                        };
                        info!("Bot {:?} claimed job {} at {:?}", entity, job_id, job_position);
                    } else {
                        world.mark_job_unreachable(job_id);
                    }
                }
            }

            BotState::MovingToJob { job_id, mut path_index } => {
                if path_index >= bot.path.len() {
                    // Reached job location, start harvesting
                    bot.state = BotState::Harvesting {
                        job_id,
                        progress: 0.0,
                    };
                    info!("Bot {:?} started harvesting job {}", entity, job_id);
                } else {
                    // Move to next tile
                    bot.position = bot.path[path_index];
                    path_index += 1;
                    bot.state = BotState::MovingToJob { job_id, path_index };
                    bot.movement_cooldown = BASE_MOVE_TIME / bot.get_speed_multiplier(&world);
                }
            }

            BotState::Harvesting { job_id, mut progress } => {
                progress += dt;

                if progress >= HARVEST_TIME {
                    // Complete harvest
                    world.complete_job(job_id);

                    // Path back to core
                    if let Some(path) = find_path(bot.position, world.core_position, &world.grid) {
                        bot.path = path;
                        bot.state = BotState::ReturningToCore {
                            scrap: 5,
                            path_index: 0,
                        };
                        info!("Bot {:?} returning to core with scrap", entity);
                    } else {
                        warn!("Bot {:?} can't path to core! Staying put.", entity);
                        bot.state = BotState::Idle;
                    }
                } else {
                    bot.state = BotState::Harvesting { job_id, progress };
                }
            }

            BotState::ReturningToCore { scrap, mut path_index } => {
                // Check if path is empty (was blocked/invalidated)
                if bot.path.is_empty() {
                    // Need to replan path to core
                    if let Some(path) = find_path(bot.position, world.core_position, &world.grid) {
                        bot.path = path;
                        bot.state = BotState::ReturningToCore { scrap, path_index: 0 };
                        info!("Bot {:?} replanned path to core with {} scrap", entity, scrap);
                    } else {
                        warn!("Bot {:?} can't path to core! Dropping scrap.", entity);
                        bot.state = BotState::Idle;
                    }
                } else if path_index >= bot.path.len() {
                    // Reached end of path - check if actually at core
                    if bot.position == world.core_position {
                        // Delivered!
                        world.add_scrap(scrap);
                        bot.state = BotState::Idle;
                        info!("Bot {:?} delivered {} scrap to core", entity, scrap);
                    } else {
                        // Somehow not at core - replan
                        if let Some(path) = find_path(bot.position, world.core_position, &world.grid) {
                            bot.path = path;
                            bot.state = BotState::ReturningToCore { scrap, path_index: 0 };
                        } else {
                            warn!("Bot {:?} can't reach core! Dropping scrap.", entity);
                            bot.state = BotState::Idle;
                        }
                    }
                } else {
                    // Move to next tile
                    bot.position = bot.path[path_index];
                    path_index += 1;
                    bot.state = BotState::ReturningToCore { scrap, path_index };
                    bot.movement_cooldown = BASE_MOVE_TIME / bot.get_speed_multiplier(&world);
                }
            }
        }
    }
}
