use bevy::prelude::*;
use crate::world::World;
use crate::bot::Bot;

pub const BOT_POWER_IDLE: f32 = 2.0;
pub const BOT_POWER_ACTIVE: f32 = 5.0;
pub const BOT_POWER_LOW: f32 = 1.0;

#[derive(Debug, Clone)]
pub struct PowerSystem {
    pub generation: f32,
    pub consumption_buildings: f32,
    pub storage: f32,
    pub capacity: f32,
    pub low_power_mode: bool,
    pub collapse_timer: f32,
}

impl Default for PowerSystem {
    fn default() -> Self {
        Self {
            generation: 10.0,
            consumption_buildings: 0.0,
            storage: 50.0,
            capacity: 100.0,
            low_power_mode: false,
            collapse_timer: 0.0,
        }
    }
}

impl PowerSystem {
    pub fn update(&mut self, bot_count: usize, active_bot_count: usize, dt: f32) {
        // Calculate bot consumption
        let idle_bots = bot_count - active_bot_count;
        let bot_consumption = if self.low_power_mode {
            bot_count as f32 * BOT_POWER_LOW
        } else {
            (idle_bots as f32 * BOT_POWER_IDLE) + (active_bot_count as f32 * BOT_POWER_ACTIVE)
        };

        let total_consumption = self.consumption_buildings + bot_consumption;
        let net_power = self.generation - total_consumption;

        // Update storage
        self.storage = (self.storage + net_power * dt).clamp(0.0, self.capacity);

        // Enter low power mode if storage is critically low
        self.low_power_mode = self.storage < 10.0 && net_power < 0.0;

        // Track collapse timer
        if self.is_collapsed() {
            self.collapse_timer += dt;
        } else {
            self.collapse_timer = 0.0;
        }
    }

    pub fn is_collapsed(&self) -> bool {
        self.storage <= 0.0 && self.generation < self.consumption_buildings
    }

    pub fn add_generation(&mut self, amount: f32) {
        self.generation += amount;
    }

    pub fn add_consumption(&mut self, amount: f32) {
        self.consumption_buildings += amount;
    }

    pub fn add_capacity(&mut self, amount: f32) {
        self.capacity += amount;
    }

    pub fn get_speed_multiplier(&self) -> f32 {
        if self.low_power_mode {
            0.2  // 20% speed in low power mode
        } else {
            1.0
        }
    }
}

pub fn update_power_system(
    mut world: ResMut<World>,
    bots: Query<&Bot>,
    time: Res<Time>,
) {
    let bot_count = bots.iter().count();
    let active_bot_count = bots.iter().filter(|b| b.is_active()).count();

    world.power.update(bot_count, active_bot_count, time.delta_seconds());
}
