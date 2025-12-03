use bevy::prelude::*;
use std::collections::HashMap;
use crate::grid::{Position, Grid};
use crate::interact::Interaction;

#[derive(Resource, Default)]
pub struct SpriteMapping {
  entity_to_sprite: HashMap<Entity, Entity>
}

#[derive(Component)]
pub struct Renderable {
  color: Color,
  sprite_entity: Option<Entity>
}

impl Renderable {
  pub fn new(r: f32, g: f32, b: f32) -> Self {
    Self {
      color: Color::srgb(r, g, b),
      sprite_entity: None
    }
  }
}

pub fn spawn_sprites_for_new_renderables(
  mut commands: Commands,
  grid: Res<Grid>,
  mut sprite_mapping: ResMut<SpriteMapping>,
  mut query: Query<(Entity, &mut Renderable, &Position), Added<Renderable>>
) {
  for (entity, mut renderable, position) in query.iter_mut() {
    let world_x = ((position.x as f32 - grid.width as f32 / 2.0) + 0.5) * grid.tile_size;
    let world_y = ((position.y as f32 - grid.height as f32 / 2.0) + 0.5) * grid.tile_size;

    let sprite_entity = commands.spawn(SpriteBundle {
      sprite: Sprite {
        color: renderable.color,
        custom_size: Some(Vec2::new(grid.tile_size, grid.tile_size)),
        ..default()
      },
      transform: Transform::from_xyz(world_x, world_y, 1.0),
      ..default()
    }).id();

    renderable.sprite_entity = Some(sprite_entity);
    sprite_mapping.entity_to_sprite.insert(entity, sprite_entity);
  }
}

pub fn update_sprite_positions(
  grid: Res<Grid>,
  query: Query<(&Renderable, &Position), Changed<Position>>,
  mut transforms: Query<&mut Transform>
) {
  for (renderable, position) in query.iter() {
    if let Some(sprite_entity) = renderable.sprite_entity {
      if let Ok(mut transform) = transforms.get_mut(sprite_entity) {
        let world_x = ((position.x as f32 - grid.width as f32 / 2.0) + 0.5) * grid.tile_size;
        let world_y = ((position.y as f32 - grid.height as f32 / 2.0) + 0.5) * grid.tile_size;

        transform.translation.x = world_x;
        transform.translation.y = world_y;
      }
    }
  }
}

pub fn cleanup_despawned_sprites(
  mut commands: Commands,
  mut removed: RemovedComponents<Renderable>,
  mut sprite_mapping: ResMut<SpriteMapping>
) {
  for entity in removed.read() {
    if let Some(sprite_entity) = sprite_mapping.entity_to_sprite.remove(&entity) {
      commands.entity(sprite_entity).despawn();
    }
  }
}

pub fn draw_interaction_progress_bars(
  mut commands: Commands,
  grid: Res<Grid>,
  mut interactions: Query<&mut Interaction>,
  positions: Query<&Position>,
  mut sprites: Query<&mut Sprite>,
  mut transforms: Query<&mut Transform>,
) {
  for mut interaction in interactions.iter_mut() {
    if interaction.completed {
      if let Some(bar_entity) = interaction.progress_bar_entity {
        commands.entity(bar_entity).despawn();
        interaction.progress_bar_entity = None;
      }
      continue;
    }

    if let Ok(position) = positions.get(interaction.actor) {
      let progress = interaction.ticks_completed as f32 / interaction.ticks_to_complete as f32;

      let world_x = ((position.x as f32 - grid.width as f32 / 2.0) + 0.5) * grid.tile_size;
      let world_y = ((position.y as f32 - grid.height as f32 / 2.0) + 0.5) * grid.tile_size;

      if let Some(bar_entity) = interaction.progress_bar_entity {
        if let Ok(mut sprite) = sprites.get_mut(bar_entity) {
          sprite.custom_size = Some(Vec2::new(grid.tile_size * progress, 5.0));
        }
        if let Ok(mut transform) = transforms.get_mut(bar_entity) {
          transform.translation.x = world_x;
          transform.translation.y = world_y + grid.tile_size * 0.6;
        }
      } else {
        let bar_entity = commands.spawn(SpriteBundle {
          sprite: Sprite {
            color: Color::srgb(0.0, 1.0, 0.0),
            custom_size: Some(Vec2::new(grid.tile_size * progress, 5.0)),
            ..default()
          },
          transform: Transform::from_xyz(world_x, world_y + grid.tile_size * 0.6, 2.0),
          ..default()
        }).id();

        interaction.progress_bar_entity = Some(bar_entity);
      }
    }
  }
}