use bevy::prelude::*;
use crate::sim::power_levels::{PowerConsumer, PowerLevel};

/// Component: Represents a status effect applied to an entity
#[derive(Component, Clone, Debug)]
pub struct Condition {
    pub condition_type: ConditionType,
    pub duration: Timer,
}

impl Condition {
    /// Create a new condition with specified type and duration in seconds
    pub fn new(condition_type: ConditionType, duration_secs: f32) -> Self {
        Self {
            condition_type,
            duration: Timer::from_seconds(duration_secs, TimerMode::Once),
        }
    }
}

/// Types of conditions that can affect entities
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConditionType {
    Jammed
}

/// System: Tick all condition timers and remove expired conditions
/// Run this in FixedUpdate for consistent timing
pub fn condition_system(
    time: Res<Time>,
    mut conditions: Query<(Entity, &mut Condition)>,
    mut commands: Commands,
) {
    for (entity, mut condition) in conditions.iter_mut() {
        condition.duration.tick(time.delta());
        if condition.duration.just_finished() {
            commands.entity(entity).remove::<Condition>();
        }
    }
}

/// Helper: Check if entity has a specific condition type
pub fn has_condition(condition: &Condition, condition_type: ConditionType) -> bool {
    condition.condition_type == condition_type
}

/// System: Apply condition effects to entities
pub fn apply_condition_effects_system(
    mut entities: Query<(&Condition, &mut Sprite, Option<&mut PowerConsumer>)>,
) {
    for (condition, mut sprite, power_consumer) in entities.iter_mut() {
        match condition.condition_type {
            ConditionType::Jammed => {
                sprite.color = Color::srgb(0.0, 1.0, 0.0).with_alpha(0.5);

                if let Some(mut pc) = power_consumer {
                    if pc.power_level != PowerLevel::Shutdown {
                        pc.power_level = PowerLevel::LowPower;
                    }
                }
            }
        }
    }
}
