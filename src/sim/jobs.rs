use bevy::prelude::*;
use std::collections::BinaryHeap;
use std::cmp::Ordering;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JobType {
    Scavenge { x: u32, y: u32 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Job {
    pub job_type: JobType,
    pub priority: u32,
    pub entity: Entity,
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

#[derive(Resource, Default)]
pub struct JobQueue {
    pub queue: BinaryHeap<Job>,
}

impl JobQueue {
    pub fn push(&mut self, job_type: JobType, priority: u32, commands: &mut Commands) -> Entity {
        let entity = commands.spawn_empty().id();
        let job = Job {
            job_type,
            priority,
            entity,
        };
        self.queue.push(job);
        entity
    }

    pub fn pop(&mut self) -> Option<Job> {
        self.queue.pop()
    }

    pub fn remove_job(&mut self, entity: Entity) {
        self.queue.retain(|job| job.entity != entity);
    }
}

pub fn create_scavenge_jobs(
    mut commands: Commands,
    mut job_queue: ResMut<JobQueue>,
    grid: Res<crate::sim::grid::WorldGrid>,
    keys: Res<ButtonInput<KeyCode>>,
    debug: Res<crate::sim::debug::DebugSettings>,
) {
    if !keys.just_pressed(KeyCode::Space) {
        return;
    }

    job_queue.queue.clear();

    let mut job_count = 0;
    for y in 0..grid.h {
        for x in 0..grid.w {
            let tile = grid.tiles[grid.idx(x, y)];
            if matches!(tile, crate::sim::grid::TileKind::Scavenge) {
                job_queue.push(
                    JobType::Scavenge { x, y },
                    10,
                    &mut commands,
                );
                job_count += 1;
            }
        }
    }
    if debug.log_jobs {
        info!("Cleared old jobs, created {} new scavenge jobs", job_count);
    }
}
