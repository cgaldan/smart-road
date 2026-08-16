use std::time::Instant;

use crate::geometry::build_path;
use crate::models::{
    Direction, Layout, Route, COLOR_VEHICLE_LEFT, COLOR_VEHICLE_RIGHT, COLOR_VEHICLE_STRAIGHT,
};

pub struct Vehicle {
    pub id: u64,
    pub origin: Direction,
    pub route: Route,

    path: Vec<(f32, f32)>,
    cum: Vec<f32>,
    pub entry_arc: f32,
    pub exit_arc: f32,
    pub total_len: f32,
    pub progress: f32,

    pub x: f32,
    pub y: f32,
    pub angle: f32,
    pub velocity: f32,
    pub color: (u8, u8, u8),

    pub spawn_time: Instant,
    pub max_velocity_reached: f32,
    pub min_velocity_reached: f32,
}

impl Vehicle {
    pub fn new(id: u64, origin: Direction, route: Route, layout: &Layout) -> Self {
        let (path, cum) = build_path(origin, route, layout);
        let unit = layout.vehicle_width;
        let off = route.lane_offset(unit);

        let entry = crate::geometry::box_entry(origin, off, layout);
        let exit_dir = crate::geometry::exit_direction(origin, route);
        let exit = crate::geometry::box_exit(exit_dir, off, layout);
        let entry_arc = nearest_arc(&path, &cum, entry);
        let exit_arc = nearest_arc(&path, &cum, exit);
        let total_len = *cum.last().unwrap_or(&0.0);

        let color = match route {
            Route::Straight => COLOR_VEHICLE_STRAIGHT,
            Route::Left => COLOR_VEHICLE_LEFT,
            Route::Right => COLOR_VEHICLE_RIGHT,
        };

        let mut v = Self {
            id,
            origin,
            route,
            path,
            cum,
            entry_arc,
            exit_arc,
            total_len,
            progress: 0.0,
            x: 0.0,
            y: 0.0,
            angle: 0.0,
            velocity: 0.0,
            color,
            spawn_time: Instant::now(),
            max_velocity_reached: 0.0,
            min_velocity_reached: f32::MAX,
        };
        v.recompute_position();
        v
    }

    pub fn is_finished(&self) -> bool {
        self.progress >= self.total_len
    }

    pub fn is_in_box(&self) -> bool {
        self.progress >= self.entry_arc && self.progress < self.exit_arc
    }

    pub fn remaining_to_entry(&self) -> f32 {
        self.entry_arc - self.progress
    }

    pub fn has_cleared_box(&self) -> bool {
        self.progress >= self.exit_arc
    }

    pub fn forward_vector(&self) -> (f32, f32) {
        let rad = self.angle.to_radians();
        (rad.sin(), -rad.cos())
    }

    pub fn advance(&mut self, distance: f32) {
        self.progress = (self.progress + distance).clamp(0.0, self.total_len);
        self.recompute_position();
    }

    fn recompute_position(&mut self) {
        let target = self.progress;
        let mut seg = 0;
        while seg + 2 < self.cum.len() && self.cum[seg + 1] < target {
            seg += 1;
        }
        let seg_len = self.cum[seg + 1] - self.cum[seg];
        let t = if seg_len > 0.0 {
            ((target - self.cum[seg]) / seg_len).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let (ax, ay) = self.path[seg];
        let (bx, by) = self.path[seg + 1];
        self.x = ax + (bx - ax) * t;
        self.y = ay + (by - ay) * t;

        let dx = bx - ax;
        let dy = by - ay;
        if dx != 0.0 || dy != 0.0 {
            self.angle = dx.atan2(-dy).to_degrees();
        }
    }

    pub fn resize(&mut self, old: &Layout, new: &Layout) {
        let ratio = new.scale / old.scale;
        for p in &mut self.path {
            let (x, y) = old.remap(new, p.0, p.1);
            p.0 = x;
            p.1 = y;
        }
        for c in &mut self.cum {
            *c *= ratio;
        }
        self.entry_arc *= ratio;
        self.exit_arc *= ratio;
        self.total_len *= ratio;
        self.progress *= ratio;
        self.velocity *= ratio;
        self.recompute_position();
    }
}

fn nearest_arc(pts: &[(f32, f32)], cum: &[f32], target: (f32, f32)) -> f32 {
    let mut best_arc = 0.0;
    let mut best_dist = f32::MAX;
    for (i, p) in pts.iter().enumerate() {
        let d = (p.0 - target.0).powi(2) + (p.1 - target.1).powi(2);
        if d < best_dist {
            best_dist = d;
            best_arc = cum[i];
        }
    }
    best_arc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advance_moves_vehicle_by_velocity_times_time() {
        let layout = Layout::new(800, 800);
        let mut v = Vehicle::new(0, Direction::N, Route::Straight, &layout);

        let velocity: f32 = 120.0;
        let dt: f32 = 0.2;
        let distance = velocity * dt;

        let (x0, y0) = (v.x, v.y);
        let progress0 = v.progress;

        v.velocity = velocity;
        v.advance(distance);

        let progress_moved = v.progress - progress0;
        assert!(
            (progress_moved - distance).abs() < 1e-4,
            "expected progress to move by {distance}, moved {progress_moved}"
        );

        let euclidean_moved = ((v.x - x0).powi(2) + (v.y - y0).powi(2)).sqrt();
        let recovered_velocity = euclidean_moved / dt;
        assert!(
            (recovered_velocity - velocity).abs() < 1.0,
            "expected recovered velocity ~{velocity}, got {recovered_velocity}"
        );
    }

    #[test]
    fn advance_clamps_progress_to_total_length() {
        let layout = Layout::new(800, 800);
        let mut v = Vehicle::new(0, Direction::N, Route::Straight, &layout);

        v.advance(v.total_len + 1_000.0);

        assert_eq!(v.progress, v.total_len);
        assert!(v.is_finished());
    }
}
