pub const COLOR_GROUND: (u8, u8, u8) = (96, 96, 96);

pub const COLOR_ROAD: (u8, u8, u8) = (55, 58, 66);
pub const COLOR_LANE_DIVIDER: (u8, u8, u8) = (200, 200, 160);
pub const COLOR_STOP_LINE: (u8, u8, u8) = (230, 230, 230);

pub const COLOR_VEHICLE_STRAIGHT: (u8, u8, u8) = (235, 64, 52);
pub const COLOR_VEHICLE_LEFT: (u8, u8, u8) = (52, 120, 235);
pub const COLOR_VEHICLE_RIGHT: (u8, u8, u8) = (58, 201, 97);

pub const BASE_DIM: u32 = 800;

pub const ROAD_WIDTH: u32 = 300;
pub const LANE_WIDTH: u32 = ROAD_WIDTH / 2;
pub const LANE_COUNT: u32 = 6;

pub const VEHICLE_WIDTH: u32 = LANE_WIDTH / LANE_COUNT;
pub const VEHICLE_LENGTH_RATIO: f32 = 1.7;

pub const DASH_LENGTH: u32 = 30;
pub const DASH_WIDTH: u32 = 2;
pub const DASH_GAP: u32 = 8;
pub const STOP_LINE_THICKNESS: u32 = 2;

// Base (unscaled, BASE_DIM reference) physics constants. All are multiplied
// by `Layout::scale` so behaviour is resolution independent.
pub const BASE_SPEED_SLOW: f32 = 70.0;
pub const BASE_SPEED_MEDIUM: f32 = 140.0;
pub const BASE_SPEED_FAST: f32 = 230.0;

pub const BASE_SAFETY_DISTANCE: f32 = 46.0;
pub const BASE_LANE_TOLERANCE: f32 = 30.0;
pub const BASE_LOOKAHEAD_DISTANCE: f32 = 320.0;
pub const BASE_STOP_MARGIN: f32 = 24.0;

pub const BASE_MAX_ACCEL: f32 = 260.0;
pub const BASE_MAX_DECEL: f32 = 520.0;

pub const MIN_SPAWN_INTERVAL_SECS: f32 = 0.55;
pub const RANDOM_SPAWN_INTERVAL_SECS: f32 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    N, S, W, E
}

impl Direction {
    pub const ALL: [Direction; 4] = [Direction::N, Direction::S, Direction::E, Direction::W];

    pub fn index(&self) -> usize {
        match self {
            Direction::N => 0,
            Direction::S => 1,
            Direction::E => 2,
            Direction::W => 3,
        }
    }

    /// Unit vector pointing in the direction this vehicle travels (screen coords, y grows down).
    pub fn forward(&self) -> (f32, f32) {
        match self {
            Direction::N => (0.0, -1.0),
            Direction::S => (0.0, 1.0),
            Direction::E => (1.0, 0.0),
            Direction::W => (-1.0, 0.0),
        }
    }

    pub fn is_vertical(&self) -> bool {
        matches!(self, Direction::N | Direction::S)
    }

    pub fn turn_right(&self) -> Direction {
        match self {
            Direction::N => Direction::E,
            Direction::E => Direction::S,
            Direction::S => Direction::W,
            Direction::W => Direction::N,
        }
    }

    pub fn turn_left(&self) -> Direction {
        match self {
            Direction::N => Direction::W,
            Direction::W => Direction::S,
            Direction::S => Direction::E,
            Direction::E => Direction::N,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Straight,
    Left,
    Right,
}

impl Route {
    pub const ALL: [Route; 3] = [Route::Straight, Route::Left, Route::Right];

    pub fn index(&self) -> usize {
        match self {
            Route::Straight => 0,
            Route::Left => 1,
            Route::Right => 2,
        }
    }

    /// Offset of this route's lane from the road centerline, in "vehicle width" units.
    pub fn lane_offset(&self, unit: f32) -> f32 {
        match self {
            Route::Left => unit,
            Route::Straight => unit * 3.0,
            Route::Right => unit * 5.0,
        }
    }
}

pub fn combo_index(dir: Direction, route: Route) -> usize {
    dir.index() * 3 + route.index()
}

#[derive(Debug, Clone, Copy)]
pub struct Layout {
    pub w: f32,
    pub h: f32,
    pub cx: f32,
    pub cy: f32,
    pub scale: f32,
    pub road_width: f32,
    pub lane_width: f32,
    pub vehicle_width: f32,
    pub vehicle_length: f32,
    pub dash_length: f32,
    pub dash_width: f32,
    pub dash_gap: f32,
    pub stop_line_thickness: f32,

    pub speed_slow: f32,
    pub speed_medium: f32,
    pub speed_fast: f32,
    pub safety_distance: f32,
    pub lane_tolerance: f32,
    pub lookahead_distance: f32,
    pub stop_margin: f32,
    pub max_accel: f32,
    pub max_decel: f32,
}

impl Layout {
    pub fn new(w: u32, h: u32) -> Self {
        let wf = w.max(1) as f32;
        let hf = h.max(1) as f32;
        let scale = w.min(h).max(1) as f32 / BASE_DIM as f32;
        let px = |base: u32| (base as f32 * scale).max(1.0);
        Self {
            w: wf,
            h: hf,
            cx: wf / 2.0,
            cy: hf / 2.0,
            scale,
            road_width: ROAD_WIDTH as f32 * scale,
            lane_width: LANE_WIDTH as f32 * scale,
            vehicle_width: VEHICLE_WIDTH as f32 * scale,
            vehicle_length: VEHICLE_WIDTH as f32 * scale * VEHICLE_LENGTH_RATIO,
            dash_length: DASH_LENGTH as f32 * scale,
            dash_width: px(DASH_WIDTH),
            dash_gap: DASH_GAP as f32 * scale,
            stop_line_thickness: px(STOP_LINE_THICKNESS),

            speed_slow: BASE_SPEED_SLOW * scale,
            speed_medium: BASE_SPEED_MEDIUM * scale,
            speed_fast: BASE_SPEED_FAST * scale,
            safety_distance: BASE_SAFETY_DISTANCE * scale,
            lane_tolerance: BASE_LANE_TOLERANCE * scale,
            lookahead_distance: BASE_LOOKAHEAD_DISTANCE * scale,
            stop_margin: BASE_STOP_MARGIN * scale,
            max_accel: BASE_MAX_ACCEL * scale,
            max_decel: BASE_MAX_DECEL * scale,
        }
    }

    pub fn box_x_right(&self) -> f32 { self.cx + self.road_width / 2.0 }
    pub fn box_y_bottom(&self) -> f32 { self.cy + self.road_width / 2.0 }
    pub fn box_x_left(&self) -> f32 { self.cx - self.road_width / 2.0 }
    pub fn box_y_top(&self) -> f32 { self.cy - self.road_width / 2.0 }

    pub fn remap(&self, next: &Layout, x: f32, y: f32) -> (f32, f32) {
        let ratio = next.scale / self.scale;
        (
            next.cx + (x - self.cx) * ratio,
            next.cy + (y - self.cy) * ratio
        )
    }
}
