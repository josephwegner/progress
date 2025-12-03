use bevy::prelude::*;
use crate::grid::{Position, Grid, Impassable};
use std::collections::{HashMap, BinaryHeap};
use std::cmp::Ordering;

#[derive(Component)]
pub struct Path {
  pub target: Position,
  pub path: Vec<Position>
}

impl Path {
  pub fn new(target: Position) -> Self {
    Self { target, path: Vec::new() }
  }
}

pub fn distance(position: &Position, target: &Position) -> f32 {
  let dx = position.x as f32 - target.x as f32;
  let dy = position.y as f32 - target.y as f32;
  (dx * dx + dy * dy).sqrt()
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct State {
  cost: u32,
  position: Position,
}

impl Ord for State {
  fn cmp(&self, other: &Self) -> Ordering {
    other.cost.cmp(&self.cost)
  }
}

impl PartialOrd for State {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

fn heuristic(a: &Position, b: &Position) -> u32 {
  let dx = (a.x as i32 - b.x as i32).abs() as u32;
  let dy = (a.y as i32 - b.y as i32).abs() as u32;
  dx + dy
}

fn reconstruct_path(came_from: &HashMap<Position, Position>, mut current: Position) -> Vec<Position> {
  let mut path = vec![current];
  while let Some(&prev) = came_from.get(&current) {
    path.push(prev);
    current = prev;
  }
  path.reverse();
  path
}

fn is_tile_passable(position: &Position, grid: &Grid, impassable_entities: &std::collections::HashSet<Entity>) -> bool {
  let idx = position.index();
  if let Some(Some(tile)) = grid.tiles.get(idx) {
    !tile.residents.iter().any(|entity| impassable_entities.contains(entity))
  } else {
    false
  }
}

fn find_passable_adjacent_tile(goal: &Position, grid: &Grid, impassable_entities: &std::collections::HashSet<Entity>) -> Option<Position> {
  let directions = [
    (goal.x.wrapping_sub(1), goal.y),
    (goal.x + 1, goal.y),
    (goal.x, goal.y.wrapping_sub(1)),
    (goal.x, goal.y + 1),
  ];

  for (nx, ny) in directions {
    if nx >= grid.width || ny >= grid.height {
      continue;
    }

    let neighbor = Position::new(nx, ny);
    if is_tile_passable(&neighbor, grid, impassable_entities) {
      return Some(neighbor);
    }
  }

  None
}

fn astar(start: &Position, goal: &Position, grid: &Grid, impassable_entities: &std::collections::HashSet<Entity>) -> Option<Vec<Position>> {
  let actual_goal = if !is_tile_passable(goal, grid, impassable_entities) {
    if let Some(adjacent_goal) = find_passable_adjacent_tile(goal, grid, impassable_entities) {
      adjacent_goal
    } else {
      return None;
    }
  } else {
    *goal
  };

  let mut open_set = BinaryHeap::new();
  let mut came_from = HashMap::new();
  let mut g_score = HashMap::new();

  g_score.insert(*start, 0u32);
  open_set.push(State {
    cost: heuristic(start, &actual_goal),
    position: *start,
  });

  while let Some(State { cost: _, position: current }) = open_set.pop() {
    if current == actual_goal {
      return Some(reconstruct_path(&came_from, current));
    }

    let current_g = *g_score.get(&current).unwrap_or(&u32::MAX);

    let directions = [
      (current.x.wrapping_sub(1), current.y),
      (current.x + 1, current.y),
      (current.x, current.y.wrapping_sub(1)),
      (current.x, current.y + 1),
    ];

    for (nx, ny) in directions {
      if nx >= grid.width || ny >= grid.height {
        continue;
      }

      let neighbor = Position::new(nx, ny);

      if !is_tile_passable(&neighbor, grid, impassable_entities) {
        continue;
      }

      let tentative_g = current_g + 1;
      let neighbor_g = *g_score.get(&neighbor).unwrap_or(&u32::MAX);

      if tentative_g < neighbor_g {
        came_from.insert(neighbor, current);
        g_score.insert(neighbor, tentative_g);
        let f_score = tentative_g + heuristic(&neighbor, &actual_goal);
        open_set.push(State {
          cost: f_score,
          position: neighbor,
        });
      }
    }
  }

  None
}

pub fn pathfind(
  grid: Res<Grid>,
  impassable: Query<Entity, With<Impassable>>,
  mut paths: Query<(&mut Path, &Position)>,
) {
  let impassable_set: std::collections::HashSet<Entity> = impassable.iter().collect();

  for (mut path, current_position) in paths.iter_mut() {
    if !path.path.is_empty() {
      continue;
    }

    if distance(current_position, &path.target) <= 1.0 {
      continue;
    }

    if let Some(found_path) = astar(current_position, &path.target, &grid, &impassable_set) {
      path.path = found_path;
      info!("Path found, path: {:?}", path.path);
    } else {
      warn!("Failed to find path");
    }
  }
}