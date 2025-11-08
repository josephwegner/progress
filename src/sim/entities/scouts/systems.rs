use bevy::prelude::*;
use rand::Rng;
use crate::sim::grid::{WorldGrid, TileKind};
use crate::sim::entities::{Position, Path};
use crate::sim::entities::ai_core::AICore;
use crate::sim::entities::scouts::{Scout, ScoutState, ScoutSpawnTimer, SCOUT_DETECTION_RADIUS, SCOUT_WANDER_RANGE, SCOUT_PATHING_RADIUS};
use crate::sim::pathfinding::find_path;
use crate::sim::combat::{Attacker, AttackType, ATTACK_INTERVAL};
use crate::sim::speed_modifiers::{SpeedModifiers, MovementSpeed};
use crate::sim::notifications::{Notification, NotificationSeverity};

// ============================================================================
// SYSTEMS
// ============================================================================

/// Spawns scouts at map edges on a timer
pub fn spawn_scouts_system(
    time: Res<Time>,
    mut timer_resource: ResMut<ScoutSpawnTimer>,
    mut commands: Commands,
    grid: Res<WorldGrid>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    timer_resource.timer.tick(time.delta());

    // Manual spawn with 'M' key or timer
    let should_spawn = timer_resource.timer.just_finished() || keyboard.just_pressed(KeyCode::KeyM);

    if should_spawn {
        spawn_scout(&mut commands, &grid);
        commands.spawn(Notification::new(
            "Scout spawned!".to_string(),
            NotificationSeverity::Warning,
        ));

        // Reset timer when spawning
        timer_resource.timer.reset();
    }
}

/// Moves scouts towards their target, chooses new target when reached
pub fn scout_movement_system(
    mut commands: Commands,
    mut scouts: Query<(Entity, &mut Scout, &Position, Option<&Path>), With<Scout>>,
    grid: Res<WorldGrid>
) {
    for (entity, mut scout, position, path) in scouts.iter_mut() {
        // Only assign new paths when scout has no path
        // Path traversal and removal is handled by move_entities_along_path system
        if path.is_none() && scout.state != ScoutState::Jamming {
            if scout.state != ScoutState::Wandering {
                scout.state = ScoutState::Wandering;
            }

            let target = choose_random_destination(position, &grid);
            if let Some(path_nodes) = find_path(position.clone(), target.clone(), &grid) {
                commands.entity(entity).insert(Path {
                    nodes: path_nodes,
                    current_idx: 0,
                    movement_cooldown: 0.0,
                });
            }
        }
    }
}

/// Checks if any scout can detect the AI Core
pub fn scout_detection_system(
    mut commands: Commands,
    mut scouts: Query<(Entity, &mut Scout, &Position), With<Scout>>,
    ai_core: Query<&Position, With<AICore>>,
    grid: Res<WorldGrid>,
) {
    if let Ok(ai_core_pos) = ai_core.get_single() {
        for (entity, mut scout, position) in scouts.iter_mut() {
            let distance = (position.x as f32 - ai_core_pos.x as f32).powi(2) + (position.y as f32 - ai_core_pos.y as f32).powi(2);

            if scout.state != ScoutState::Jamming && distance <= SCOUT_PATHING_RADIUS.powi(2) {
                scout.state = ScoutState::Jamming;
                commands.entity(entity).remove::<Path>();
                commands.entity(entity).insert(Attacker {
                    cooldown: Timer::from_seconds(ATTACK_INTERVAL, TimerMode::Repeating),
                    attack_type: AttackType::JammingPulse,
                });
                info!("Scout at ({},{}) is jamming", position.x, position.y);

                continue;
            } else if scout.state != ScoutState::Detected && scout.state != ScoutState::Jamming {
                if distance <= SCOUT_DETECTION_RADIUS.powi(2) && !is_line_blocked_by_walls(position, ai_core_pos, &grid) {
                    info!("Scout at ({},{}) is detected at distance {}", position.x, position.y, distance);
                let path_to_core = find_path(position.clone(), ai_core_pos.clone(), &grid);

                    if let Some(path_nodes) = path_to_core {
                        commands.entity(entity).remove::<Path>();
                        commands.entity(entity).insert(Path {
                            nodes: path_nodes,
                            current_idx: 0,
                            movement_cooldown: 0.0,
                        });
                    }

                    scout.state = ScoutState::Detected;
                }
            }
        }
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Helper: Spawn a single scout at a random edge position
fn spawn_scout(commands: &mut Commands, grid: &WorldGrid) {
    let edge = choose_random_edge();
    let position = choose_random_position_on_edge(edge, &grid);
    commands.spawn((
        Scout { state: ScoutState::Wandering },
        Position { x: position.0, y: position.1 },
        SpeedModifiers::default(),
        MovementSpeed::new(1.5),
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(1.0, 0.2, 0.2),
                custom_size: Some(Vec2::new(
                    crate::sim::grid::TILE_SIZE * 0.6,
                    crate::sim::grid::TILE_SIZE * 0.6
                )),
                ..default()
            },
            transform: Transform::from_xyz(
                position.0 as f32 * crate::sim::grid::TILE_SIZE,
                position.1 as f32 * crate::sim::grid::TILE_SIZE,
                2.0,
            ),
            ..default()
        },
    ));
}

enum Edge {
    North,
    South,
    East,
    West,
}

fn choose_random_edge() -> Edge {
    let choice = rand::thread_rng().gen_range(0..4);
    match choice {
        0 => Edge::North,
        1 => Edge::South,
        2 => Edge::East,
        _ => Edge::West,
    }
}

fn choose_random_position_on_edge(edge: Edge, grid: &WorldGrid) -> (u32, u32) {
    match edge {
        Edge::North => (rand::thread_rng().gen_range(0..grid.w), grid.h - 1),
        Edge::South => (rand::thread_rng().gen_range(0..grid.w), 0),
        Edge::East => (grid.w - 1, rand::thread_rng().gen_range(0..grid.h)),
        Edge::West => (0, rand::thread_rng().gen_range(0..grid.h)),
    }
}

/// Helper: Choose random destination within SCOUT_WANDER_RANGE tiles
fn choose_random_destination(
    position: &Position,
    grid: &WorldGrid,
) -> Position {
    let x_offset = rand::thread_rng().gen_range(-(SCOUT_WANDER_RANGE as i32)..=(SCOUT_WANDER_RANGE as i32));
    let y_offset = rand::thread_rng().gen_range(-(SCOUT_WANDER_RANGE as i32)..=(SCOUT_WANDER_RANGE as i32));

    let x = ((position.x as i32 + x_offset).max(0).min(grid.w as i32 - 1)) as u32;
    let y = ((position.y as i32 + y_offset).max(0).min(grid.h as i32 - 1)) as u32;

    Position { x, y }
}

/// Helper: Check if walls block line between two points
fn is_line_blocked_by_walls(
    start: &Position,
    end: &Position,
    grid: &WorldGrid,
) -> bool {
    let mut x = start.x as f32;
    let mut y = start.y as f32;
    let dx = (end.x as i32 - start.x as i32) as f32;
    let dy = (end.y as i32 - start.y as i32) as f32;
    let steps = dx.abs().max(dy.abs()) as u32;

    if steps == 0 {
        return false;
    }

    let x_step = dx / steps as f32;
    let y_step = dy / steps as f32;

    for _ in 0..steps {
        x += x_step;
        y += y_step;
        let tile_x = x.round() as u32;
        let tile_y = y.round() as u32;

        if tile_x < grid.w && tile_y < grid.h {
            if grid.tiles[grid.idx(tile_x, tile_y)] == TileKind::Wall {
                return true;
            }
        }
    }

    false
}
