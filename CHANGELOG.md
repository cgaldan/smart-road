# Changelog

## [Unreleased]

### Added

- `Stats.collisions` — a counter separate from `close_calls`, tracking actual vehicle overlaps (physical collisions) rather than safety-distance violations (near misses). Displayed as `"COLLISIONS"` on the end-of-simulation stats screen, right under `"CLOSE CALLS"`. Purely a passive counter — it does not affect vehicle behavior.
- `BASE_COLLISION_DISTANCE` in `models.rs` (`vehicle_width * 0.6`) — the center-to-center distance below which two vehicles are considered to be actually overlapping, as opposed to `BASE_SAFETY_DISTANCE` which flags a close call. Scaled into `Layout` the same way as the other physics constants, as `Layout.collision_distance`.

### Changed

- `Simulation::detect_close_calls` renamed to `detect_close_calls_and_collisions` and extended to track two independent pair-sets in the same pass over all vehicle pairs: `violating_pairs` (safety-distance breaches, unchanged behavior) and the new `colliding_pairs` (actual overlap). Each counter increments only when a pair *newly* enters that state, so one continuous violation/collision is still counted once.
- `Simulation::remove_finished` now also prunes a departed vehicle's id out of `colliding_pairs`, mirroring the existing cleanup of `violating_pairs`.
- The random-load integration test's diagnostic `println!` now also reports `collisions=...` alongside the existing metrics.
