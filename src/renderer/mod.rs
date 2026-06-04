mod helpers;

use sdl2::render::{BlendMode, Canvas};
use sdl2::video::Window;
use helpers::{set_color, fill, rect};

use crate::models;
use crate::vehicle;

pub fn draw( canvas: &mut Canvas<Window>, layout: &models::Layout, vehicles: &Vec<vehicle::Vehicle>) {
    draw_background(canvas);
    draw_roads(canvas, layout);
    canvas.set_blend_mode(BlendMode::Blend);

    draw_lane_dividers(canvas, layout);
    draw_stop_lines(canvas, layout);
    draw_traffic_lights(canvas, layout);
    draw_vehicles(canvas, vehicles);
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
    let nroad = layout.box_y_top();
    let sroad = layout.box_y_bottom();
    let wroad = layout.box_x_left();
    let eroad = layout.box_x_right();
    let w = layout.stop_line_thickness;
    let lw = layout.lane_width;

    fill(canvas, rect(layout.cx, sroad, lw, w));
    fill(canvas, rect(eroad, layout.cy - lw, w, lw));
    fill(canvas, rect(layout.cx - lw, nroad - w, lw, w));
    fill(canvas, rect(wroad - w, layout.cy, w, lw));
}

fn draw_traffic_lights(canvas: &mut Canvas<Window>, layout: &models::Layout) {
    let nroad = layout.box_y_top();
    let sroad = layout.box_y_bottom();
    let wroad = layout.box_x_left();
    let eroad = layout.box_x_right();
    let size = layout.light_size;
    let gap = layout.light_gap;

    let lights = [
        (models::Direction::N, eroad + gap, sroad + gap),
        (models::Direction::S, wroad - gap - size, nroad - gap - size),
        (models::Direction::W, eroad + gap, nroad - gap - size),
        (models::Direction::E, wroad - gap - size, sroad + gap),
    ];

    set_color(canvas, models::COLOR_LIGHT_RED);
    for (dir, x, y) in lights {
        match dir {
            models::Direction::N => fill(canvas, rect(x, y, size, size)),
            models::Direction::S => fill(canvas, rect(x, y, size, size)),
            models::Direction::W => fill(canvas, rect(x, y, size, size)),
            models::Direction::E => fill(canvas, rect(x, y, size, size)),
        }
    }
}

fn draw_vehicles(canvas: &mut Canvas<Window>, vehicles: &Vec<vehicle::Vehicle>) {
    for vehicle in vehicles {
        set_color(canvas, vehicle.color);
        fill(canvas, rect(vehicle.x - vehicle.size / 2.0, vehicle.y - vehicle.size / 2.0, vehicle.size, vehicle.size));
    }
}