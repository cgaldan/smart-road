use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::pixels::Color;

pub fn set_color(canvas: &mut Canvas<Window>, (r, g, b): (u8, u8, u8)) {
    canvas.set_draw_color(Color::RGB(r, g, b));
}