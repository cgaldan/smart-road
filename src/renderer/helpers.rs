use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

pub fn set_color(canvas: &mut Canvas<Window>, (r, g, b): (u8, u8, u8)) {
    canvas.set_draw_color(Color::RGB(r, g, b));
}

pub fn fill(canvas: &mut Canvas<Window>, rect: Rect) {
    canvas.fill_rect(rect).expect("fill_rect");
}

pub fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::new(
        x.round() as i32,
        y.round() as i32,
        (w.round() as u32).max(1),
        (h.round() as u32).max(1),
    )
}
