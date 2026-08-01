# Changelog

## [Unreleased]

### 2026-07-26

#### Added

- Dedicated unit tests for the four areas called out in the audit, replacing reliance on the single broad randomized integration test:
  - **Physics Engine** (`vehicle::tests::advance_moves_vehicle_by_velocity_times_time`) — confirms `velocity = distance / time` by feeding a known `velocity * dt` displacement into `Vehicle::advance` and recovering the velocity from the resulting on-screen movement. Plus `advance_clamps_progress_to_total_length` for the end-of-path edge case.
  - **Safety Distance Detection** (`simulation::tests::detects_close_call_and_collision_thresholds`, `simulation::tests::target_speed_is_zero_when_lead_vehicle_within_safety_distance`) — verifies the close-call/collision counters trip at the right distance thresholds without double-counting a continuous violation, and that a vehicle is commanded to a hard stop when the gap to a lead vehicle falls within the safety distance.
  - **Smart Intersection Algorithm** (`simulation::tests::conflicting_routes_cause_the_later_vehicle_to_yield`) — places two vehicles on a real conflicting combo (N-straight vs. E-straight) and confirms the vehicle reaching the box first proceeds at full speed while the later one yields to a full stop.
  - **Statistics Accumulation** (`simulation::tests::remove_finished_updates_min_max_and_count_stats`, `simulation::tests::try_spawn_increments_vehicles_created_count`) — covers max/min velocity, max/min time, and the created/passed counts.

  `cargo test` now runs 8 tests (up from 1).

### 2026-07-25

#### Added

- `Stats.collisions` — a counter separate from `close_calls`, tracking actual vehicle overlaps (physical collisions) rather than safety-distance violations (near misses). Displayed as `"COLLISIONS"` on the end-of-simulation stats screen, right under `"CLOSE CALLS"`. Purely a passive counter — it does not affect vehicle behavior.
- `BASE_COLLISION_DISTANCE` in `models.rs` (`vehicle_width * 0.6`) — the center-to-center distance below which two vehicles are considered to be actually overlapping, as opposed to `BASE_SAFETY_DISTANCE` which flags a close call. Scaled into `Layout` the same way as the other physics constants, as `Layout.collision_distance`.

#### Changed

- `Simulation::detect_close_calls` renamed to `detect_close_calls_and_collisions` and extended to track two independent pair-sets in the same pass over all vehicle pairs: `violating_pairs` (safety-distance breaches, unchanged behavior) and the new `colliding_pairs` (actual overlap). Each counter increments only when a pair *newly* enters that state, so one continuous violation/collision is still counted once.
- `Simulation::remove_finished` now also prunes a departed vehicle's id out of `colliding_pairs`, mirroring the existing cleanup of `violating_pairs`.
- The random-load integration test's diagnostic `println!` now also reports `collisions=...` alongside the existing metrics.
