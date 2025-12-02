use std::collections::HashMap;
use bevy::prelude::*;
use crate::grid::Position;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ReservationKey {
  Tile(Position),
  Entity(Entity),
}

#[derive(Resource)]
pub struct ReservationSystem {
  reservations: HashMap<ReservationKey, Entity>,
}

impl Default for ReservationSystem {
  fn default() -> Self {
    Self {
      reservations: HashMap::new(),
    }
  }
}

impl ReservationSystem {
  pub fn try_reserve(&mut self, key: ReservationKey, bot_entity: Entity) -> bool {
    if self.reservations.contains_key(&key) {
      false  // Already reserved by another bot
    } else {
      self.reservations.insert(key, bot_entity);
      true
    }
  }

  pub fn unreserve(&mut self, key: &ReservationKey) {
    self.reservations.remove(key);
  }

  pub fn is_reserved(&self, key: &ReservationKey) -> bool {
    self.reservations.contains_key(key)
  }

  /// Get who reserved something
  pub fn get_reserver(&self, key: &ReservationKey) -> Option<Entity> {
    self.reservations.get(key).copied()
  }
}
