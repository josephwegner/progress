use bevy::prelude::*;
use crate::grid::{Position, Grid, Impassable};
use crate::entities::scrap::Scrap;
use crate::reservation::{ReservationSystem, ReservationKey};

#[derive(Component)]
pub struct Bot {
    /// The current reservation this bot is working on (if any)
    pub current_reservation: Option<ReservationKey>,
}

impl Bot {
    pub fn new() -> Self {
        Self {
            current_reservation: None
        }
    }
}

pub fn find_bot_jobs(
    mut reservations: ResMut<ReservationSystem>,
    grid: Res<Grid>,
    impassable: Query<Entity, With<Impassable>>,
    scrap: Query<Entity, With<Scrap>>,
    mut bots: Query<(Entity, &mut Bot, &Position)>,
) {
    let impassable_set: std::collections::HashSet<Entity> = impassable.iter().collect();

    for (bot_entity, mut bot, bot_position) in bots.iter_mut() {
        if bot.current_reservation.is_none() {
            let reachable_entities = grid.flood_search(bot_position, &impassable_set);

            let reachable_scrap: Vec<Entity> = reachable_entities
                .into_iter()
                .filter(|entity| {
                  return scrap.contains(*entity);
                })
                .collect();

            for scrap_entity in reachable_scrap {
                let key = ReservationKey::Entity(scrap_entity);

                if reservations.is_reserved(&key) {
                    continue;
                }

                if reservations.try_reserve(key.clone(), bot_entity) {
                  info!("Bot {:?} reserved scrap {:?}. Key: {:?}", bot_entity, scrap_entity, key);
                    bot.current_reservation = Some(key);
                    break;
                }
            }
        }
    }
}
