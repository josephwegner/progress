use bevy::prelude::*;
use crate::grid::{Position, Grid};

#[derive(Component)]
pub struct Renderable {
  color: Color
}

impl Renderable {
  pub fn new(r: f32, g: f32, b: f32) -> Self {
    Self { color: Color::srgb(r, g, b) }
  }
}

pub fn draw_renderable(mut commands: Commands, grid: Res<Grid>, query: Query<(&Renderable, &Position)>) {
  for (renderable, position) in query.iter() {
    let world_x = (((position.x - 1) as f32 - grid.width as f32 / 2.0) + 0.5) * grid.tile_size;
    let world_y = (((grid.height as f32 / 2.0) - position.y as f32) + 0.5) * grid.tile_size;

    commands.spawn(SpriteBundle {
      sprite: Sprite {
        color: renderable.color,
        custom_size: Some(Vec2::new(grid.tile_size, grid.tile_size)),
        ..default()
      },
      transform: Transform::from_xyz(world_x, world_y, 0.0),
      ..default()
    });
  }
}