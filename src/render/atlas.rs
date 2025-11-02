use bevy::prelude::*;

#[derive(Resource, Clone)]
pub struct TileAtlas {
    pub image: Handle<Image>,
    pub cols: u32,
    pub rows: u32,
}

impl TileAtlas {
    pub fn uv_rect(&self, tile_id: u32) -> [Vec2; 4] {
        let x = tile_id % self.cols;
        let y = tile_id / self.cols;
        let u0 = x as f32 / self.cols as f32;
        let v0 = y as f32 / self.rows as f32;
        let u1 = (x+1) as f32 / self.cols as f32;
        let v1 = (y+1) as f32 / self.rows as f32;
        // (u0,v1) (u1,v1) (u1,v0) (u0,v0)
        [Vec2::new(u0,v1), Vec2::new(u1,v1), Vec2::new(u1,v0), Vec2::new(u0,v0)]
    }
}
