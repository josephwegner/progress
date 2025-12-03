use bevy::prelude::*;
use crate::grid::{Position, Grid, Impassable};
use crate::entities::scrap::Scrap;
use crate::reservation::{ReservationSystem, ReservationKey};
use crate::pathfinding::{Path, distance};
use crate::interact::Interaction;

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

pub fn work(
  bots: Query<(Entity, &Bot, &Position, Option<&Path>, Option<&Interaction>)>,
  scrap: Query<&Position, With<Scrap>>,
  mut commands: Commands
) {
  for (bot_entity, bot, bot_position, path, interaction) in bots.iter() {
    let Some(reservation_key) = &bot.current_reservation else {
      continue;
    };

    match reservation_key {
      ReservationKey::Tile(_tile_pos) => {
        warn!("Bot {:?} has a tile reservation {:?}. This should not happen.", bot_entity, reservation_key);
      },
      ReservationKey::Entity(scrap_entity) => {
        if let Ok(scrap_position) = scrap.get(*scrap_entity) {
          work_on_scrap(&mut commands, bot_entity, bot_position, scrap_position, *scrap_entity, path, interaction);
        } else {
          warn!("Bot {:?} has a non-scrap reservation {:?}. This should not happen.", bot_entity, reservation_key);
        }
      },
    }
  }
}

fn work_on_scrap(
  commands: &mut Commands,
  bot_entity: Entity,
  bot_position: &Position,
  scrap_position: &Position,
  scrap_entity: Entity,
  bot_path: Option<&Path>,
  interaction: Option<&Interaction>
) {
  if distance(bot_position, scrap_position) <= 1.0 {
    if interaction.is_none() {
      info!("Bot {:?} is ready to mine scrap {:?}", bot_entity, scrap_entity);
      commands.entity(bot_entity).insert(Interaction::new(bot_entity, scrap_entity, 50));
    } else if interaction.unwrap().completed {
      info!("Bot {:?} has completed interaction with scrap {:?}", bot_entity, scrap_entity);
      commands.entity(bot_entity).remove::<Interaction>();
      commands.entity(bot_entity).remove::<Path>();
      commands.entity(scrap_entity).despawn();
    }
  } else if bot_path.is_none() {
    commands.entity(bot_entity).insert(Path::new(*scrap_position));
  }
}