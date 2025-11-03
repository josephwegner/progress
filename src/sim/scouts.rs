use bevy::prelude::*;
use rand::Rng;
use crate::sim::grid::{WorldGrid, TileKind};
use crate::sim::entities::{AICore, Position, Path};
use crate::sim::pathfinding::find_path;

pub const SCOUT_DETECTION_RADIUS: f32 = 15.0;
pub const SCOUT_JAMMING_RADIUS: f32 = 1.0;
pub const SCOUT_WANDER_RANGE: u32 = 10;
pub const SCOUT_SPAWN_INTERVAL: f32 = 120.0; // seconds

#[derive(Component, Clone, Debug, PartialEq, Eq)]
pub enum ScoutState {
    Wandering,
    Detected,
    Jamming,
}

#[derive(Component, Clone, Debug)]
pub struct Scout {
    pub state: ScoutState
}

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

/// Spawns scouts at map edges on a timer
pub fn spawn_scouts_system(
    time: Res<Time>,
    mut timer_resource: ResMut<ScoutSpawnTimer>,
    mut commands: Commands,
    grid: Res<WorldGrid>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    timer_resource.timer.tick(time.delta());

    // Manual spawn with 'S' key
    let should_spawn = timer_resource.timer.just_finished() || keyboard.just_pressed(KeyCode::KeyM);

    if should_spawn {
        spawn_scout(&mut commands, &grid);

        // Reset timer when spawning
        timer_resource.timer.reset();
    }
}

/// Helper: Spawn a single scout at a random edge position
fn spawn_scout(commands: &mut Commands, grid: &WorldGrid) {
    let edge = choose_random_edge();
    let position = choose_random_position_on_edge(edge, &grid);
    commands.spawn((
        Scout { state: ScoutState::Wandering },
        Position { x: position.0, y: position.1 },
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
                commands.entity(entity).insert(crate::sim::entities::Path {
                    nodes: path_nodes,
                    current_idx: 0,
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
            if scout.state != ScoutState::Detected {
                let distance = (position.x as f32 - ai_core_pos.x as f32).powi(2) + (position.y as f32 - ai_core_pos.y as f32).powi(2);
                if distance <= SCOUT_JAMMING_RADIUS.powi(2) {
                    scout.state = ScoutState::Jamming;
                    commands.entity(entity).remove::<Path>();
                    
                    info!("Scout at ({},{}) is jamming", position.x, position.y);
                } else if distance <= SCOUT_DETECTION_RADIUS.powi(2) && !is_line_blocked_by_walls(position, ai_core_pos, &grid) {
                    info!("Scout at ({},{}) is detected at distance {}", position.x, position.y, distance);
                    let path_to_core = find_path(position.clone(), ai_core_pos.clone(), &grid);

                    if let Some(path_nodes) = path_to_core {
                        commands.entity(entity).remove::<Path>();
                        commands.entity(entity).insert(Path {
                            nodes: path_nodes,
                            current_idx: 0,
                        });
                    }

                    scout.state = ScoutState::Detected;
                }
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    // Test: Scout spawns at map edge
    #[test]
    fn test_scout_spawns_at_edge() {
        // Setup world with grid
        // Call spawn_scout
        // Assert scout Position is at x=0 or x=MAP_W-1 or y=0 or y=MAP_H-1
        todo!("Implement test")
    }

    // Test: Scout chooses random destination within 10 tiles
    #[test]
    fn test_scout_picks_random_destination() {
        // Setup scout at position (5, 5)
        // Call scout_choose_destination
        // Assert destination is within 10 tile radius
        todo!("Implement test")
    }

    // Test: Scout detects AI Core within radius
    #[test]
    fn test_scout_detects_ai_core_in_range() {
        // Setup scout at (10, 10)
        // Setup AI Core at (15, 15) - within detection radius
        // No walls between them
        // Call scout_detection_system
        // Assert scout state is Detected
        todo!("Implement test")
    }

    // Test: Scout cannot detect through walls
    #[test]
    fn test_scout_blocked_by_walls() {
        // Setup scout and AI Core within detection radius
        // Add wall tiles between them
        // Call scout_detection_system
        // Assert scout state is still Wandering
        todo!("Implement test")
    }

    // Test: Scout spawns on timer
    #[test]
    fn test_scout_spawn_timer() {
        // Setup ScoutSpawnTimer with short interval
        // Advance time past interval
        // Call spawn_scouts_system
        // Assert new scout entity exists
        todo!("Implement test")
    }
}