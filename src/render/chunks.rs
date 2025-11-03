use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use crate::sim::grid::{WorldGrid, TILE_SIZE, TileKind};
use crate::render::atlas::TileAtlas;

#[derive(Component)]
pub struct Chunk { pub cx:u32, pub cy:u32 }

#[derive(Resource)]
pub struct RenderConf { pub chunk_w:u32, pub chunk_h:u32 }

pub fn setup_render(
    mut cmds: Commands,
    assets: Res<AssetServer>,
) {
    cmds.insert_resource(RenderConf{ chunk_w:32, chunk_h:32 });

    let atlas_img: Handle<Image> = assets.load("tiles.png");
    cmds.insert_resource(super::atlas::TileAtlas{
        image: atlas_img, cols: 2, rows: 2,
    });
}

pub fn spawn_chunk_entities(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<ColorMaterial>>,
    atlas: Res<TileAtlas>,
    mut grid: ResMut<WorldGrid>,
    conf: Res<RenderConf>,
) {
    // Build chunks immediately with initial mesh data
    for cy in 0..grid.chunk_rows {
        for cx in 0..grid.chunk_cols {
            // Build mesh for this chunk
            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
            let mut positions: Vec<[f32;3]> = Vec::new();
            let mut uvs: Vec<[f32;2]> = Vec::new();
            let mut indices: Vec<u32> = Vec::new();

            let x0 = cx * conf.chunk_w;
            let y0 = cy * conf.chunk_h;

            for ly in 0..conf.chunk_h {
                for lx in 0..conf.chunk_w {
                    let gx = x0 + lx;
                    let gy = y0 + ly;
                    if gx >= grid.w || gy >= grid.h { continue; }

                    let tile = grid.tiles[grid.idx(gx,gy)];
                    let tid = tile_index(tile);
                    let uv = atlas.uv_rect(tid);

                    // Use LOCAL coordinates relative to chunk position
                    let wx = lx as f32 * TILE_SIZE;
                    let wy = ly as f32 * TILE_SIZE;
                    let z = 0.0;
                    let i0 = positions.len() as u32;

                    positions.extend_from_slice(&[
                        [wx,          wy,          z],
                        [wx+TILE_SIZE,wy,          z],
                        [wx+TILE_SIZE,wy+TILE_SIZE,z],
                        [wx,          wy+TILE_SIZE,z],
                    ]);
                    uvs.extend_from_slice(&[
                        uv[0].to_array(), uv[1].to_array(), uv[2].to_array(), uv[3].to_array()
                    ]);
                    indices.extend_from_slice(&[i0, i0+1, i0+2,  i0, i0+2, i0+3]);
                }
            }

            let chunk_world_x = (cx * conf.chunk_w) as f32 * TILE_SIZE;
            let chunk_world_y = (cy * conf.chunk_h) as f32 * TILE_SIZE;

            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
            mesh.insert_indices(Indices::U32(indices));

            let mat = ColorMaterial {
                texture: Some(atlas.image.clone()),
                ..default()
            };
            let mat_handle = mats.add(mat);
            let mesh_handle = meshes.add(mesh);

            cmds.spawn((
                MaterialMesh2dBundle {
                    mesh: mesh_handle.into(),
                    material: mat_handle,
                    transform: Transform::from_xyz(chunk_world_x, chunk_world_y, 0.0),
                    ..default()
                },
                Chunk{ cx, cy },
            ));

            // Mark chunk as clean since we just built it
            let cidx = (cy*grid.chunk_cols + cx) as usize;
            grid.chunk_dirty[cidx] = false;
        }
    }
}

// map TileKind -> tile index in the 2x2 atlas
fn tile_index(kind: TileKind) -> u32 {
    match kind {
        TileKind::Ground    => 0,
        TileKind::Wall      => 1,
        TileKind::Stockpile => 2,
        TileKind::Scavenge  => 3,
    }
}

// rebuild meshes for dirty chunks
pub fn rebuild_dirty_chunks(
    mut grid: ResMut<WorldGrid>,
    conf: Res<RenderConf>,
    atlas: Res<TileAtlas>,
    mut meshes: ResMut<Assets<Mesh>>,
    q_chunks: Query<(&Chunk, &Mesh2dHandle)>,
) {
    for (chunk, mesh_2d_h) in q_chunks.iter() {
        let cidx = (chunk.cy*grid.chunk_cols + chunk.cx) as usize;
        if !grid.chunk_dirty[cidx] { continue; }
        grid.chunk_dirty[cidx] = false;

        let Some(mesh) = meshes.get_mut(&mesh_2d_h.0) else {
            warn!("Failed to get mesh for chunk ({},{})", chunk.cx, chunk.cy);
            continue;
        };

        let mut positions: Vec<[f32;3]> = Vec::new();
        let mut uvs: Vec<[f32;2]> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        let x0 = chunk.cx * conf.chunk_w;
        let y0 = chunk.cy * conf.chunk_h;
        for ly in 0..conf.chunk_h {
            for lx in 0..conf.chunk_w {
                let gx = x0 + lx;
                let gy = y0 + ly;
                if gx >= grid.w || gy >= grid.h { continue; }

                let tile = grid.tiles[grid.idx(gx,gy)];
                let tid = tile_index(tile);
                let uv = atlas.uv_rect(tid);

                let wx = lx as f32 * TILE_SIZE;
                let wy = ly as f32 * TILE_SIZE;
                let z = 0.0;
                let i0 = positions.len() as u32;

                positions.extend_from_slice(&[
                    [wx,          wy,          z],
                    [wx+TILE_SIZE,wy,          z],
                    [wx+TILE_SIZE,wy+TILE_SIZE,z],
                    [wx,          wy+TILE_SIZE,z],
                ]);
                uvs.extend_from_slice(&[
                    uv[0].to_array(), uv[1].to_array(), uv[2].to_array(), uv[3].to_array()
                ]);
                indices.extend_from_slice(&[i0, i0+1, i0+2,  i0, i0+2, i0+3]);
            }
        }
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_indices(Indices::U32(indices));
    }
}
