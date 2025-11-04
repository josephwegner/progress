use bevy::prelude::*;
use crate::sim::resources::GameResources;
use crate::sim::buildings::BuildMode;
use crate::sim::game_state::GameTime;

#[derive(Component)]
pub struct ResourcesText;

#[derive(Component)]
pub struct ControlsText;

pub fn setup_hud(mut commands: Commands) {
    info!("Setting up HUD");
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        padding: UiRect::all(Val::Px(5.0)),
                        ..default()
                    },
                    background_color: Color::srgba(0.0, 0.0, 0.0, 0.7).into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle::from_section(
                            "Resources Loading...",
                            TextStyle {
                                font_size: 18.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        ),
                        ResourcesText,
                    ));
                });

            parent
                .spawn(NodeBundle {
                    style: Style {
                        padding: UiRect::all(Val::Px(5.0)),
                        ..default()
                    },
                    background_color: Color::srgba(0.0, 0.0, 0.0, 0.7).into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle::from_section(
                            "Controls:\n\
                            WASD - Pan Camera\n\
                            Mouse Wheel - Zoom\n\
                            1 - Scavenge Tool (paint resources)\n\
                            2 - Stockpile Tool\n\
                            3 - Build Server Rack (20 scrap)\n\
                            4 - Build Power Node (15 scrap)\n\
                            5 - Build Bot (50 scrap)\n\
                            ESC - Cancel Build Mode",
                            TextStyle {
                                font_size: 14.0,
                                color: Color::srgb(0.8, 0.8, 0.8),
                                ..default()
                            },
                        ),
                        ControlsText,
                    ));
                });
        });
}

pub fn update_hud(
    resources: Res<GameResources>,
    build_mode: Res<BuildMode>,
    game_time: Res<GameTime>,
    mut text_query: Query<&mut Text, With<ResourcesText>>,
) {
    for mut text in text_query.iter_mut() {
        let mode_str = match *build_mode {
            BuildMode::None => "None",
            BuildMode::ServerRack => "Server Rack",
            BuildMode::PowerNode => "Power Node",
            BuildMode::Bot => "Bot",
        };

        let net_power = resources.power_produced - resources.power_consumed;
        let power_status = if net_power > 0 {
            format!("(+{}/tick)", net_power)
        } else if net_power < 0 {
            format!("({}/tick)", net_power)
        } else {
            "(balanced)".to_string()
        };

        let minutes = (game_time.elapsed_seconds as u32) / 60;
        let seconds = (game_time.elapsed_seconds as u32) % 60;
        text.sections[0].value = format!(
            "Game Time: {:02}:{:02}\n\
             ┌─ Resources ─────────────────┐\n\
             │ Scrap: {}\n\
             │\n\
             │ Power: {}/{} {}\n\
             │   Production: {}/tick\n\
             │   Consumption: {}/tick\n\
             │\n\
             │ Compute: {}/{}\n\
             └─────────────────────────────┘\n\
             \n\
             Build Mode: {}",
            minutes, seconds,
            resources.scrap,
            resources.power_stored,
            resources.power_cap,
            power_status,
            resources.power_produced,
            resources.power_consumed,
            resources.compute_cycles,
            resources.compute_cap,
            mode_str,
        );
    }
}
