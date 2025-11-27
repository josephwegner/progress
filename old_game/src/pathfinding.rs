use pathfinding::prelude::astar;
use crate::grid::Grid;

pub type Position = (u32, u32);

pub fn find_path(start: Position, goal: Position, grid: &Grid) -> Option<Vec<Position>> {
    // Check if goal is walkable
    let goal_is_walkable = grid.is_walkable(goal.0, goal.1);

    let result = astar(
        &start,
        |&(x, y)| {
            let mut neighbors = Vec::new();
            let directions = [(0i32, 1i32), (1, 0), (0, -1), (-1, 0)];

            for (dx, dy) in directions.iter() {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;

                if nx >= 0 && ny >= 0 && nx < grid.width as i32 && ny < grid.height as i32 {
                    let nx = nx as u32;
                    let ny = ny as u32;

                    if grid.is_walkable(nx, ny) {
                        neighbors.push(((nx, ny), 1u32));
                    }
                }
            }
            neighbors
        },
        |&(x, y)| {
            // Manhattan distance heuristic
            ((x as i32 - goal.0 as i32).abs() + (y as i32 - goal.1 as i32).abs()) as u32
        },
        |&pos| {
            if goal_is_walkable {
                // Goal is walkable: must reach exactly
                pos == goal
            } else {
                // Goal is non-walkable (resource): succeed when adjacent
                let dx = (pos.0 as i32 - goal.0 as i32).abs();
                let dy = (pos.1 as i32 - goal.1 as i32).abs();
                dx + dy == 1
            }
        },
    );

    result.map(|(path, _cost)| path)
}

pub fn manhattan_distance(a: Position, b: Position) -> u32 {
    ((a.0 as i32 - b.0 as i32).abs() + (a.1 as i32 - b.1 as i32).abs()) as u32
}
