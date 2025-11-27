use bevy::prelude::*;
use crate::grid::{Grid, TileKind, MAP_WIDTH, MAP_HEIGHT};
use crate::jobs::{Job, JobType};
use crate::power::PowerSystem;
use crate::pathfinding::{Position, manhattan_distance};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameOverState {
    Victory,
    PowerCollapse,
    Detected,
}

#[derive(Resource)]
pub struct World {
    pub grid: Grid,
    pub scrap: u32,
    pub compute_capacity: u32,
    pub power: PowerSystem,
    pub available_jobs: Vec<Job>,
    next_job_id: u32,
    pub core_position: Position,
    pub game_time: f32,
    pub game_over: Option<GameOverState>,
}

impl Default for World {
    fn default() -> Self {
        Self {
            grid: Grid::new(MAP_WIDTH, MAP_HEIGHT),
            scrap: 100,  // Start with some scrap to build initial infrastructure
            compute_capacity: 10,
            power: PowerSystem::default(),
            available_jobs: Vec::new(),
            next_job_id: 0,
            core_position: (MAP_WIDTH / 2, MAP_HEIGHT / 2),
            game_time: 0.0,
            game_over: None,
        }
    }
}

impl World {
    pub fn scan_for_jobs(&mut self) {
        // Clear old jobs
        self.available_jobs.clear();

        // Scan grid for scavenge tiles
        for y in 0..self.grid.height {
            for x in 0..self.grid.width {
                if self.grid.get(x, y) == TileKind::Scavenge {
                    let job = Job::new(self.next_job_id, JobType::Harvest, (x, y));
                    self.available_jobs.push(job);
                    self.next_job_id += 1;
                }
            }
        }

        info!("Scanned for jobs: found {}", self.available_jobs.len());
    }

    pub fn claim_nearest_job(&mut self, from: Position) -> Option<u32> {
        let mut best_job_idx: Option<usize> = None;
        let mut best_dist = u32::MAX;

        for (idx, job) in self.available_jobs.iter().enumerate() {
            if job.is_available() {
                let dist = manhattan_distance(from, job.position);
                if dist < best_dist {
                    best_dist = dist;
                    best_job_idx = Some(idx);
                }
            }
        }

        if let Some(idx) = best_job_idx {
            let job_id = self.available_jobs[idx].id;
            Some(job_id)
        } else {
            None
        }
    }

    pub fn claim_job(&mut self, job_id: u32, claimer: Entity) {
        if let Some(job) = self.available_jobs.iter_mut().find(|j| j.id == job_id) {
            job.claimed_by = Some(claimer);
        }
    }

    pub fn get_job(&self, job_id: u32) -> Option<&Job> {
        self.available_jobs.iter().find(|j| j.id == job_id)
    }

    pub fn mark_job_unreachable(&mut self, job_id: u32) {
        if let Some(job) = self.available_jobs.iter_mut().find(|j| j.id == job_id) {
            job.reachable = false;
            warn!("Marked job {} at {:?} as unreachable", job_id, job.position);
        }
    }

    pub fn complete_job(&mut self, job_id: u32) {
        if let Some(idx) = self.available_jobs.iter().position(|j| j.id == job_id) {
            let job = self.available_jobs.remove(idx);
            // Remove the tile from the grid
            self.grid.set(job.position.0, job.position.1, TileKind::Ground);
            info!("Completed job {} at {:?}", job_id, job.position);
        }
    }

    pub fn add_scrap(&mut self, amount: u32) {
        self.scrap += amount;
    }

    pub fn spend_scrap(&mut self, amount: u32) -> bool {
        if self.scrap >= amount {
            self.scrap -= amount;
            true
        } else {
            false
        }
    }

    pub fn check_win_condition(&self) -> bool {
        // Win if: 50 compute capacity, sustainable power, and survived 5 minutes
        self.compute_capacity >= 50
            && self.power.generation >= self.power.consumption_buildings
            && self.game_time >= 300.0  // 5 minutes
    }

    pub fn check_power_collapse(&self) -> bool {
        // Lose if power collapsed for 60 seconds
        self.power.collapse_timer >= 60.0
    }
}

pub fn setup_world(
    mut commands: Commands,
    mut world: ResMut<World>,
) {
    // Place AI Core in center
    let core_pos = world.core_position;
    world.grid.set(core_pos.0, core_pos.1, TileKind::AICore);

    // Spawn some initial bots
    let core_x = world.core_position.0 as i32;
    let core_y = world.core_position.1 as i32;

    for i in 0..2 {
        let offset = if i == 0 { 2 } else { -2 };
        let position = ((core_x + offset) as u32, core_y as u32);

        commands.spawn(crate::bot::Bot::new(position));
    }

    // Add some initial scavenge tiles for testing
    for i in 0..10 {
        world.grid.set(10 + i * 2, 10, TileKind::Scavenge);
        world.grid.set(10 + i * 2, 15, TileKind::Scavenge);
    }

    // Add some resource clusters
    for y in 20..25 {
        for x in 40..45 {
            world.grid.set(x, y, TileKind::Scavenge);
        }
    }

    // Scan for initial jobs
    world.scan_for_jobs();

    info!("World initialized: core at {:?}, {} bots spawned", world.core_position, 2);
}

pub fn update_game_time(
    mut world: ResMut<World>,
    time: Res<Time>,
) {
    if world.game_over.is_some() {
        return;  // Game is over, stop updating time
    }

    world.game_time += time.delta_seconds();

    // Check win/lose conditions
    if world.check_win_condition() {
        world.game_over = Some(GameOverState::Victory);
        info!("Victory! Game won at {:.1}s", world.game_time);
    } else if world.check_power_collapse() {
        world.game_over = Some(GameOverState::PowerCollapse);
        info!("Defeat! Power collapsed at {:.1}s", world.game_time);
    }
}
