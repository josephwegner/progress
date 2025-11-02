use bevy::prelude::*;
use std::env;

#[derive(Resource)]
pub struct DebugSettings {
    pub log_pathfinding: bool,
    pub log_jobs: bool,
    pub log_bot_behavior: bool,
}

impl DebugSettings {
    pub fn from_env() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            // For WASM builds, read from environment at compile time
            let log_pathfinding = option_env!("DEBUG_PATHFINDING")
                .unwrap_or("false")
                .parse()
                .unwrap_or(false);

            let log_jobs = option_env!("DEBUG_JOBS")
                .unwrap_or("false")
                .parse()
                .unwrap_or(false);

            let log_bot_behavior = option_env!("DEBUG_BOT_BEHAVIOR")
                .unwrap_or("false")
                .parse()
                .unwrap_or(false);

            info!("Debug settings (WASM) - Pathfinding: {}, Jobs: {}, Bot Behavior: {}",
                  log_pathfinding, log_jobs, log_bot_behavior);

            Self {
                log_pathfinding,
                log_jobs,
                log_bot_behavior,
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let log_pathfinding = env::var("DEBUG_PATHFINDING")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false);

            let log_jobs = env::var("DEBUG_JOBS")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false);

            let log_bot_behavior = env::var("DEBUG_BOT_BEHAVIOR")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false);

            info!("Debug settings - Pathfinding: {}, Jobs: {}, Bot Behavior: {}",
                  log_pathfinding, log_jobs, log_bot_behavior);

            Self {
                log_pathfinding,
                log_jobs,
                log_bot_behavior,
            }
        }
    }
}
