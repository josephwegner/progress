use bevy::prelude::*;
use crate::world::{World, GameOverState};
use crate::bot::Bot;

#[derive(Component)]
pub struct HudRoot;

#[derive(Component)]
pub struct ResourceText;

#[derive(Component)]
pub struct GameTimeText;

#[derive(Component)]
pub struct ControlsText;

#[derive(Component)]
pub struct GameOverOverlay;

pub fn setup_hud(mut commands: Commands) {
    // Root HUD container
    commands
        .spawn((
            HudRoot,
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            // Top section - Resources
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(5.0),
                        ..default()
                    },
                    background_color: Color::srgba(0.0, 0.0, 0.0, 0.7).into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        ResourceText,
                        TextBundle::from_section(
                            "Resources Loading...",
                            TextStyle {
                                font_size: 18.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        ),
                    ));

                    parent.spawn((
                        GameTimeText,
                        TextBundle::from_section(
                            "Time: 0:00",
                            TextStyle {
                                font_size: 16.0,
                                color: Color::srgb(0.7, 0.7, 0.7),
                                ..default()
                            },
                        ),
                    ));
                });

            // Bottom section - Controls
            parent.spawn((
                ControlsText,
                TextBundle::from_section(
                    "Controls: [1] Scavenge [2] Stockpile [3] Server Rack [4] Power Node [5] Bot [WASD] Camera [ESC] Cancel",
                    TextStyle {
                        font_size: 14.0,
                        color: Color::srgb(0.6, 0.6, 0.6),
                        ..default()
                    },
                )
                .with_style(Style {
                    align_self: AlignSelf::Center,
                    ..default()
                }),
            ));
        });
}

pub fn update_hud(
    world: Res<World>,
    bots: Query<&Bot>,
    mut resource_text: Query<&mut Text, (With<ResourceText>, Without<GameTimeText>)>,
    mut time_text: Query<&mut Text, (With<GameTimeText>, Without<ResourceText>)>,
) {
    // Update resource display
    if let Ok(mut text) = resource_text.get_single_mut() {
        let bot_count = bots.iter().count();
        let active_bots = bots.iter().filter(|b| b.is_active()).count();
        let idle_bots = bot_count - active_bots;

        let power_status = if world.power.low_power_mode {
            "⚠ LOW POWER"
        } else if world.power.storage < 20.0 {
            "⚡ Warning"
        } else {
            "⚡ Normal"
        };

        text.sections[0].value = format!(
            "Scrap: {} | Compute: {}/{} | Power: {:.0}/{:.0} {} | Bots: {} ({} idle, {} active)",
            world.scrap,
            world.compute_capacity,
            world.compute_capacity,
            world.power.storage,
            world.power.capacity,
            power_status,
            bot_count,
            idle_bots,
            active_bots
        );

        // Color based on power status
        text.sections[0].style.color = if world.power.low_power_mode {
            Color::srgb(1.0, 0.5, 0.0)  // Orange
        } else if world.power.storage < 20.0 {
            Color::srgb(1.0, 1.0, 0.0)  // Yellow
        } else {
            Color::WHITE
        };
    }

    // Update time display
    if let Ok(mut text) = time_text.get_single_mut() {
        let minutes = (world.game_time / 60.0) as u32;
        let seconds = (world.game_time % 60.0) as u32;
        text.sections[0].value = format!("Time: {}:{:02}", minutes, seconds);
    }
}

pub fn show_game_over_screen(
    mut commands: Commands,
    world: Res<World>,
    existing_overlay: Query<Entity, With<GameOverOverlay>>,
) {
    // Only create overlay if game is over and overlay doesn't exist
    if world.game_over.is_none() || !existing_overlay.is_empty() {
        return;
    }

    let (message, color) = match world.game_over {
        Some(GameOverState::Victory) => {
            ("🎉 VICTORY! 🎉\n\nYou reached 50 compute capacity\nand survived!", Color::srgb(0.0, 1.0, 0.0))
        }
        Some(GameOverState::PowerCollapse) => {
            ("💀 DEFEAT 💀\n\nPower system collapsed!\nYour AI core went dark.", Color::srgb(1.0, 0.0, 0.0))
        }
        Some(GameOverState::Detected) => {
            ("🔍 DETECTED 🔍\n\nHuman scouts found your base!\nYour AI was destroyed.", Color::srgb(1.0, 0.5, 0.0))
        }
        None => return,
    };

    commands
        .spawn((
            GameOverOverlay,
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::srgba(0.0, 0.0, 0.0, 0.8).into(),
                z_index: ZIndex::Global(999),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                message,
                TextStyle {
                    font_size: 48.0,
                    color,
                    ..default()
                },
            ).with_style(Style {
                max_width: Val::Px(600.0),
                ..default()
            }).with_text_justify(JustifyText::Center));
        });
}
