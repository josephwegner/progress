use bevy::prelude::*;
use std::collections::{HashMap, HashSet, BinaryHeap, VecDeque};
use crate::sim::grid::{WorldGrid, TileKind};
use crate::sim::entities::Position;
use crate::sim::game_state::WorldChangeTracker;
use crate::sim::jobs::{Job, JobType};
use crate::sim::debug::DebugSettings;

// ============================================================================
// RESOURCES
// ============================================================================

/// Maps each walkable position to its zone ID
/// Zones are contiguous regions of walkable tiles
#[derive(Resource, Default)]
pub struct ReachabilityZones {
    pub zones: HashMap<(u32, u32), usize>,  // position -> zone_id
    pub zone_count: usize,
}

impl ReachabilityZones {
    pub fn get_zone(&self, x: u32, y: u32) -> Option<usize> {
        self.zones.get(&(x, y)).copied()
    }
}

/// Per-zone job queues - each zone has its own priority queue of jobs
#[derive(Resource, Default)]
pub struct ZonedJobQueue {
    pub queues: HashMap<usize, BinaryHeap<(u32, Entity)>>,  // zone_id -> (priority, job_entity)
}

impl ZonedJobQueue {
    pub fn push(&mut self, zone_id: usize, priority: u32, job_entity: Entity) {
        self.queues.entry(zone_id)
            .or_insert_with(BinaryHeap::new)
            .push((priority, job_entity));
    }

    pub fn pop(&mut self, zone_id: usize) -> Option<(u32, Entity)> {
        self.queues.get_mut(&zone_id)?.pop()
    }

    pub fn clear(&mut self) {
        self.queues.clear();
    }
}

// ============================================================================
// COMPONENTS
// ============================================================================

/// Tag component indicating which zone an entity is currently in
#[derive(Component, Default)]
pub struct InZone(pub Option<usize>);  // None if in non-walkable area

// ============================================================================
// SYSTEMS
// ============================================================================

/// System: Build initial zones on startup
/// Run in Startup, after map generation
pub fn build_initial_zones(
    mut zones: ResMut<ReachabilityZones>,
    mut in_zone_entities: Query<(Entity, &Position, &mut InZone)>,
    grid: Res<WorldGrid>,
    debug: Res<DebugSettings>,
) {
    build_zones(&mut zones, &mut in_zone_entities, &grid, &debug);
}

/// System: Rebuild reachability zones when world changes
/// Uses flood-fill to identify contiguous walkable regions
/// Optimized to only rebuild affected zones
/// Run in FixedUpdate, after track_world_changes
pub fn rebuild_reachability_zones(
    mut zones: ResMut<ReachabilityZones>,
    mut in_zone_entities: Query<(Entity, &Position, &mut InZone)>,
    tracker: Res<WorldChangeTracker>,
    grid: Res<WorldGrid>,
    debug: Res<DebugSettings>,
) {
    // Only rebuild if tiles changed
    if tracker.tiles_changed.is_empty() {
        return;
    }

    // Find affected zones - zones that contain or are adjacent to changed tiles
    let mut affected_zones = HashSet::new();
    for &(x, y) in tracker.tiles_changed.iter() {
        // Check the changed tile itself
        if let Some(zone_id) = zones.get_zone(x, y) {
            affected_zones.insert(zone_id);
        }

        // Check all 4 neighbors
        let directions = [(0i32, 1i32), (1, 0), (0, -1), (-1, 0)];
        for (dx, dy) in directions.iter() {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;

            if nx >= 0 && ny >= 0 && nx < grid.w as i32 && ny < grid.h as i32 {
                if let Some(zone_id) = zones.get_zone(nx as u32, ny as u32) {
                    affected_zones.insert(zone_id);
                }
            }
        }
    }

    // If changes affect zones, do a full rebuild
    // (Partial rebuild is complex due to zone splitting/merging)
    if !affected_zones.is_empty() {
        if debug.log_pathfinding {
            info!("Rebuilding {} affected zones out of {} total zones", affected_zones.len(), zones.zone_count);
        }
        build_zones(&mut zones, &mut in_zone_entities, &grid, &debug);
    }
}

fn build_zones(
    zones: &mut ReachabilityZones,
    in_zone_entities: &mut Query<(Entity, &Position, &mut InZone)>,
    grid: &WorldGrid,
    debug: &DebugSettings,
) {
    zones.zones.clear();
    zones.zone_count = 0;

    let mut visited = HashSet::new();

    // Flood fill from each unvisited walkable tile
    for y in 0..grid.h {
        for x in 0..grid.w {
            if visited.contains(&(x, y)) {
                continue;
            }

            let tile = grid.tiles[grid.idx(x, y)];
            if tile != TileKind::Ground && tile != TileKind::Stockpile {
                continue;
            }

            // Found a new zone! Flood fill it
            let zone_id = zones.zone_count;
            zones.zone_count += 1;

            let mut queue = VecDeque::new();
            queue.push_back((x, y));
            visited.insert((x, y));

            while let Some((cx, cy)) = queue.pop_front() {
                zones.zones.insert((cx, cy), zone_id);

                // Check all 4 neighbors
                let directions = [(0i32, 1i32), (1, 0), (0, -1), (-1, 0)];
                for (dx, dy) in directions.iter() {
                    let nx = cx as i32 + dx;
                    let ny = cy as i32 + dy;

                    if nx < 0 || ny < 0 || nx >= grid.w as i32 || ny >= grid.h as i32 {
                        continue;
                    }

                    let nx = nx as u32;
                    let ny = ny as u32;

                    if visited.contains(&(nx, ny)) {
                        continue;
                    }

                    let neighbor_tile = grid.tiles[grid.idx(nx, ny)];
                    if neighbor_tile == TileKind::Ground || neighbor_tile == TileKind::Stockpile {
                        visited.insert((nx, ny));
                        queue.push_back((nx, ny));
                    }
                }
            }
        }
    }

    add_entities_to_zones(in_zone_entities, &zones);

    if debug.log_pathfinding {
        info!("Rebuilt reachability zones: {} zones found", zones.zone_count);
    }
}

fn add_entities_to_zones(in_zone_entities: &mut Query<(Entity, &Position, &mut InZone)>, zones: &ReachabilityZones) {
    for (_entity, position, mut in_zone) in in_zone_entities.iter_mut() {
        let zone_id = zones.get_zone(position.x, position.y);
        in_zone.0 = zone_id;
    }
}

/// System: Scan for initial jobs on startup
/// Run in Startup, after build_initial_zones
pub fn scan_initial_jobs(
    mut commands: Commands,
    mut zoned_queue: ResMut<ZonedJobQueue>,
    zones: Res<ReachabilityZones>,
    grid: Res<WorldGrid>,
    debug: Res<DebugSettings>,
) {
    scan_jobs(&mut commands, &mut zoned_queue, &zones, &grid, &debug);
}

/// System: Scan for jobs in each zone by looking at edge tiles
/// Edge tiles are walkable tiles adjacent to non-walkable tiles
/// Run in FixedUpdate, after rebuild_reachability_zones
pub fn scan_for_jobs_in_zones(
    mut commands: Commands,
    mut zoned_queue: ResMut<ZonedJobQueue>,
    zones: Res<ReachabilityZones>,
    tracker: Res<WorldChangeTracker>,
    grid: Res<WorldGrid>,
    debug: Res<DebugSettings>,
) {
    // Only rescan if zones were rebuilt (tiles changed)
    if tracker.tiles_changed.is_empty() {
        return;
    }

    scan_jobs(&mut commands, &mut zoned_queue, &zones, &grid, &debug);
}

fn scan_jobs(
    commands: &mut Commands,
    zoned_queue: &mut ZonedJobQueue,
    zones: &ReachabilityZones,
    grid: &WorldGrid,
    debug: &DebugSettings,
) {
    // Clear old jobs
    zoned_queue.clear();

    let mut jobs_created = 0;
    let mut scavenge_tiles_processed: HashSet<(u32, u32)> = HashSet::new();

    // Iterate through all positions in zones
    for (&(x, y), &zone_id) in zones.zones.iter() {
        // Check all 4 neighbors for non-walkable tiles with jobs
        let directions = [(0i32, 1i32), (1, 0), (0, -1), (-1, 0)];
        for (dx, dy) in directions.iter() {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;

            if nx < 0 || ny < 0 || nx >= grid.w as i32 || ny >= grid.h as i32 {
                continue;
            }

            let nx = nx as u32;
            let ny = ny as u32;

            let neighbor_tile = grid.tiles[grid.idx(nx, ny)];

            // If neighbor is Scavenge (non-walkable with job), create job in this zone
            if neighbor_tile == TileKind::Scavenge && !scavenge_tiles_processed.contains(&(nx, ny)) {
                scavenge_tiles_processed.insert((nx, ny));
                let job_entity = commands.spawn(Job {
                    job_type: JobType::Scavenge { x: nx, y: ny },
                    priority: 10,
                }).id();

                zoned_queue.push(zone_id, 10, job_entity);
                jobs_created += 1;
            }
        }
    }

    if debug.log_jobs {
        info!("Scanned zones and created {} jobs across {} zones", jobs_created, zones.zone_count);
    }
}

/// System: Update which zone each entity is in
/// Run in FixedUpdate, after scan_for_jobs_in_zones
pub fn update_entity_zones(
    mut entities: Query<(&Position, &mut InZone)>,
    zones: Res<ReachabilityZones>,
) {
    for (position, mut in_zone) in entities.iter_mut() {
        let new_zone = zones.get_zone(position.x, position.y);
        if in_zone.0 != new_zone {
            in_zone.0 = new_zone;
        }
    }
}
