use bevy::prelude::*;
use std::cmp::max;
use crate::sim::resources::GameResources;


#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PowerLevel {
    Normal,
    LowPower,
    Shutdown,
}

#[derive(Component, Debug)]
pub struct PowerConsumer {
    pub base_consumption: f32,
    pub active_consumption: f32,
    pub low_power_multiplier: f32,
    pub power_level: PowerLevel,
}

impl PowerConsumer {
    pub fn new(base_consumption: f32, active_consumption: f32, low_power_multiplier: f32) -> Self {
        Self {
            base_consumption,
            active_consumption,
            low_power_multiplier,
            power_level: PowerLevel::Normal,
        }
    }

    pub fn current_consumption(&self) -> f32 {
        match self.power_level {
            PowerLevel::Normal => self.base_consumption,
            PowerLevel::LowPower => self.base_consumption * self.low_power_multiplier,
            PowerLevel::Shutdown => 0.0,
        }
    }
}

#[derive(Component, Debug)]
pub struct PowerGenerator {
    pub generation_rate: f32,
}

impl PowerGenerator {
    pub fn new(generation_rate: f32) -> Self {
        Self { generation_rate }
    }
}

/// Main power management system - runs during FixedUpdate
pub fn power_management_system(
    power_generators: Query<&PowerGenerator>,
    mut power_consumers: Query<&mut PowerConsumer>,
    mut game_resources: ResMut<GameResources>
) {
    // Reset all power consumers back to normal level
    for mut consumer in power_consumers.iter_mut() {
        consumer.power_level = PowerLevel::Normal;
    }

    let total_power_generation = power_generators.iter().map(|generator| generator.generation_rate as i32).sum::<i32>();
    let mut total_power_demand = power_consumers.iter().map(|consumer| consumer.current_consumption() as i32).sum::<i32>();

    if total_power_demand > total_power_generation {
        for mut consumer in power_consumers.iter_mut() {
            consumer.power_level = PowerLevel::LowPower;
        }
    }

    // Recalculate total power demand after setting low power level
    total_power_demand = power_consumers.iter().map(|consumer| consumer.current_consumption() as i32).sum::<i32>();

    if total_power_demand > total_power_generation {
        for mut consumer in power_consumers.iter_mut() {
            let usage = consumer.current_consumption() as i32;
            consumer.power_level = PowerLevel::Shutdown;
            total_power_demand -= usage;

            if total_power_demand <= total_power_generation {
                break;
            }
        }

        total_power_demand = total_power_demand.max(0)
    }

    game_resources.power_stored = (total_power_generation - total_power_demand).max(0);
}
