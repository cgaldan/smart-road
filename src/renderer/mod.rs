mod helpers;

use sdl2::render::{BlendMode, Canvas};
use sdl2::video::Window;
use helpers::{set_color, fill, rect};

use crate::models;

pub fn draw( canvas: &mut Canvas<Window>, layout: &models::Layout) {
    draw_background(canvas);
    draw_roads(canvas, layout);
    canvas.set_blend_mode(BlendMode::Blend);

    canvas.present();
}

fn draw_background(canvas: &mut Canvas<Window>) {
    set_color(canvas, models::COLOR_GROUND);
    canvas.clear();
}

fn draw_roads(canvas: &mut Canvas<Window>, layout: &models::Layout) {
    set_color(canvas, models::COLOR_ROAD);
    fill(canvas, rect(layout.box_x_min(), 0.0, layout.road_width, layout.h));
    fill(canvas, rect(0.0, layout.box_y_min(), layout.w, layout.road_width));
}