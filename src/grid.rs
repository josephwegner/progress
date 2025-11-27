use bevy::prelude::*;

pub const GRID_WIDTH: u32 = 20;
pub const GRID_HEIGHT: u32 = 20;
pub const TILE_SIZE: f32 = 32.0;

#[derive(Component, Clone)]
pub struct Position {
  pub x: u32,
  pub y: u32,
}

impl Position {
  pub fn new(x: u32, y: u32) -> Self {
    Self { x, y }
  }

  pub fn index(&self) -> usize {
    (self.y * GRID_WIDTH + self.x) as usize
  }
}

#[derive(Component, Clone)]
pub struct Resident {
  pub tile_position: Position,
}

impl Resident {
  pub fn new(tile_position: Position) -> Self {
    Self {
      tile_position
    }
  }
}

#[derive(Clone)]
pub struct Tile {
  pub position: Position,
  pub residents: Vec<Entity>,
}

impl Tile {
  pub fn new(x: u32, y: u32) -> Self {
    Self {
      position: Position::new(x, y), 
      residents: Vec::new()
    }
  }
}

#[derive(Resource)]
pub struct Grid {
  pub width: u32,
  pub height: u32,
  pub tile_size: f32,
  pub tiles: Vec<Option<Tile>>
}

impl Grid {
  pub fn new(width: u32, height: u32, tile_size: f32) -> Self {
    let capacity = (width * height) as usize;
    Self { width, height, tile_size, tiles: vec![None; capacity] }
  }

  fn index(&self, x: u32, y: u32) -> usize {
    (y * self.width + x) as usize
  }
}

pub fn setup_grid(mut commands: Commands) {
  let mut grid = Grid::new(GRID_WIDTH, GRID_HEIGHT, TILE_SIZE);

  for x in 0..GRID_WIDTH as u32 {
    for y in 0..GRID_HEIGHT as u32 {
      let tile = Tile::new(x, y);
      let idx = grid.index(x, y);
      grid.tiles[idx] = Some(tile);
    }
  }

  draw_tiles(&mut commands, &grid);
  commands.insert_resource(grid);
}

fn draw_tiles(commands: &mut Commands, grid: &Grid) {
  for tile in &grid.tiles {
    if let Some(tile) = tile {
      let world_x = ((tile.position.x as f32 - grid.width as f32 / 2.0) + 0.5) * grid.tile_size;
      let world_y = ((tile.position.y as f32 - grid.height as f32 / 2.0) + 0.5) * grid.tile_size;

      commands.spawn(SpriteBundle {
        sprite: Sprite {
          color: Color::srgb(0.2, 0.2, 0.2),
          custom_size: Some(Vec2::new(grid.tile_size, grid.tile_size)),
          ..default()
        },
        transform: Transform::from_xyz(world_x, world_y, 0.0),
        ..default()
      });
    }
  }
}

pub fn add_new_positions_as_residents(mut commands: Commands, mut grid: ResMut<Grid>, query: Query<(Entity, &Position), Added<Position>>) {
  for (entity, position) in query.iter() {
    let idx = position.index();
    if let Some(tile) = grid.tiles[idx].as_mut() {
      tile.residents.push(entity);
      commands.entity(entity).insert(Resident::new(tile.position.clone()));
    }
  }
}

pub fn update_residents(mut commands: Commands, mut grid: ResMut<Grid>, query: Query<(Entity, &Position, &Resident), Changed<Position>>) {
  for (entity, position, resident) in query.iter() {
    let old_idx = resident.tile_position.index();
    let new_idx = position.index();

    if let Some(old_tile) = grid.tiles[old_idx].as_mut() {
      old_tile.residents.retain(|&resident_entity| resident_entity != entity);
    }

    if let Some(new_tile) = grid.tiles[new_idx].as_mut() {
      new_tile.residents.push(entity);
      commands.entity(entity).insert(Resident::new(new_tile.position.clone()));
    }
  }
}