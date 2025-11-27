use bevy::prelude::*;
use crate::grid::{Position, Grid};
use crate::renderable::Renderable;

pub fn spawn_initial_components(mut commands: Commands, grid: Res<Grid>) {
  spawn_scrap(&mut commands, 5, 15);

  spawn_bot(&mut commands, 15, 5);
}

fn spawn_scrap(commands: &mut Commands, x: u32, y: u32) {
  commands.spawn((Renderable::new(0.2, 0.5, 0.5), Position::new(x, y)));
}

fn spawn_bot(commands: &mut Commands, x: u32, y: u32) {
  commands.spawn((Renderable::new(0.2, 0.2, 0.8), Position::new(x, y)));
}