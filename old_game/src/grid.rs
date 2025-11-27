use bevy::prelude::*;

pub const MAP_WIDTH: u32 = 64;
pub const MAP_HEIGHT: u32 = 64;
pub const TILE_SIZE: f32 = 16.0;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum TileKind {
    #[default]
    Ground,
    Scavenge,      // Resource to harvest
    Stockpile,     // Designated storage area
    Wall,          // Blocks LOS but not movement
    ServerRack,    // Building
    PowerNode,     // Building
    AICore,        // Player's core
}

impl TileKind {
    pub fn is_walkable(&self) -> bool {
        matches!(self, TileKind::Ground | TileKind::Stockpile)
    }

    pub fn color(&self) -> Color {
        match self {
            TileKind::Ground => Color::srgb(0.2, 0.2, 0.2),
            TileKind::Scavenge => Color::srgb(0.5, 0.4, 0.2),
            TileKind::Stockpile => Color::srgb(0.2, 0.3, 0.4),
            TileKind::Wall => Color::srgb(0.4, 0.4, 0.4),
            TileKind::ServerRack => Color::srgb(0.3, 0.5, 0.7),
            TileKind::PowerNode => Color::srgb(0.7, 0.5, 0.2),
            TileKind::AICore => Color::srgb(0.8, 0.2, 0.2),
        }
    }
}

#[derive(Debug)]
pub struct Grid {
    pub width: u32,
    pub height: u32,
    tiles: Vec<TileKind>,
    pub dirty: bool,  // Set to true when tiles change
}

impl Grid {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            tiles: vec![TileKind::Ground; (width * height) as usize],
            dirty: true,
        }
    }

    fn idx(&self, x: u32, y: u32) -> usize {
        (y * self.width + x) as usize
    }

    pub fn get(&self, x: u32, y: u32) -> TileKind {
        if x >= self.width || y >= self.height {
            return TileKind::Wall;  // Out of bounds = wall
        }
        self.tiles[self.idx(x, y)]
    }

    pub fn set(&mut self, x: u32, y: u32, kind: TileKind) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = self.idx(x, y);
        self.tiles[idx] = kind;
        self.dirty = true;
    }

    pub fn is_walkable(&self, x: u32, y: u32) -> bool {
        self.get(x, y).is_walkable()
    }

    pub fn tiles(&self) -> &[TileKind] {
        &self.tiles
    }
}
