use bevy::prelude::*;
use std::collections::HashMap;
use crate::sim::power_levels::{PowerConsumer, PowerLevel};

/// Component: Aggregates speed multipliers from multiple sources
/// Each key represents a source (e.g., "low_power", "rusty", "speed_boost")
/// Values are multipliers that are all multiplied together
#[derive(Component, Default)]
pub struct SpeedModifiers {
    pub modifiers: HashMap<String, f32>,
}

#[derive(Component)]
pub struct MovementSpeed {
    pub base_speed: f32,
    pub current_speed: f32,
}

impl MovementSpeed {
    pub fn new(base_speed: f32) -> Self {
        Self {
            base_speed,
            current_speed: base_speed,
        }
    }
}

#[derive(Component)]
pub struct PowerLevelEffects {
    pub low_power_speed_multiplier: f32,
}

impl PowerLevelEffects {
    pub fn new(low_power_speed_multiplier: f32) -> Self {
        Self { low_power_speed_multiplier }
    }
}

/// System: Update SpeedModifiers based on current PowerLevel
/// Inserts/removes "low_power" modifier key based on entity's power state
pub fn update_power_level_speed_modifiers(
    mut query: Query<(&PowerConsumer, &PowerLevelEffects, &mut SpeedModifiers)>,
) {
    for (power_consumer, power_level_effects, mut speed_modifiers) in query.iter_mut() {
        if power_consumer.power_level == PowerLevel::LowPower
           && speed_modifiers.modifiers.get("low_power").is_none()
           && power_level_effects.low_power_speed_multiplier != 1.0 {
            speed_modifiers.modifiers.insert("low_power".to_string(), power_level_effects.low_power_speed_multiplier);
        } else if power_consumer.power_level != PowerLevel::LowPower && speed_modifiers.modifiers.get("low_power").is_some() {
            speed_modifiers.modifiers.remove("low_power");
        }
    }
}

pub fn calculate_movement_speed(
    mut query: Query<(&SpeedModifiers, &mut MovementSpeed)>,
) {
    for (speed_modifiers, mut movement_speed) in query.iter_mut() {
        let product = speed_modifiers.modifiers.values().product::<f32>();
        movement_speed.current_speed = movement_speed.base_speed * product;
    }
}
