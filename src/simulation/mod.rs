use std::collections::HashSet;
use std::time::Instant;

use crate::geometry::build_conflict_table;
use crate::helpers::{random_direction, random_route};
use crate::models::{combo_index, Direction, Layout, Route, BASE_SAFETY_DISTANCE, MIN_SPAWN_INTERVAL_SECS, RANDOM_SPAWN_INTERVAL_SECS};
use crate::vehicle::Vehicle;

pub struct Stats {
    pub vehicles_created: u32,
    pub vehicles_passed: u32,
    pub max_velocity: f32,
    pub min_velocity: f32,
    pub max_time: f32,
    pub min_time: f32,
    pub close_calls: u32,
    pub collisions: u32,
}

impl Stats {
    fn new() -> Self {
        Self {
            vehicles_created: 0,
            vehicles_passed: 0,
            max_velocity: 0.0,
            min_velocity: f32::MAX,
            max_time: 0.0,
            min_time: f32::MAX,
            close_calls: 0,
            collisions: 0,
        }
    }

    pub fn has_data(&self) -> bool {
        self.vehicles_passed > 0
    }
}

pub struct Simulation {
    pub vehicles: Vec<Vehicle>,
    next_id: u64,
    last_spawn: [Option<Instant>; 4],
    pub random_mode: bool,
    last_random_spawn: Instant,
    conflict_table: Vec<Vec<bool>>,
    violating_pairs: HashSet<(u64, u64)>,
    colliding_pairs: HashSet<(u64, u64)>,
    pub stats: Stats,
}

impl Default for Simulation {
    fn default() -> Self {
        Self::new()
    }
}

impl Simulation {
    pub fn new() -> Self {
        Self {
            vehicles: Vec::new(),
            next_id: 0,
            last_spawn: [None; 4],
            random_mode: false,
            last_random_spawn: Instant::now(),
            conflict_table: build_conflict_table(BASE_SAFETY_DISTANCE),
            violating_pairs: HashSet::new(),
            colliding_pairs: HashSet::new(),
            stats: Stats::new(),
        }
    }

    /// Attempts to spawn a vehicle, respecting the per-direction cooldown
    /// and refusing to spawn on top of a vehicle still stopped near the
    /// spawn point (e.g. queued up in heavy congestion). Returns whether a
    /// vehicle was actually spawned.
    pub fn try_spawn(&mut self, dir: Direction, route: Route, layout: &Layout) -> bool {
        let now = Instant::now();
        let idx = dir.index();
        if let Some(last) = self.last_spawn[idx]
            && now.duration_since(last).as_secs_f32() < MIN_SPAWN_INTERVAL_SECS
        {
            return false;
        }

        let unit = layout.vehicle_width;
        let off = route.lane_offset(unit);
        let spawn_at = crate::geometry::spawn_point(dir, off, unit, layout);
        let clearance = layout.safety_distance * 2.0;
        let blocked = self.vehicles.iter().any(|v| {
            let d = ((v.x - spawn_at.0).powi(2) + (v.y - spawn_at.1).powi(2)).sqrt();
            d < clearance
        });
        if blocked {
            return false;
        }

        self.last_spawn[idx] = Some(now);

        let id = self.next_id;
        self.next_id += 1;
        self.vehicles.push(Vehicle::new(id, dir, route, layout));
        self.stats.vehicles_created += 1;
        true
    }

    pub fn toggle_random(&mut self) {
        self.random_mode = !self.random_mode;
    }

    pub fn update(&mut self, dt: f32, layout: &Layout) {
        if self.random_mode {
            let now = Instant::now();
            if now.duration_since(self.last_random_spawn).as_secs_f32() >= RANDOM_SPAWN_INTERVAL_SECS {
                self.last_random_spawn = now;
                self.try_spawn(random_direction(), random_route(), layout);
            }
        }

        let n = self.vehicles.len();
        let mut targets = vec![0.0f32; n];
        for i in 0..n {
            targets[i] = self.target_speed_for(i, layout);
        }

        for i in 0..n {
            let target = targets[i];
            let v = &mut self.vehicles[i];
            let dv = target - v.velocity;
            let max_step = if dv >= 0.0 { layout.max_accel } else { layout.max_decel } * dt;
            if dv.abs() <= max_step {
                v.velocity = target;
            } else {
                v.velocity += max_step.copysign(dv);
            }
            v.velocity = v.velocity.max(0.0);
            v.advance(v.velocity * dt);

            v.max_velocity_reached = v.max_velocity_reached.max(v.velocity);
            v.min_velocity_reached = v.min_velocity_reached.min(v.velocity);
        }

        self.detect_close_calls_and_collisions(layout);
        self.remove_finished();
    }

    fn target_speed_for(&self, i: usize, layout: &Layout) -> f32 {
        let v = &self.vehicles[i];

        let (fx, fy) = v.forward_vector();
        let mut gap = f32::MAX;
        for (j, other) in self.vehicles.iter().enumerate() {
            if j == i {
                continue;
            }
            let dx = other.x - v.x;
            let dy = other.y - v.y;
            let along = dx * fx + dy * fy;
            if along <= 0.0 || along > layout.lookahead_distance {
                continue;
            }
            let lateral = (dx * fy - dy * fx).abs();
            if lateral > layout.lane_tolerance {
                continue;
            }
            if along < gap {
                gap = along;
            }
        }

        let lane_target = if gap < layout.safety_distance {
            0.0
        } else if gap < layout.safety_distance * 3.0 {
            layout.speed_slow
        } else if gap < layout.safety_distance * 6.0 {
            layout.speed_medium
        } else {
            layout.speed_fast
        };

        if v.has_cleared_box() {
            return lane_target;
        }

        let my_combo = combo_index(v.origin, v.route);
        let mut blocked = false;
        for (j, other) in self.vehicles.iter().enumerate() {
            if j == i || other.has_cleared_box() {
                continue;
            }
            let other_combo = combo_index(other.origin, other.route);
            if !self.conflict_table[my_combo][other_combo] {
                continue;
            }
            let mine = v.remaining_to_entry();
            let theirs = other.remaining_to_entry();
            let they_go_first = theirs < mine || (theirs == mine && other.id < v.id);
            if they_go_first {
                blocked = true;
                break;
            }
        }

        if blocked {
            let gate_target = if v.remaining_to_entry() > layout.stop_margin {
                layout.speed_slow
            } else {
                0.0
            };
            lane_target.min(gate_target)
        } else {
            lane_target
        }
    }

    /// Tracks vehicle pairs currently violating the safety distance or
    /// actually overlapping, counting a close call / collision each time a
    /// pair newly enters that state (so one continuous violation only
    /// counts once).
    fn detect_close_calls_and_collisions(&mut self, layout: &Layout) {
        let n = self.vehicles.len();
        let mut current_violations: HashSet<(u64, u64)> = HashSet::new();
        let mut current_collisions: HashSet<(u64, u64)> = HashSet::new();
        for i in 0..n {
            for j in (i + 1)..n {
                let a = &self.vehicles[i];
                let b = &self.vehicles[j];
                let d = ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt();
                let key = if a.id < b.id { (a.id, b.id) } else { (b.id, a.id) };
                if d < layout.safety_distance {
                    current_violations.insert(key);
                }
                if d < layout.collision_distance {
                    current_collisions.insert(key);
                }
            }
        }
        self.stats.close_calls += current_violations.difference(&self.violating_pairs).count() as u32;
        self.stats.collisions += current_collisions.difference(&self.colliding_pairs).count() as u32;
        self.violating_pairs = current_violations;
        self.colliding_pairs = current_collisions;
    }

    fn remove_finished(&mut self) {
        let mut i = 0;
        while i < self.vehicles.len() {
            if self.vehicles[i].is_finished() {
                let v = self.vehicles.remove(i);
                let elapsed = v.spawn_time.elapsed().as_secs_f32();
                self.stats.vehicles_passed += 1;
                self.stats.max_time = self.stats.max_time.max(elapsed);
                self.stats.min_time = self.stats.min_time.min(elapsed);
                self.stats.max_velocity = self.stats.max_velocity.max(v.max_velocity_reached);
                self.stats.min_velocity = self.stats.min_velocity.min(v.min_velocity_reached);
                self.violating_pairs.retain(|&(a, b)| a != v.id && b != v.id);
                self.colliding_pairs.retain(|&(a, b)| a != v.id && b != v.id);
            } else {
                i += 1;
            }
        }
    }

    pub fn resize(&mut self, old: &Layout, new: &Layout) {
        for v in &mut self.vehicles {
            v.resize(old, new);
        }
    }

    /// Folds velocity extremes of any still-in-transit vehicles into the
    /// stats, so the end screen reflects them even if the sim is stopped
    /// before they finish crossing the intersection.
    pub fn finalize_stats(&mut self) {
        for v in &self.vehicles {
            self.stats.max_velocity = self.stats.max_velocity.max(v.max_velocity_reached);
            self.stats.min_velocity = self.stats.min_velocity.min(v.min_velocity_reached);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// Hammers spawn requests (like spamming/holding every arrow key at
    /// once) for several real seconds and checks that no two vehicles ever
    /// physically overlap, and that vehicles do complete the crossing.
    #[test]
    fn no_collisions_under_random_load() {
        let layout = Layout::new(800, 800);
        let mut sim = Simulation::new();

        let run_for = Duration::from_secs(12);
        let start = Instant::now();
        let mut last = Instant::now();
        let mut min_dist_ever = f32::MAX;
        let mut max_active = 0usize;

        while start.elapsed() < run_for {
            let now = Instant::now();
            let dt = (now - last).as_secs_f32().min(0.05);
            last = now;

            for dir in Direction::ALL {
                sim.try_spawn(dir, random_route(), &layout);
            }

            sim.update(dt, &layout);
            max_active = max_active.max(sim.vehicles.len());

            for i in 0..sim.vehicles.len() {
                for j in (i + 1)..sim.vehicles.len() {
                    let a = &sim.vehicles[i];
                    let b = &sim.vehicles[j];
                    let d = ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt();
                    min_dist_ever = min_dist_ever.min(d);
                }
            }

            std::thread::sleep(Duration::from_millis(5));
        }

        println!(
            "created={} passed={} close_calls={} collisions={} max_v={:.1} min_v={:.1} max_t={:.2} min_t={:.2} min_dist_ever={:.2} max_active={}",
            sim.stats.vehicles_created,
            sim.stats.vehicles_passed,
            sim.stats.close_calls,
            sim.stats.collisions,
            sim.stats.max_velocity,
            sim.stats.min_velocity,
            sim.stats.max_time,
            sim.stats.min_time,
            min_dist_ever,
            max_active,
        );

        assert!(sim.stats.vehicles_created > 20, "expected sustained spawning");
        assert!(sim.stats.vehicles_passed > 0, "no vehicle ever finished crossing");
        assert!(
            min_dist_ever > layout.vehicle_width * 0.6,
            "vehicles overlapped: min_dist_ever={min_dist_ever}"
        );
    }


    #[test]
    fn detects_close_call_and_collision_thresholds() {
        let layout = Layout::new(800, 800);
        let mut sim = Simulation::new();

        let mut a = Vehicle::new(0, Direction::N, Route::Straight, &layout);
        let mut b = Vehicle::new(1, Direction::S, Route::Straight, &layout);

        a.x = 0.0;
        a.y = 0.0;
        b.x = 1_000.0;
        b.y = 1_000.0;
        sim.vehicles.push(a);
        sim.vehicles.push(b);

        sim.detect_close_calls_and_collisions(&layout);
        assert_eq!(sim.stats.close_calls, 0);
        assert_eq!(sim.stats.collisions, 0);

        let safe_gap = layout.safety_distance * 0.8;
        assert!(safe_gap > layout.collision_distance, "test assumes safety_distance > collision_distance");
        sim.vehicles[1].x = sim.vehicles[0].x + safe_gap;
        sim.vehicles[1].y = sim.vehicles[0].y;
        sim.detect_close_calls_and_collisions(&layout);
        assert_eq!(sim.stats.close_calls, 1, "expected a new close call to be counted");
        assert_eq!(sim.stats.collisions, 0, "gap is still outside collision distance");

        let collide_gap = layout.collision_distance * 0.5;
        sim.vehicles[1].x = sim.vehicles[0].x + collide_gap;
        sim.detect_close_calls_and_collisions(&layout);
        assert_eq!(sim.stats.collisions, 1, "expected a new collision to be counted");
        assert_eq!(
            sim.stats.close_calls, 1,
            "same continuous violation shouldn't be double-counted as a close call"
        );
    }

    #[test]
    fn target_speed_is_zero_when_lead_vehicle_within_safety_distance() {
        let layout = Layout::new(800, 800);
        let mut sim = Simulation::new();

        let mut follower = Vehicle::new(0, Direction::N, Route::Straight, &layout);
        let mut leader = Vehicle::new(1, Direction::N, Route::Straight, &layout);

        follower.progress = 50.0;
        follower.advance(0.0);
        leader.progress = 50.0 + layout.safety_distance * 0.5;
        leader.advance(0.0);

        sim.vehicles.push(follower);
        sim.vehicles.push(leader);

        let target = sim.target_speed_for(0, &layout);
        assert_eq!(target, 0.0, "expected a hard stop when gap is within safety distance");
    }

    #[test]
    fn conflicting_routes_cause_the_later_vehicle_to_yield() {
        let layout = Layout::new(800, 800);
        let mut sim = Simulation::new();

        let mut first = Vehicle::new(0, Direction::N, Route::Straight, &layout);
        let mut second = Vehicle::new(1, Direction::E, Route::Straight, &layout);

        let my_combo = combo_index(Direction::N, Route::Straight);
        let other_combo = combo_index(Direction::E, Route::Straight);
        assert!(
            sim.conflict_table[my_combo][other_combo],
            "test assumes N-straight and E-straight conflict inside the box"
        );

        first.progress = first.entry_arc - 2.0;
        first.advance(0.0);
        second.progress = second.entry_arc - 20.0;
        second.advance(0.0);
        assert!(second.remaining_to_entry() < layout.stop_margin);

        sim.vehicles.push(first);
        sim.vehicles.push(second);

        let first_target = sim.target_speed_for(0, &layout);
        let second_target = sim.target_speed_for(1, &layout);

        assert_eq!(first_target, layout.speed_fast, "closer vehicle should proceed at full speed");
        assert_eq!(second_target, 0.0, "later vehicle on a conflicting route must yield/stop");
    }

    #[test]
    fn remove_finished_updates_min_max_and_count_stats() {
        let layout = Layout::new(800, 800);
        let mut sim = Simulation::new();

        let mut fast_and_long = Vehicle::new(0, Direction::N, Route::Straight, &layout);
        fast_and_long.max_velocity_reached = 200.0;
        fast_and_long.min_velocity_reached = 50.0;
        fast_and_long.spawn_time = Instant::now() - Duration::from_secs_f32(5.0);
        fast_and_long.progress = fast_and_long.total_len;

        let mut slow_and_short = Vehicle::new(1, Direction::S, Route::Straight, &layout);
        slow_and_short.max_velocity_reached = 80.0;
        slow_and_short.min_velocity_reached = 10.0;
        slow_and_short.spawn_time = Instant::now() - Duration::from_secs_f32(1.0);
        slow_and_short.progress = slow_and_short.total_len;

        sim.vehicles.push(fast_and_long);
        sim.vehicles.push(slow_and_short);

        sim.remove_finished();

        assert_eq!(sim.stats.vehicles_passed, 2, "both finished vehicles should be counted");
        assert!(sim.vehicles.is_empty(), "finished vehicles should be removed from the active list");
        assert!((sim.stats.max_velocity - 200.0).abs() < 0.01, "max_velocity={}", sim.stats.max_velocity);
        assert!((sim.stats.min_velocity - 10.0).abs() < 0.01, "min_velocity={}", sim.stats.min_velocity);
        assert!(sim.stats.max_time >= 4.9, "max_time={}", sim.stats.max_time);
        assert!(sim.stats.min_time <= 1.5, "min_time={}", sim.stats.min_time);
    }

    #[test]
    fn try_spawn_increments_vehicles_created_count() {
        let layout = Layout::new(800, 800);
        let mut sim = Simulation::new();

        for dir in Direction::ALL {
            assert!(
                sim.try_spawn(dir, Route::Straight, &layout),
                "expected the first spawn per direction to succeed"
            );
        }

        assert_eq!(sim.stats.vehicles_created, 4);
    }
}
