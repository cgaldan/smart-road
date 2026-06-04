use crate::models::{COLOR_VEHICLE_LEFT, COLOR_VEHICLE_RIGHT, COLOR_VEHICLE_STRAIGHT, Direction, LANE_COUNT, Layout, Route};

pub struct Vehicle {
    pub heading: Direction,
    pub route: Route,
    pub x: f32,
    pub y: f32,
    pub turned: bool,
    pub color: (u8, u8, u8),
    pub size: f32,
}

impl Vehicle {
    pub fn new(heading: Direction, route: Route, layout: &Layout) -> Self {
        let size = layout.lane_width / LANE_COUNT as f32;
        let lw = match route {
            Route::Straight => size * 3.0,
            Route::Left => size,
            Route::Right => size * 5.0,
        };
        let (x, y) = match heading {
            Direction::N => (layout.cx + lw, layout.h - size),
            Direction::S => (layout.cx - lw, size),
            Direction::W => (size, layout.cy + lw),
            Direction::E => (layout.w - size, layout.cy - lw),
        };
        let base = match route {
            Route::Straight => COLOR_VEHICLE_STRAIGHT,
            Route::Left => COLOR_VEHICLE_LEFT,
            Route::Right => COLOR_VEHICLE_RIGHT,
        };
        Self {
            heading,
            route,
            x,
            y,
            turned: false,
            color: base,
            size,
        }
    }
}