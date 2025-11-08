use bevy::prelude::*;

// ============================================================================
// BOT COMPONENTS
// ============================================================================

/// Core bot component - just identity and power consumption
#[derive(Component, Clone, Debug)]
pub struct Bot {
    pub power_drain_idle: i32,
    pub power_drain_active: i32,
}

impl Default for Bot {
    fn default() -> Self {
        Self {
            power_drain_idle: 2,
            power_drain_active: 5,
        }
    }
}

// ============================================================================
// STATE MARKER COMPONENTS
// ============================================================================

/// Bot has been assigned a job
#[derive(Component, Debug)]
pub struct HasJob(pub Entity);

/// Bot is carrying scrap to deliver
#[derive(Component, Debug)]
pub struct CarryingScrap(pub i32);
