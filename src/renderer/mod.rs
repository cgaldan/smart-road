mod helpers;
mod models;

use sdl2::render::Canvas;
use sdl2::video::Window;

pub fn draw( canvas: &mut Canvas<Window>) {
    draw_background(canvas);

    canvas.present();
}

fn draw_background(canvas: &mut Canvas<Window>) {
    helpers::set_color(canvas, models::COLOR_GROUND);
    canvas.clear();
}