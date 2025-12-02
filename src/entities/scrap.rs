use bevy::prelude::*;

#[derive(Component)]
pub struct Scrap {
    pub size: u32
}

impl Scrap {
    pub fn new(size: u32) -> Self {
        Self { size }
    }
}
