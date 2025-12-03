use bevy::prelude::*;

#[derive(Component)]
pub struct Interaction {
  pub actor: Entity,
  pub target: Entity,
  pub ticks_to_complete: u32,
  pub ticks_completed: u32,
  pub completed: bool,
  pub progress_bar_entity: Option<Entity>,
}

impl Interaction {
  pub fn new(actor: Entity, target: Entity, ticks_to_complete: u32) -> Self {
    Self {
      actor,
      target,
      ticks_to_complete,
      ticks_completed: 0,
      completed: false,
      progress_bar_entity: None,
    }
  }
}

pub fn update_interactions(
  mut interactions: Query<&mut Interaction>,
) {
  for mut interaction in interactions.iter_mut() {
    interaction.ticks_completed += 1;
    if interaction.ticks_completed >= interaction.ticks_to_complete {
      interaction.completed = true;
      interaction.ticks_completed = interaction.ticks_to_complete;
    }
    info!("Updating interaction for actor {:?}, ticks completed: {:?}, max: {:?}", interaction.actor, interaction.ticks_completed, interaction.ticks_to_complete);
  }
}