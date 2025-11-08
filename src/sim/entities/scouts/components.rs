use bevy::prelude::*;

// ============================================================================
// CONSTANTS
// ============================================================================

pub const SCOUT_DETECTION_RADIUS: f32 = 15.0;
pub const SCOUT_JAMMING_RADIUS: f32 = 5.0;
pub const SCOUT_WANDER_RANGE: u32 = 10;
pub const SCOUT_SPAWN_INTERVAL: f32 = 120.0; // seconds
pub const SCOUT_PATHING_RADIUS: f32 = 1.0;

// ============================================================================
// COMPONENTS
// ============================================================================

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

// ============================================================================
// RESOURCES
// ============================================================================

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
