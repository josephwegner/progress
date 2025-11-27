use bevy::prelude::*;
use rand::Rng;
use crate::grid::{TILE_SIZE, TileKind};
use crate::world::{World, GameOverState};
use crate::bot::Bot;
use crate::pathfinding::Position;

const SCOUT_SPAWN_INTERVAL: f32 = 45.0;  // Spawn scout every 45 seconds
const SCOUT_SPEED: f32 = 0.3;  // Slower than bots
const SCOUT_DETECTION_RANGE: f32 = 5.0;  // Tiles
const SCOUT_JAM_RANGE: f32 = 3.0;  // Tiles

#[derive(Resource)]
pub struct ScoutSpawnTimer {
    pub timer: Timer,
}

impl Default for ScoutSpawnTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(SCOUT_SPAWN_INTERVAL, TimerMode::Repeating),
        }
    }
}

#[derive(Component, Debug)]
pub struct Scout {
    pub position: Position,
    pub target: Position,
    pub movement_cooldown: f32,
}

impl Scout {
    pub fn new(position: Position, target: Position) -> (Self, SpriteBundle) {
        let scout = Self {
            position,
            target,
            movement_cooldown: 0.0,
        };

        let sprite = SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(1.0, 0.2, 0.2),
                custom_size: Some(Vec2::new(TILE_SIZE * 0.7, TILE_SIZE * 0.7)),
                ..default()
            },
            transform: Transform::from_xyz(
                position.0 as f32 * TILE_SIZE,
                position.1 as f32 * TILE_SIZE,
                3.0,  // Above bots
            ),
            ..default()
        };

        (scout, sprite)
    }
}

pub fn spawn_scouts_system(
    mut commands: Commands,
    mut timer: ResMut<ScoutSpawnTimer>,
    world: Res<World>,
    time: Res<Time>,
) {
    if world.game_over.is_some() {
        return;
    }

    timer.timer.tick(time.delta());

    if timer.timer.just_finished() {
        // Spawn scout at random edge
        let mut rng = rand::thread_rng();
        let edge = rng.gen_range(0..4);  // 0=top, 1=right, 2=bottom, 3=left

        let spawn_pos = match edge {
            0 => (rng.gen_range(0..world.grid.width), world.grid.height - 1),  // Top
            1 => (world.grid.width - 1, rng.gen_range(0..world.grid.height)),  // Right
            2 => (rng.gen_range(0..world.grid.width), 0),  // Bottom
            _ => (0, rng.gen_range(0..world.grid.height)),  // Left
        };

        // Target is AI core
        let target = world.core_position;

        commands.spawn(Scout::new(spawn_pos, target));

        info!("Scout spawned at {:?}, heading to {:?}", spawn_pos, target);
    }
}

pub fn update_scouts(
    mut scouts: Query<(Entity, &mut Scout, &mut Transform)>,
    world: Res<World>,
    time: Res<Time>,
    mut commands: Commands,
) {
    if world.game_over.is_some() {
        return;
    }

    for (entity, mut scout, mut transform) in scouts.iter_mut() {
        // Update movement cooldown
        if scout.movement_cooldown > 0.0 {
            scout.movement_cooldown -= time.delta_seconds();
            continue;
        }

        // Simple movement toward target
        let dx = (scout.target.0 as i32 - scout.position.0 as i32).signum();
        let dy = (scout.target.1 as i32 - scout.position.1 as i32).signum();

        // Try to move (prefer diagonal movement)
        if dx != 0 && dy != 0 {
            let new_x = (scout.position.0 as i32 + dx) as u32;
            let new_y = (scout.position.1 as i32 + dy) as u32;

            if world.grid.is_walkable(new_x, new_y) {
                scout.position = (new_x, new_y);
                scout.movement_cooldown = 0.3 / SCOUT_SPEED;
            }
        } else if dx != 0 {
            let new_x = (scout.position.0 as i32 + dx) as u32;
            if world.grid.is_walkable(new_x, scout.position.1) {
                scout.position = (new_x, scout.position.1);
                scout.movement_cooldown = 0.3 / SCOUT_SPEED;
            }
        } else if dy != 0 {
            let new_y = (scout.position.1 as i32 + dy) as u32;
            if world.grid.is_walkable(scout.position.0, new_y) {
                scout.position = (scout.position.0, new_y);
                scout.movement_cooldown = 0.3 / SCOUT_SPEED;
            }
        }

        // Check if reached target (despawn)
        if scout.position == scout.target {
            commands.entity(entity).despawn_recursive();
            info!("Scout reached target and despawned");
        }

        // Update sprite position
        transform.translation.x = scout.position.0 as f32 * TILE_SIZE;
        transform.translation.y = scout.position.1 as f32 * TILE_SIZE;
    }
}

pub fn scout_detection_system(
    scouts: Query<&Scout>,
    mut world: ResMut<World>,
) {
    if world.game_over.is_some() {
        return;
    }

    for scout in scouts.iter() {
        // Check distance to AI core
        let dx = (scout.position.0 as f32 - world.core_position.0 as f32).abs();
        let dy = (scout.position.1 as f32 - world.core_position.1 as f32).abs();
        let distance = (dx * dx + dy * dy).sqrt();

        if distance <= SCOUT_DETECTION_RANGE {
            // Check line of sight (simple check - no walls blocking)
            let has_los = check_line_of_sight(scout.position, world.core_position, &world.grid);

            if has_los {
                world.game_over = Some(GameOverState::Detected);
                warn!("Scout detected AI Core! Game over!");
                return;
            }
        }
    }
}

pub fn scout_jamming_system(
    scouts: Query<&Scout>,
    mut bots: Query<&mut Bot>,
) {
    // Reset all bots to not jammed
    for mut bot in bots.iter_mut() {
        bot.jammed = false;
    }

    // Apply jamming to bots near scouts
    for scout in scouts.iter() {
        for mut bot in bots.iter_mut() {
            let dx = (scout.position.0 as f32 - bot.position.0 as f32).abs();
            let dy = (scout.position.1 as f32 - bot.position.1 as f32).abs();
            let distance = (dx * dx + dy * dy).sqrt();

            if distance <= SCOUT_JAM_RANGE {
                bot.jammed = true;
            }
        }
    }
}

fn check_line_of_sight(from: Position, to: Position, grid: &crate::grid::Grid) -> bool {
    // Bresenham's line algorithm
    let mut x0 = from.0 as i32;
    let mut y0 = from.1 as i32;
    let x1 = to.0 as i32;
    let y1 = to.1 as i32;

    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        // Check if current tile blocks LOS (walls block)
        if x0 >= 0 && y0 >= 0 && x0 < grid.width as i32 && y0 < grid.height as i32 {
            if grid.get(x0 as u32, y0 as u32) == TileKind::Wall {
                return false;  // Wall blocks LOS
            }
        }

        if x0 == x1 && y0 == y1 {
            break;
        }

        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }

    true  // Clear line of sight
}
