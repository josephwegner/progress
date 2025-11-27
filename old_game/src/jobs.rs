use bevy::prelude::*;
use crate::pathfinding::Position;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobType {
    Harvest,
}

#[derive(Debug, Clone)]
pub struct Job {
    pub id: u32,
    pub job_type: JobType,
    pub position: Position,
    pub claimed_by: Option<Entity>,
    pub reachable: bool,
}

impl Job {
    pub fn new(id: u32, job_type: JobType, position: Position) -> Self {
        Self {
            id,
            job_type,
            position,
            claimed_by: None,
            reachable: true,
        }
    }

    pub fn is_available(&self) -> bool {
        self.claimed_by.is_none() && self.reachable
    }
}
