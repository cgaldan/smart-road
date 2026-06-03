pub const COLOR_GROUND: (u8, u8, u8) = (96, 96, 96);

pub const COLOR_ROAD: (u8, u8, u8) = (55, 58, 66);
pub const COLOR_LANE_DIVIDER: (u8, u8, u8) = (200, 200, 160);
pub const COLOR_STOP_LINE: (u8, u8, u8) = (230, 230, 230);

pub const COLOR_LIGHT_RED: (u8, u8, u8) = (220, 40, 40);
// pub const COLOR_LIGHT_GREEN: (u8, u8, u8) = (40, 200, 90);
// pub const COLOR_LIGHT_YELLOW: (u8, u8, u8) = (220, 220, 40);

pub const BASE_DIM: u32 = 800;

pub const ROAD_WIDTH: u32 = 300;
pub const LANE_COUNT: u32 = 6;

pub const DASH_LENGTH: u32 = 30;
pub const DASH_WIDTH: u32 = 2;
pub const DASH_GAP: u32 = 8;

pub const LIGHT_SIZE: u32 = 12;
pub const LIGHT_GAP: u32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    N, S, W, E
}

#[derive(Debug, Clone, Copy)]
pub struct Layout {
    pub w: f32,
    pub h: f32,
    pub cx: f32,
    pub cy: f32,
    pub road_width: f32,
    pub dash_length: f32,
    pub dash_width: f32,
    pub dash_gap: f32,
    pub light_size: f32,
    pub light_gap: f32,
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
            road_width: ROAD_WIDTH as f32 * scale,
            dash_length: DASH_LENGTH as f32 * scale,
            dash_width: px(DASH_WIDTH),
            dash_gap: DASH_GAP as f32 * scale,
            light_size: px(LIGHT_SIZE),
            light_gap: px(LIGHT_GAP),
        }
    }

    pub fn box_x_right(&self) -> f32 { self.cx + self.road_width / 2.0 }
    pub fn box_y_bottom(&self) -> f32 { self.cy + self.road_width / 2.0 }
    pub fn box_x_left(&self) -> f32 { self.cx - self.road_width / 2.0 }
    pub fn box_y_top(&self) -> f32 { self.cy - self.road_width / 2.0 }
}