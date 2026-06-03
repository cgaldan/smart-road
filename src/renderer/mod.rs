mod helpers;

use sdl2::render::{BlendMode, Canvas};
use sdl2::video::Window;
use helpers::{set_color, fill, rect};

use crate::models;

pub fn draw( canvas: &mut Canvas<Window>, layout: &models::Layout) {
    draw_background(canvas);
    draw_roads(canvas, layout);
    canvas.set_blend_mode(BlendMode::Blend);

    draw_lane_dividers(canvas, layout);
    draw_stop_lines(canvas, layout);
    canvas.present();
}

fn draw_background(canvas: &mut Canvas<Window>) {
    set_color(canvas, models::COLOR_GROUND);
    canvas.clear();
}

fn draw_roads(canvas: &mut Canvas<Window>, layout: &models::Layout) {
    set_color(canvas, models::COLOR_ROAD);
    fill(canvas, rect(layout.box_x_left(), 0.0, layout.road_width, layout.h));
    fill(canvas, rect(0.0, layout.box_y_top(), layout.w, layout.road_width));
}

fn draw_lane_dividers(canvas: &mut Canvas<Window>, layout: &models::Layout) {
    set_color(canvas, models::COLOR_LANE_DIVIDER);

    draw_main_divider(canvas, layout, layout.cx, layout.cy);
    draw_v_dashes(canvas, layout, layout.cx, 0.0, layout.box_y_top());
    draw_v_dashes(canvas, layout, layout.cx, layout.box_y_bottom(), layout.h);
    draw_h_dashes(canvas, layout, layout.cy, 0.0, layout.box_x_left());
    draw_h_dashes(canvas, layout, layout.cy, layout.box_x_right(), layout.w);
}

fn draw_main_divider(canvas: &mut Canvas<Window>, layout: &models::Layout, cx: f32, cy: f32) {
    set_color(canvas, models::COLOR_LANE_DIVIDER);

    let main_dash_width = layout.dash_width * 2.0;
    fill(canvas, rect(cx, 0.0, main_dash_width, layout.box_y_top()));
    fill(canvas, rect(0.0, cy, layout.box_x_left(), main_dash_width));
    fill(canvas, rect(cx, layout.box_y_bottom(), main_dash_width, layout.h - layout.box_y_bottom()));
    fill(canvas, rect(layout.box_x_right(), cy, layout.w - layout.box_x_right(), main_dash_width));
}

fn draw_v_dashes(canvas: &mut Canvas<Window>, layout: &models::Layout, cx: f32, y_start: f32, y_end: f32) {
    let lane_step = layout.road_width / models::LANE_COUNT as f32;
    let step = layout.dash_length + layout.dash_gap;

    for i in 1..models::LANE_COUNT {
        let x = cx - layout.road_width / 2.0 + lane_step * i as f32;
        let mut y = y_start;

        while y < y_end {
            let len = layout.dash_length.min(y_end - y);
            fill(canvas, rect(x, y, layout.dash_width, len));
            y += step;
        }
    }
}

fn draw_h_dashes(canvas: &mut Canvas<Window>, layout: &models::Layout, cy: f32, x_start: f32, x_end: f32) {
    let lane_step = layout.road_width / models::LANE_COUNT as f32;
    let step = layout.dash_length + layout.dash_gap;

    for i in 1..models::LANE_COUNT {
        let y = cy - layout.road_width / 2.0 + lane_step * i as f32;
        let mut x = x_start;

        while x < x_end {
            let len = layout.dash_length.min(x_end - x);
            fill(canvas, rect(x, y, len, layout.dash_width));
            x += step;
        }
    }
}

fn draw_stop_lines(canvas: &mut Canvas<Window>, layout: &models::Layout) {
    set_color(canvas, models::COLOR_STOP_LINE);

    let stop_line_width = layout.road_width / 2.0;
    fill(canvas, rect(layout.cx, layout.box_y_bottom(), stop_line_width, layout.dash_width));
    fill(canvas, rect(layout.box_x_right(), layout.cy - stop_line_width, layout.dash_width, stop_line_width));
    fill(canvas, rect(layout.cx - stop_line_width, layout.box_y_top() - layout.dash_width, stop_line_width, layout.dash_width));
    fill(canvas, rect(layout.box_x_left() - layout.dash_width, layout.cy, layout.dash_width, stop_line_width));
}