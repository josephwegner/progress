use bevy::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

#[derive(Component, Clone, Debug)]
pub struct AICore;

#[derive(Component, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum BotState {
    Idle,
    MovingToJob,
    Working,
    Hauling,
}

#[derive(Component, Clone, Debug)]
pub struct Bot {
    pub state: BotState,
    pub carry_scrap: i32,
    pub current_job: Option<Entity>,
    pub power_drain_idle: i32,
    pub power_drain_active: i32,
}

impl Default for Bot {
    fn default() -> Self {
        Self {
            state: BotState::Idle,
            carry_scrap: 0,
            current_job: None,
            power_drain_idle: 2,
            power_drain_active: 5,
        }
    }
}

#[derive(Component, Clone, Debug)]
pub struct Path {
    pub nodes: Vec<(u32, u32)>,
    pub current_idx: usize,
}

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

pub fn spawn_ai_core(
    mut commands: Commands,
    grid: Res<crate::sim::grid::WorldGrid>,
) {
    let core_x = grid.w / 2;
    let core_y = grid.h / 2;
    let world_x = core_x as f32 * crate::sim::grid::TILE_SIZE;
    let world_y = core_y as f32 * crate::sim::grid::TILE_SIZE;

    commands.spawn((
        AICore,
        Position { x: core_x, y: core_y },
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.2, 0.8, 1.0),
                custom_size: Some(Vec2::new(crate::sim::grid::TILE_SIZE, crate::sim::grid::TILE_SIZE)),
                ..default()
            },
            transform: Transform::from_xyz(world_x, world_y, 1.0),
            ..default()
        },
    ));
}

pub fn spawn_initial_bots(
    mut commands: Commands,
    core_query: Query<&Position, With<AICore>>,
) {
    if let Ok(core_pos) = core_query.get_single() {
        for i in 0..2 {
            let offset = if i == 0 { 2 } else { -2 };
            commands.spawn((
                Bot::default(),
                Position {
                    x: (core_pos.x as i32 + offset) as u32,
                    y: core_pos.y
                },
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(0.9, 0.9, 0.2),
                        custom_size: Some(Vec2::new(
                            crate::sim::grid::TILE_SIZE * 0.6,
                            crate::sim::grid::TILE_SIZE * 0.6
                        )),
                        ..default()
                    },
                    transform: Transform::from_xyz(
                        ((core_pos.x as i32 + offset) as f32) * crate::sim::grid::TILE_SIZE,
                        core_pos.y as f32 * crate::sim::grid::TILE_SIZE,
                        2.0,
                    ),
                    ..default()
                },
            ));
        }
    }
}

pub fn update_sprite_positions(
    mut query: Query<(&Position, &mut Transform), Or<(With<Bot>, With<AICore>)>>,
) {
    for (pos, mut transform) in query.iter_mut() {
        transform.translation.x = pos.x as f32 * crate::sim::grid::TILE_SIZE;
        transform.translation.y = pos.y as f32 * crate::sim::grid::TILE_SIZE;
    }
}
