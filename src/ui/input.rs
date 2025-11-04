use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::input::mouse::MouseWheel;
use crate::sim::grid::{WorldGrid, TILE_SIZE, TileKind};
use bevy::render::camera::RenderTarget;

#[derive(Resource, Default, Copy, Clone, Eq, PartialEq, Debug)]
pub enum PaintTool { #[default] Scavenge, Stockpile }

#[derive(Component)]
pub struct MainCamera;

pub fn setup_camera(mut cmds: Commands, grid: Res<WorldGrid>) {
    let world_center_x = (grid.w as f32 / 2.0) * TILE_SIZE;
    let world_center_y = (grid.h as f32 / 2.0) * TILE_SIZE;

    let mut cam = Camera2dBundle::default();
    cam.projection.scale = 0.5; // zoom in to see tiles properly
    cam.transform.translation.x = world_center_x;
    cam.transform.translation.y = world_center_y;
    cmds.spawn((cam, MainCamera));
}

pub fn camera_controls(
    mut q_cam: Query<(&mut Transform, &mut OrthographicProjection), With<MainCamera>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut scroll: EventReader<MouseWheel>,
    time: Res<Time>,
) {
    let (mut t, mut proj) = q_cam.single_mut();
    let dt = time.delta_seconds();
    let speed = 500.0 * proj.scale; // scale-aware pan

    if keys.pressed(KeyCode::KeyA) { t.translation.x -= speed * dt; }
    if keys.pressed(KeyCode::KeyD) { t.translation.x += speed * dt; }
    if keys.pressed(KeyCode::KeyW) { t.translation.y += speed * dt; }
    if keys.pressed(KeyCode::KeyS) { t.translation.y -= speed * dt; }

    for ev in scroll.read() {
        let factor = if ev.y > 0.0 { 0.9 } else { 1.1 };
        proj.scale = (proj.scale * factor).clamp(0.5, 4.0);
    }
}

pub fn switch_tools(
    keys: Res<ButtonInput<KeyCode>>,
    mut tool: ResMut<PaintTool>,
    mut build_mode: ResMut<crate::sim::buildings::BuildMode>,
) {
    if keys.just_pressed(KeyCode::Digit1) {
        *tool = PaintTool::Scavenge;
        *build_mode = crate::sim::buildings::BuildMode::None;
    }
    if keys.just_pressed(KeyCode::Digit2) {
        *tool = PaintTool::Stockpile;
        *build_mode = crate::sim::buildings::BuildMode::None;
    }
}

pub fn paint_brush(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    q_primary: Query<&Window, With<PrimaryWindow>>,
    q_cam: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut grid: ResMut<WorldGrid>,
    mut job_queue: ResMut<crate::sim::jobs::JobQueue>,
    tool: Res<PaintTool>,
    build_mode: Res<crate::sim::buildings::BuildMode>,
) {
    if !buttons.pressed(MouseButton::Left) { return; }
    if *build_mode != crate::sim::buildings::BuildMode::None { return; }

    let Ok((camera, cam_xform)) = q_cam.get_single() else { return; };

    let window = match camera.target {
        RenderTarget::Window(id) => match id {
            bevy::window::WindowRef::Primary => q_primary.get_single().ok(),
            bevy::window::WindowRef::Entity(entity) => windows.get(entity).ok(),
        },
        _ => None,
    };
    let Some(window) = window else { return; };
    let Some(cursor) = window.cursor_position() else { return; };

    if let Some(ray) = camera.viewport_to_world(cam_xform, cursor) {
        let world = ray.origin.truncate();
        if world.x < 0.0 || world.y < 0.0 { return; }
        let gx = (world.x / TILE_SIZE).floor() as u32;
        let gy = (world.y / TILE_SIZE).floor() as u32;
        if gx >= grid.w || gy >= grid.h { return; }

        let kind = match *tool {
            PaintTool::Scavenge  => TileKind::Scavenge,
            PaintTool::Stockpile => TileKind::Stockpile,
        };
        let idx = grid.idx(gx, gy);

        // Only paint if tile changed
        if grid.tiles[idx] != kind {
            grid.tiles[idx] = kind;
            grid.mark_chunk_dirty(gx, gy);

            // Create job immediately when painting Scavenge tile
            if kind == TileKind::Scavenge {
                job_queue.push(
                    crate::sim::jobs::JobType::Scavenge { x: gx, y: gy },
                    10,
                    &mut commands,
                );
            }
        }
    }
}
