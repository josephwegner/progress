use bevy::prelude::*;
use crate::sim::entities::{Position, SCOUT_JAMMING_RADIUS};
use crate::sim::conditions::{Condition, ConditionType};

/// How often an attacker attempts to execute its attack
pub const ATTACK_INTERVAL: f32 = 1.0; // seconds

#[derive(Component)]
pub struct Attacker {
    pub cooldown: Timer,
    pub attack_type: AttackType,
}

impl Default for Attacker {
    fn default() -> Self {
        Self {
            cooldown: Timer::from_seconds(ATTACK_INTERVAL, TimerMode::Repeating),
            attack_type: AttackType::JammingPulse,
        }
    }
}

/// Types of attacks entities can perform
///
/// Each attack type defines:
/// - What condition it applies
/// - What radius it affects
/// - What targets it can hit (could add target filters later)
#[derive(Clone, Debug)]
pub enum AttackType {
    JammingPulse
}

/// System: Tick attack cooldowns and execute attacks when ready
/// Run this in FixedUpdate schedule for consistent timing
pub fn combat_system(
    time: Res<Time>,
    mut attackers: Query<(Entity, &mut Attacker, &Position)>,
    mut commands: Commands,
    potential_targets: Query<(Entity, &Position)>,
) {
    for (entity, mut attacker, position) in attackers.iter_mut() {
        attacker.cooldown.tick(time.delta());
        if attacker.cooldown.just_finished() {
            execute_attack(entity, position, &attacker.attack_type, &mut commands, &potential_targets);
        }
    }
}

/// Execute an attack from a position
fn execute_attack(
    attacker_entity: Entity,
    attacker_pos: &Position,
    attack_type: &AttackType,
    commands: &mut Commands,
    potential_targets: &Query<(Entity, &Position)>,
) {
    match attack_type {
        AttackType::JammingPulse => {
            jamming_pulse(attacker_entity, attacker_pos, commands, potential_targets);
        }
    }
}

// Apply a jamming attack to all targets within the radius
fn jamming_pulse(attacker_entity: Entity, attacker_pos: &Position, commands: &mut Commands, potential_targets: &Query<(Entity, &Position)>) {
    for (target, target_pos) in potential_targets.iter() {
        if target == attacker_entity {
            continue;
        }

        let distance = distance_squared(attacker_pos, target_pos);
        if distance <= SCOUT_JAMMING_RADIUS.powi(2) {
            info!("Jamming target at ({},{})", target_pos.x, target_pos.y);
            commands.entity(target).insert(Condition::new(ConditionType::Jammed, 1.0));
        }
    }
}

/// Helper: Calculate squared distance between two positions
///
/// Returns distance² to avoid expensive sqrt() operations
/// Compare against radius² for efficiency
fn distance_squared(pos1: &Position, pos2: &Position) -> f32 {
    (pos1.x as f32 - pos2.x as f32).powi(2) + (pos1.y as f32 - pos2.y as f32).powi(2)
}
