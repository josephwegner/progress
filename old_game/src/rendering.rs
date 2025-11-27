use bevy::prelude::*;
use crate::grid::TILE_SIZE;
use crate::world::World;
use crate::bot::Bot;

#[derive(Component)]
pub struct TileSprite {
    pub x: u32,
    pub y: u32,
}

pub fn setup_rendering(
    mut commands: Commands,
    world: Res<World>,
) {
    // Spawn tile sprites for the entire grid
    for y in 0..world.grid.height {
        for x in 0..world.grid.width {
            let tile_kind = world.grid.get(x, y);

            commands.spawn((
                TileSprite { x, y },
                SpriteBundle {
                    sprite: Sprite {
                        color: tile_kind.color(),
                        custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                        ..default()
                    },
                    transform: Transform::from_xyz(
                        x as f32 * TILE_SIZE,
                        y as f32 * TILE_SIZE,
                        0.0,
                    ),
                    ..default()
                },
            ));
        }
    }

    info!("Rendered {} tiles", world.grid.width * world.grid.height);
}

pub fn update_tile_rendering(
    mut tiles: Query<(&TileSprite, &mut Sprite)>,
    mut world: ResMut<World>,
) {
    if !world.grid.dirty {
        return;
    }

    // Update tile colors when grid changes
    for (tile_sprite, mut sprite) in tiles.iter_mut() {
        let tile_kind = world.grid.get(tile_sprite.x, tile_sprite.y);
        sprite.color = tile_kind.color();
    }

    world.grid.dirty = false;
}

pub fn update_bot_sprites(
    mut bots: Query<(&Bot, &mut Transform, &mut Sprite)>,
    world: Res<World>,
) {
    for (bot, mut transform, mut sprite) in bots.iter_mut() {
        // Update position
        transform.translation.x = bot.position.0 as f32 * TILE_SIZE;
        transform.translation.y = bot.position.1 as f32 * TILE_SIZE;

        // Update color based on state
        sprite.color = if bot.jammed {
            Color::srgb(0.0, 1.0, 0.0).with_alpha(0.5)  // Green when jammed
        } else if world.power.low_power_mode {
            Color::srgb(0.9, 0.5, 0.2)  // Orange in low power
        } else {
            Color::srgb(0.9, 0.9, 0.2)  // Yellow normal
        };
    }
}
