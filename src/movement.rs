use bevy::prelude::*;
use crate::grid::Position;
use crate::pathfinding::Path;

pub fn move_along_path(
  mut paths: Query<(Entity, &mut Path, &mut Position)>,
  mut commands: Commands,
) {
  for (entity, mut path, mut position) in paths.iter_mut() {
    if path.path.is_empty() {
      continue;
    }

    let next_position = path.path[0];
    position.x = next_position.x;
    position.y = next_position.y;
    path.path.remove(0);

    if path.path.is_empty() {
      commands.entity(entity).remove::<Path>();
    }
  }
}