use bevy::prelude::*;
use crate::sim::game_state::GameTime;

const NOTIFICATION_DURATION_INFO: f32 = 5.0;
const NOTIFICATION_DURATION_WARNING: f32 = 10.0;
const NOTIFICATION_DURATION_CRITICAL: f32 = 15.0;

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum NotificationSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Component, Clone)]
pub struct Notification {
    pub message: String,
    pub severity: NotificationSeverity,
    pub timer: Timer,
}

impl Notification {
    pub fn new(message: String, severity: NotificationSeverity) -> Self {
        let duration = match severity {
            NotificationSeverity::Info => NOTIFICATION_DURATION_INFO,
            NotificationSeverity::Warning => NOTIFICATION_DURATION_WARNING,
            NotificationSeverity::Critical => NOTIFICATION_DURATION_CRITICAL,
        };
        Self {
            message,
            severity,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// Entry in permanent notification history
#[derive(Clone)]
pub struct NotificationEntry {
    pub message: String,
    pub severity: NotificationSeverity,
    pub timestamp: f64,
}

/// Permanent history of all notifications
#[derive(Resource, Default)]
pub struct NotificationHistory {
    pub entries: Vec<NotificationEntry>,
}

/// Marker component for the notification text display
#[derive(Component)]
pub struct NotificationContainer;

/// Marker component for the notification background node
#[derive(Component)]
pub struct NotificationBackground;

/// System: Setup notification UI container in top-left corner
/// Should be called in Startup schedule
pub fn setup_notification_ui(mut commands: Commands) {
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                top: Val::Px(250.0),
                padding: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            background_color: Color::srgba(0.0, 0.0, 0.0, 0.7).into(),
            ..default()
        },
        NotificationBackground,
    ))
    .with_children(|parent| {
        parent.spawn((
            TextBundle::from_section(
                "",
                TextStyle {
                    font_size: 16.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            NotificationContainer,
        ));
    });
}

/// System: Tick notification timers, despawn expired ones, add to history
/// Should run in Update schedule
pub fn tick_notification_timers(
    time: Res<Time>,
    mut game_time: ResMut<GameTime>,
    mut notifications: Query<(Entity, &mut Notification)>,
    mut notification_background: Query<&mut BackgroundColor, With<NotificationBackground>>,
    mut history: ResMut<NotificationHistory>,
    mut commands: Commands,
) {
    game_time.elapsed_seconds += time.delta_seconds() as f64;
    for (entity, mut notification) in notifications.iter_mut() {
        notification.timer.tick(time.delta());
        if notification.timer.just_finished() {
            history.entries.push(NotificationEntry {
                message: notification.message.clone(),
                severity: notification.severity,
                timestamp: game_time.elapsed_seconds,
            });
            commands.entity(entity).despawn_recursive();
        }
    }

    // Hide background if no notifications are active
    if let Ok(mut bg_color) = notification_background.get_single_mut() {
        if notifications.is_empty() {
            *bg_color = Color::srgba(0.0, 0.0, 0.0, 0.0).into();
        } else {
            *bg_color = Color::srgba(0.0, 0.0, 0.0, 0.7).into();
        }
    }
}

/// System: Update notification display text
/// Should run in Update schedule after tick_notification_timers
pub fn update_notification_display(
    notifications: Query<&Notification>,
    mut container: Query<&mut Text, With<NotificationContainer>>,
) {
    let mut notifications_vec = notifications.iter().collect::<Vec<_>>();
    notifications_vec.sort_by(|a, b| b.timer.elapsed_secs().partial_cmp(&a.timer.elapsed_secs()).unwrap());

    let mut display_text = String::new();
    for notification in notifications_vec.iter() {
        let severity_text = match notification.severity {
            NotificationSeverity::Info => "INFO",
            NotificationSeverity::Warning => "WARN",
            NotificationSeverity::Critical => "CRIT",
        };
        display_text.push_str(&format!("[{}] {}\n", severity_text, notification.message));
    }

    if let Ok(mut notification_container) = container.get_single_mut() {
        notification_container.sections[0].value = display_text;
    }
}
