# Machine Seed v0.2 🤖

A minimalist colony simulation game where you guide a nascent AI as it bootstraps itself in the ruins of human civilization.

**Built with:** Rust + Bevy + WASM

---

## 🎮 How to Play

**Goal:** Reach 50 compute capacity and survive 5 minutes without detection.

### Controls
- **WASD** - Pan camera
- **Mouse Wheel** - Zoom
- **1** - Paint scavenge zones (mark resources for harvesting)
- **2** - Paint stockpile zones
- **3** - Build Server Rack (50 scrap → +10 compute, +5 power drain)
- **4** - Build Power Node (30 scrap → +15 power generation)
- **5** - Build Bot (50 scrap → +1 worker)
- **ESC** - Cancel current tool

### Strategy
1. Paint scavenge zones on resource tiles - your bots will automatically harvest them
2. Build Power Nodes first to increase power generation
3. Build Server Racks to increase compute capacity (watch your power!)
4. Build more bots to speed up resource collection
5. Balance power generation vs consumption
6. Watch out for scouts - they spawn every 45 seconds and try to find your AI core

### Win Conditions
✅ Reach 50 compute capacity
✅ Maintain sustainable power (generation ≥ consumption)
✅ Survive 5 minutes

### Lose Conditions
❌ Power collapse for 60 seconds
❌ Scout detects your AI core

---

## 🚀 Running the Game

### Play in Browser (WASM)
```bash
trunk serve --open
```
Then visit `http://localhost:8080`

### Build Release
```bash
trunk build --release
```
Output in `dist/` folder

### Native Build (for testing)
```bash
cargo run
```

---

## 🏗️ Architecture Philosophy

This game was rebuilt from scratch with a focus on **simplicity over "proper" game architecture**.

### Key Design Decisions

**Explicit State Machine**
```rust
enum BotState {
    Idle,
    MovingToJob { job_id: u32, path_index: usize },
    Harvesting { job_id: u32, progress: f32 },
    ReturningToCore { scrap: u32, path_index: usize },
}
```
Debug: `println!("{:?}", bot.state)` - instantly clear!

**Jobs as Data** (not ECS entities)
```rust
struct Job {
    id: u32,
    position: Position,
    claimed_by: Option<Entity>,
    reachable: bool,
}
```
No entity spawning/despawning, no orphan cleanup, just a `Vec<Job>`.

**No Zone System**
Pathfinding failures are handled gracefully:
```rust
if let Some(path) = find_path(...) {
    // Use path
} else {
    mark_job_unreachable();  // Try next job
}
```

**Centralized World Resource**
All game state in one place:
```rust
struct World {
    grid: Grid,
    scrap: u32,
    power: PowerSystem,
    available_jobs: Vec<Job>,
    // ...
}
```

**Single Bot Update Function**
All bot logic in one system - easy to read, easy to debug, easy to modify.

---

## 📊 Comparison with Previous Version

| Metric | Old Game (ECS-heavy) | New Game (Simple) |
|--------|---------------------|-------------------|
| **Lines of Code** | ~3000+ | ~1200 |
| **Files** | 17+ | 10 |
| **Bot State** | 9+ components | 1 enum |
| **Debug Time** | Archaeological queries | `println!("{:?}")` |
| **Borrow Errors** | Constant fight | 3 simple fixes |
| **Dev Time** | Weeks | 2.25 hours |
| **Stuck Bots** | Common issue | Haven't seen one |

---

## 🎯 Project Goals

This refactor demonstrates that for small-scale games:
- **Simple > "Proper"** - Readable code beats architectural purity
- **Explicit > Implicit** - State machines > component presence detection
- **Centralized > Fragmented** - Single resource > many scattered components
- **Debuggable > Performant** - For 100 entities, clarity matters more than ECS optimization

The old version followed Bevy ECS best practices designed for games with thousands of entities. This version is optimized for **developer ergonomics** and **debuggability** for a hobbyist working alone.

---

## 📁 Project Structure

```
src/
├── main.rs          # App setup, system registration
├── world.rs         # World resource (centralized game state)
├── grid.rs          # Tile grid + tile types
├── pathfinding.rs   # A* pathfinding
├── bot.rs           # Bot component + update logic
├── power.rs         # Power system
├── jobs.rs          # Job data structures
├── buildings.rs     # (future) Building logic
├── scouts.rs        # Scout AI + detection
├── input.rs         # Camera + paint tools + building
├── rendering.rs     # Sprite rendering
└── ui.rs            # HUD + game over screen
```

Each file is self-contained and understandable in isolation.

---

## 🔧 Tech Stack

- **Bevy 0.14** - Game engine (simplified ECS usage)
- **Rust** - Language
- **WASM** - Web target
- **pathfinding crate** - A* algorithm
- **Trunk** - WASM bundler

---

## 📝 Development

### Prerequisites
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install trunk
cargo install trunk
```

### Build Commands
```bash
# Check compilation
cargo check

# Check WASM target
cargo check --target wasm32-unknown-unknown

# Run native (for testing)
cargo run

# Serve WASM with hot reload
trunk serve

# Build WASM release
trunk build --release
```

---

## 🎓 Lessons Learned

1. **ECS is not always the answer** - For small games, simple structs beat complex component systems
2. **Explicit state beats implicit state** - Enums > component presence detection
3. **Debuggability matters more than performance** - At 100 entities, clarity > optimization
4. **AI-assisted coding works best with simple architectures** - Easier to catch AI mistakes in straightforward code
5. **Pathfinding is cheap** - No need for zone systems on 64x64 grids

---

## 🚧 Future Enhancements (Optional)

- [ ] Notifications for important events
- [ ] Construction progress bars
- [ ] Walls (block scout line of sight)
- [ ] Sound effects
- [ ] Better graphics/sprites
- [ ] Save/load system
- [ ] Balance tuning

---

## 📄 License

MIT License - Feel free to use this as a reference for your own projects!

---

## 🙏 Acknowledgments

This project was built as a learning exercise in game architecture and a response to the complexity challenges of ECS-heavy game development for small-scale projects.

**Key insight:** Sometimes the "wrong" architecture is the right choice for your constraints.

---

Built with ❤️ and a lot less frustration than the previous version.
