pub const COLOR_GROUND: (u8, u8, u8) = (96, 96, 96);
pub const COLOR_ROAD: (u8, u8, u8) = (55, 58, 66);

pub const BASE_DIM: u32 = 800;

pub const ROAD_WIDTH: u32 = 100;


#[derive(Debug, Clone, Copy)]
pub struct Layout {
    pub w: f32,
    pub h: f32,
    pub cx: f32,
    pub cy: f32,
    pub road_width: f32,
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
            road_width: px(ROAD_WIDTH),
        }
    }

    pub fn box_x_min(&self) -> f32 { self.cx - self.road_width / 2.0 }
    pub fn box_y_min(&self) -> f32 { self.cy - self.road_width / 2.0 }
}