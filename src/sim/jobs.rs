use bevy::prelude::*;
use std::cmp::Ordering;
use crate::sim::entities::{Position, Path};
use crate::sim::movement::Pathfinder;
use crate::sim::entities::bots::{HasJob, CarryingScrap};
use crate::sim::entities::ai_core::AICore;
use crate::sim::debug::DebugSettings;
use crate::sim::grid::TileKind;
use crate::sim::resources::GameResources;
use crate::sim::zones::{ZonedJobQueue, InZone};
use crate::sim::pathfinding::find_path;
use crate::sim::grid::WorldGrid;


#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JobType {
    Scavenge { x: u32, y: u32 },
    ReturnToCore,
}

#[derive(Component, Clone, Debug, Eq, PartialEq)]
pub struct Job {
    pub job_type: JobType,
    pub priority: u32,
}

impl Ord for Job {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl PartialOrd for Job {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


// ============================================================================
// GENERIC JOB EXECUTION SYSTEMS (work for any entity with Pathfinder)
// ============================================================================

pub fn assign_jobs_to_idle_entities(
    mut commands: Commands,
    mut zoned_queue: ResMut<ZonedJobQueue>,
    idle_entities: Query<(Entity, &Position, &InZone), (With<Pathfinder>, Without<HasJob>)>,
    debug: Res<DebugSettings>,
) {
    for (entity, position, in_zone) in idle_entities.iter() {
        let Some(zone_id) = in_zone.0 else {
            if debug.log_jobs {
                warn!("Idle entity {:?} at ({},{}) has no zone", entity, position.x, position.y);
            }
            continue;
        };

        if let Some((_priority, job_entity)) = zoned_queue.pop(zone_id) {
            commands.entity(entity).insert(HasJob(job_entity));

            if debug.log_jobs {
                info!("Assigned job {:?} to entity {:?} in zone {}", job_entity, entity, zone_id);
            }
        } else if debug.log_jobs {
            info!("No job found for entity {:?} in zone {}", entity, zone_id);
        }
    }
}


// System: Takes entities with a job but no path and find a path to the job
pub fn find_path_to_job(
    pathing_entity: Query<(Entity, &HasJob, &Position), Without<Path>>,
    job_entities: Query<&Job>,
    core_query: Query<&Position, With<AICore>>,
    grid: Res<WorldGrid>,
    debug: Res<DebugSettings>,
    mut commands: Commands,
) {
    for (entity, has_job, position) in pathing_entity.iter() {
        if let Ok(job) = job_entities.get(has_job.0) {
            let goal = match job.job_type {
                JobType::Scavenge { x, y } => Position { x, y },
                JobType::ReturnToCore => {
                    match core_query.get_single() {
                        Ok(core_pos) => core_pos.clone(),
                        Err(_) => {
                            warn!("AICore not found, cannot path to core");
                            continue;
                        }
                    }
                }
            };

            if let Some(path) = find_path(position.clone(), goal, &grid) {
                commands.entity(entity).insert(Path {
                    nodes: path,
                    current_idx: 0,
                    movement_cooldown: 0.0,
                });

                if debug.log_jobs {
                    info!("Entity {:?} assigned path to job at ({},{})", entity, goal.x, goal.y);
                }
            } else {
                // If it's a return to core job, don't remove it. The bot can't do anything else until it's back at the core.
                if job.job_type != JobType::ReturnToCore {
                    commands.entity(has_job.0).despawn();   
                    commands.entity(entity).remove::<HasJob>();
                    if debug.log_jobs {
                        info!("Removing job {:?} from entity {:?} because path couldn't be found", has_job.0, entity);
                    }
                }
            }
        }
    }
}

/// System: Execute scavenge jobs when entity reaches location
/// Works for any entity type
/// Run in FixedUpdate, after movement
pub fn execute_scavenge_jobs(
    mut has_job_commands: Commands,
    mut core_job_commands: Commands,
    entities: Query<(Entity, &Position, &HasJob), Without<Path>>,
    job_entities: Query<&Job>,
    mut grid: ResMut<WorldGrid>,
    debug: Res<DebugSettings>,
) {
    for (entity, position, has_job) in entities.iter() {
        if let Ok(job) = job_entities.get(has_job.0) {
            if let JobType::Scavenge { x, y } = job.job_type {
                let dx = (position.x as i32 - x as i32).abs();
                let dy = (position.y as i32 - y as i32).abs();
                let is_adjacent_or_on = dx + dy <= 1;
                if is_adjacent_or_on {
                    core_job_commands.entity(has_job.0).despawn();
                    core_job_commands.entity(entity).insert(CarryingScrap(5));
                    let idx = grid.idx(x, y);

                    // Mark the tile dirty where we just removed scrap
                    grid.tiles[idx] = TileKind::Ground;
                    grid.mark_chunk_dirty(x, y);

                    if debug.log_jobs {
                        info!("Entity {:?} collected scrap at ({},{}) and is returning to core", entity, x, y);
                    }

                    // Create new job to path back to the core
                    let core_job = core_job_commands.spawn(Job {
                        job_type: JobType::ReturnToCore,
                        priority: 10,
                    }).id();
                    has_job_commands.entity(entity).insert(HasJob(core_job));
                }
            }
        }
    }
}

/// System: Deliver resources when entity reaches AI Core
/// Works for any entity type
/// Run in FixedUpdate, after movement
pub fn deliver_resources_to_core(
    mut scrap_commands: Commands,
    mut job_commands: Commands,
    entities: Query<(Entity, &Position, &CarryingScrap), Without<Path>>,
    core_query: Query<&Position, With<AICore>>,
    mut resources: ResMut<GameResources>,
    debug: Res<DebugSettings>,
) {
    for (entity, position, carrying) in entities.iter() {
        if let Ok(core_pos) = core_query.get_single() {
            if position == core_pos {
                resources.add_scrap(carrying.0);
                scrap_commands.entity(entity).remove::<CarryingScrap>();
                job_commands.entity(entity).remove::<HasJob>();

                if debug.log_jobs {
                    info!("Entity {:?} delivered scrap to core", entity);
                }
            }
        }
    }
}

/// System: Clean up entities with jobs that no longer exist
/// Run in FixedUpdate
pub fn cleanup_orphaned_jobs(
    mut commands: Commands,
    entities_with_jobs: Query<(Entity, &HasJob)>,
    jobs: Query<Entity>,
    debug: Res<DebugSettings>,
) {
    for (entity, has_job) in entities_with_jobs.iter() {
        if let Err(_) = jobs.get(has_job.0) {
            commands.entity(entity).remove::<HasJob>();
            if debug.log_jobs {
                warn!("Entity with HasJob component but no corresponding job entity found, removing components");
            }
        }
    }
}
