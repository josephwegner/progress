use bevy::prelude::*;
use crate::grid::{Position, Grid};

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
  mut query: Query<(Entity, &mut Renderable, &Position), Added<Renderable>>
) {
  for (_entity, mut renderable, position) in query.iter_mut() {
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