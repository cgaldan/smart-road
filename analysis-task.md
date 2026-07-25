# Smart Road — Audit Analysis

Review of the current implementation (`main.rs`, `vehicle`, `simulation`, `models`, `geometry`, `renderer`) against `Smart-Road-Assignment.md` and `Smart-Road-New-Audit.md`. `cargo test` passes (1 test).

## What's solid

- **Spawn keys & anti-spam**: Up/Down/Left/Right map correctly to N/S/W/E spawns (`main.rs:99-117`), each direction has a 0.55s cooldown (`MIN_SPAWN_INTERVAL_SECS`) plus a spatial "don't spawn on top of a queued car" check.
- **`R` random mode**: toggles continuous random spawning (`simulation/mod.rs:102`).
- **Routes/lanes**: each route (`l/s/r`) has a fixed lane offset (`models.rs:120`), vehicles can't change lane/route.
- **≥3 velocities**: `speed_slow/medium/fast` + 0, plus a full accel/decel model (`BASE_MAX_ACCEL`/`BASE_MAX_DECEL`) — already satisfies the bonus "acceleration/deceleration" item.
- **Safety distance & collision avoidance**: strictly positive `BASE_SAFETY_DISTANCE`, a precomputed conflict table between all 12 direction×route combos, and a "who reaches the intersection first" gating rule that slows/stops the yielding vehicle (`simulation/mod.rs:142-212`).
- **Close calls**: tracked via pairwise distance violation transitions (`detect_close_calls`).
- **Stats window**: shows vehicles passed/created, max/min velocity, max/min time, close calls, triggered on `Esc` (`renderer/mod.rs:405-458`).

## Gaps found

1. **No "Collisions" stat displayed.** The audit explicitly asks *"did it also display the Collisions as 0?"* as separate from close calls, but `Stats`/`draw_stats` only has `close_calls`, no `collisions` field/row.

2. **No image/asset for the road/intersection itself.** The very first audit question is *"is there any image/asset that represents this intersection?"* The road surface, lane dividers, and stop lines are all procedurally drawn filled rectangles (`draw_roads`, `draw_lane_dividers` in `renderer/mod.rs:264-362`) — only the background (nebula/stars) and vehicles use real textures.

3. **Unit test coverage doesn't map to what's asked.** Only one `#[test]` exists (`simulation/mod.rs:277`), a broad randomized integration test. The audit wants separate, identifiable tests for:
   - Physics Engine (`velocity = distance / time` accuracy)
   - Safety Distance Detection (close call / stop when gap ≤ safety distance)
   - Smart Intersection Algorithm (conflicting routes → one vehicle yields)
   - Statistics Accumulation (max/min velocity, time, count updated correctly)

4. **Bonus "own assets" unconfirmed.** The ship/nebula sprites look like themed asset-pack art, not confirmed to be self-made — needed to actually claim that bonus point.

5. **Gameplay checklist not manually verified.** No display available in this environment, so the actual play-through checks (spawning patterns, collision-free runs, congestion staying low over 1 min, etc.) haven't been run live.

## Remaining tasks

- [ ] Add a `collisions` stat (separate from `close_calls`) to `Stats` and display it on the stats screen
- [x] Add a real road/intersection image asset and use it in `draw_roads`/`draw_lane_dividers` instead of procedurally filled rectangles (closed by user judgment call — road is a semi-transparent fill over the nebula/star background with procedural dashed lines, not a dedicated road texture; visually legible but a thinner answer to the audit question than a real asset would be)
- [ ] Add a dedicated unit test for the Physics Engine (`velocity = distance / time`)
- [ ] Add a dedicated unit test for Safety Distance Detection (close call / stop condition)
- [ ] Add a dedicated unit test for the Smart Intersection Algorithm (conflicting routes → yield)
- [ ] Add a dedicated unit test for Statistics Accumulation (max/min velocity, time, count)
- [x] Confirm provenance of vehicle/background assets (self-made vs. sourced) for the "own assets" bonus (closed by user judgment call — evidence points to sourced asset-pack art, not self-made: filenames follow a genre-standard warship taxonomy — scout/frigate/battlecruiser/dreadnought/torpedoShip — each with a matching `_engine` companion sprite sheet, a common pre-made-pack convention; `"Starry background - Layer X - Big Star.png"` follows the layer-naming pattern typical of exported parallax-background packs; all assets landed in a single commit as finished PNGs with no source files, license/credits file, or embedded authorship metadata. Not conclusive, but consistent with sourced art rather than self-made — recommend not claiming this bonus point unless provenance can be confirmed otherwise)
- [ ] Manually run the app locally and verify the full gameplay checklist from `Smart-Road-New-Audit.md`
