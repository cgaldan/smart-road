# Smart Road

Smart Road is a real-time autonomous-vehicle intersection simulation written in
Rust with SDL2. Vehicles follow dedicated left, straight, or right-turn lanes,
while an intersection controller adjusts their speed to avoid conflicting
routes without using traffic lights.

## Features

- Four-direction vehicle spawning with randomized routes
- Continuous random-traffic mode
- Dedicated lanes and animated paths for left, straight, and right turns
- Conflict-aware intersection priority and positive following distance
- Three target speed levels with gradual acceleration and braking
- Spawn throttling to prevent overlapping vehicles
- Responsive fullscreen rendering, animated sprites, and background music
- End-of-run statistics for created and passed vehicles, velocity, travel time,
  close calls, and collisions
- Unit and sustained random-load collision tests

## Requirements

- Rust and Cargo
- SDL2
- SDL2_mixer with MP3 support

On Fedora, the native dependencies can be installed with:

```bash
sudo dnf install SDL2-devel SDL2_mixer-devel
```

## Run

From the repository root:

```bash
cargo run --release --locked
```

The application starts in fullscreen mode.

## Controls

| Input | Action |
| --- | --- |
| `Arrow Up` | Spawn a vehicle travelling north from the south |
| `Arrow Down` | Spawn a vehicle travelling south from the north |
| `Arrow Left` | Spawn a vehicle travelling west from the east |
| `Arrow Right` | Spawn a vehicle travelling east from the west |
| `R` | Toggle continuous random vehicle generation |
| `M` | Mute or unmute music |
| Mouse on the sound icon | Mute or unmute music |
| `F11` | Toggle fullscreen mode |
| `Esc` | Stop the simulation and display statistics |
| `Esc` or `Enter` on statistics | Exit the application |

Every manually or automatically generated vehicle receives a random route:
left, straight, or right.

## Intersection strategy

Each origin-and-route pair is represented by a sampled path through the
intersection. At startup, the simulator builds a conflict table by comparing
these paths. During the simulation:

1. A vehicle keeps a safe following distance from traffic ahead in its lane.
2. Vehicles on non-conflicting paths may cross simultaneously.
3. For conflicting paths, the vehicle nearer the intersection receives
   priority, with its creation ID used as a deterministic tie-breaker.
4. A yielding vehicle slows down and, when necessary, stops before entering the
   conflict area.
5. Speed changes are bounded by acceleration and deceleration values instead of
   happening instantly.

This strategy is designed for the simulation and is not presented as a
real-world autonomous-driving control system.

## Statistics

Pressing `Esc` displays:

- Vehicles passed
- Vehicles created
- Maximum and minimum velocity
- Maximum and minimum crossing time
- Close calls
- Collisions

## Quality checks

Run the same checks used before submission:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --all-features --locked
cargo build --release --locked
```

The test suite includes route-priority, following-distance, statistics,
movement, spawn-throttling, and sustained randomized traffic coverage.

## Project structure

```text
src/
├── geometry.rs       Route construction and conflict detection
├── models.rs         Simulation constants and shared types
├── renderer/         SDL2 rendering and statistics screen
├── simulation/       Traffic control, spawning, statistics, and tests
├── vehicle/          Vehicle state and path movement
└── main.rs           Application loop and input handling
assets/               Embedded visual and audio resources
```

## Assets

The application embeds the image and audio files in `assets/` at compile time.
Asset authorship is separate from the traffic-simulation implementation; do not
claim the custom-assets bonus unless the submitted assets were created by the
project authors and their provenance can be demonstrated.
