use bevy::prelude::*;
use crate::sim::resources::GameResources;
use crate::sim::notifications::{Notification, NotificationSeverity};
use crate::WorldGrid;

const WIN_TIME_SECONDS: f32 = 300.0; // 5 minutes
const POWER_COLLAPSE_THRESHOLD: f32 = 60.0; // 60 seconds

/// Tracks elapsed game time
#[derive(Resource, Default)]
pub struct GameTime {
    pub elapsed_seconds: f64,
}

/// Current game state
#[derive(Resource, Default, PartialEq)]
pub enum GameState {
    #[default]
    Playing,
    Won,
    Lost,
}

/// Tracks duration at zero power
#[derive(Resource, Default)]
pub struct PowerCollapseTimer {
    pub timer: Timer,
}

impl PowerCollapseTimer {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(POWER_COLLAPSE_THRESHOLD, TimerMode::Once),
        }
    }
}

/// Tracks which tiles have changed this frame to invalidate affected paths
#[derive(Resource, Default)]
pub struct WorldChangeTracker {
    pub tiles_changed: Vec<(u32, u32)>,
}

/// Marker for game over UI
#[derive(Component)]
pub struct GameOverUI;

/// System: Check WorldGrid.chunk_dirty and populate WorldChangeTracker with changed tile positions
/// Run in FixedUpdate, early (before invalidation)
pub fn track_world_changes(
    mut tracker: ResMut<WorldChangeTracker>,
    grid: Res<crate::sim::grid::WorldGrid>,  // Need full path
) {
    tracker.tiles_changed.clear();

    // Iterate through chunks
    for cy in 0..grid.chunk_rows {
        for cx in 0..grid.chunk_cols {
            let chunk_idx = (cy * grid.chunk_cols + cx) as usize;

            if grid.chunk_dirty[chunk_idx] {
                // This chunk changed - mark all tiles in it as changed
                let x_start = cx * grid.chunk_w;
                let y_start = cy * grid.chunk_h;
                let x_end = (x_start + grid.chunk_w).min(grid.w);
                let y_end = (y_start + grid.chunk_h).min(grid.h);

                for y in y_start..y_end {
                    for x in x_start..x_end {
                        tracker.tiles_changed.push((x, y));
                    }
                }
            }
        }
    }
}

/// System: Check win condition (5 min survival, 50 compute, sustainable power)
pub fn check_win_condition(
    game_time: Res<GameTime>,
    resources: Res<GameResources>,
    mut game_state: ResMut<GameState>,
    mut commands: Commands,
) {
    if *game_state != GameState::Playing {
        return;
    }

    let time_goal_met = game_time.elapsed_seconds >= WIN_TIME_SECONDS as f64;
    let compute_goal_met = resources.compute_cap >= 50;
    let power_sustainable = resources.power_produced >= resources.power_consumed;

    if time_goal_met && compute_goal_met && power_sustainable {
        *game_state = GameState::Won;
        info!("WIN: Time {:.0}s, Compute {}, Power sustainable",
              game_time.elapsed_seconds, resources.compute_cap);

        commands.spawn(Notification::new(
            "VICTORY! AI Colony Established".to_string(),
            NotificationSeverity::Info,
        ));
    }
}

/// System: Check power collapse lose condition (60 sec at 0 power)
pub fn check_power_collapse(
    time: Res<Time>,
    resources: Res<GameResources>,
    mut collapse_timer: ResMut<PowerCollapseTimer>,
    mut game_state: ResMut<GameState>,
    mut commands: Commands,
) {
    if *game_state != GameState::Playing {
        return;
    }

    if resources.power_stored == 0 && resources.power_produced <= resources.power_consumed {
        collapse_timer.timer.tick(time.delta());

        if collapse_timer.timer.just_finished() {
            *game_state = GameState::Lost;
            info!("LOSE: Power collapse - 60 seconds at zero power");

            commands.spawn(Notification::new(
                "DEFEAT: Power Collapsed".to_string(),
                NotificationSeverity::Critical,
            ));
        }
    } else {
        // Reset timer if power recovers
        collapse_timer.timer.reset();
    }
}

/// System: Display game over UI
pub fn display_game_over(
    game_state: Res<GameState>,
    mut commands: Commands,
    existing_ui: Query<Entity, With<GameOverUI>>,
) {
    // Remove old UI if state changed
    if game_state.is_changed() {
        for entity in existing_ui.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }

    // Show game over screen
    if *game_state == GameState::Won || *game_state == GameState::Lost {
        let (message, color) = match *game_state {
            GameState::Won => ("VICTORY!\n\nAI Colony Established\n\nRefresh to play again", Color::srgb(0.0, 1.0, 0.0)),
            GameState::Lost => ("DEFEAT\n\nPower Collapsed\n\nRefresh to play again", Color::srgb(1.0, 0.0, 0.0)),
            _ => return,
        };

        commands.spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                background_color: Color::srgba(0.0, 0.0, 0.0, 0.8).into(),
                ..default()
            },
            GameOverUI,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                message,
                TextStyle {
                    font_size: 48.0,
                    color,
                    ..default()
                },
            ).with_style(Style {
                align_self: AlignSelf::Center,
                ..default()
            }));
        });
    }
}
