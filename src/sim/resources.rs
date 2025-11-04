use bevy::prelude::*;
use serde::{Serialize, Deserialize};

/// Tracks elapsed game time for notifications, win/lose conditions, and events
#[derive(Resource, Default)]
pub struct GameTime {
    pub elapsed_seconds: f64,
}

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct GameResources {
    pub scrap: i32,
    pub power_produced: i32,
    pub power_consumed: i32,
    pub power_consumed_buildings: i32,
    pub power_stored: i32,
    pub power_cap: i32,
    pub compute_cycles: i32,
    pub compute_cap: i32,
}

impl Default for GameResources {
    fn default() -> Self {
        Self {
            scrap: 0,
            power_produced: 10,
            power_consumed: 0,
            power_consumed_buildings: 0,
            power_stored: 50,
            power_cap: 100,
            compute_cycles: 0,
            compute_cap: 10,
        }
    }
}

impl GameResources {
    pub fn has_power(&self) -> bool {
        self.power_stored > 0 || self.power_produced > self.power_consumed
    }

    pub fn can_afford_scrap(&self, amount: i32) -> bool {
        self.scrap >= amount
    }

    pub fn spend_scrap(&mut self, amount: i32) -> bool {
        if self.can_afford_scrap(amount) {
            self.scrap -= amount;
            true
        } else {
            false
        }
    }

    pub fn add_scrap(&mut self, amount: i32) {
        self.scrap += amount;
    }

    pub fn add_compute(&mut self, amount: i32) {
        self.compute_cap += amount;
    }

    pub fn add_power_consumption(&mut self, amount: i32) {
        self.power_consumed_buildings += amount;
    }

    pub fn add_power_production(&mut self, amount: i32) {
        self.power_produced += amount;
    }

    pub fn add_power_capacity(&mut self, amount: i32) {
        self.power_cap += amount;
    }
}

pub fn tick_resources(
    mut resources: ResMut<GameResources>,
    time: Res<Time>,
    bot_query: Query<&crate::sim::entities::Bot>,
) {
    let mut bot_power_drain = 0;
    for bot in bot_query.iter() {
        bot_power_drain += match bot.state {
            crate::sim::entities::BotState::Idle => bot.power_drain_idle,
            _ => bot.power_drain_active,
        };
    }

    resources.power_consumed = resources.power_consumed_buildings + bot_power_drain;

    let net_power = resources.power_produced - resources.power_consumed;

    if net_power > 0 {
        resources.power_stored = (resources.power_stored + (net_power as f32 * time.delta_seconds() * 10.0) as i32)
            .min(resources.power_cap);
    } else if net_power < 0 {
        resources.power_stored = (resources.power_stored + (net_power as f32 * time.delta_seconds() * 10.0) as i32)
            .max(0);
    }

    if resources.has_power() {
        resources.compute_cycles = (resources.compute_cycles + (time.delta_seconds() * 5.0) as i32)
            .min(resources.compute_cap);
    }
}
