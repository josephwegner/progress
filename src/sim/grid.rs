use bevy::prelude::*;

pub const MAP_W: u32 = 64;
pub const MAP_H: u32 = 64;
pub const TILE_SIZE: f32 = 16.0; // world units

#[repr(u8)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum TileKind {
    #[default]
    Ground,
    Stockpile,
    Scavenge,
    Wall,
}

#[derive(Resource)]
pub struct WorldGrid {
    pub w: u32,
    pub h: u32,
    pub tiles: Vec<TileKind>,
    pub chunk_w: u32,
    pub chunk_h: u32,
    pub chunk_cols: u32,
    pub chunk_rows: u32,
    pub chunk_dirty: Vec<bool>,
}

impl WorldGrid {
    pub fn new(w:u32,h:u32, chunk_w:u32, chunk_h:u32) -> Self {
        let chunk_cols = (w + chunk_w - 1)/chunk_w;
        let chunk_rows = (h + chunk_h - 1)/chunk_h;
        let tiles = vec![TileKind::Ground; (w*h) as usize];

        Self {
            w,h,
            tiles,
            chunk_w, chunk_h,
            chunk_cols, chunk_rows,
            chunk_dirty: vec![true; (chunk_cols*chunk_rows) as usize],
        }
    }
    pub fn idx(&self, x:u32,y:u32)->usize { (y*self.w + x) as usize }
    pub fn mark_chunk_dirty(&mut self, x:u32,y:u32) {
        let cx = x / self.chunk_w;
        let cy = y / self.chunk_h;
        let cidx = (cy*self.chunk_cols + cx) as usize;
        self.chunk_dirty[cidx] = true;
    }
}
