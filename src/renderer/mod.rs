pub mod helpers;

use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};

use helpers::{fill, rect, set_color};

use crate::font;
use crate::models;
use crate::simulation::Stats;
use crate::vehicle;

const TEX_W: u32 = 34;
const TEX_H: u32 = 58;

pub struct VehicleTextures<'a> {
    straight: Texture<'a>,
    left: Texture<'a>,
    right: Texture<'a>,
}

pub fn build_vehicle_textures<'a>(
    canvas: &mut Canvas<Window>,
    creator: &'a TextureCreator<WindowContext>,
) -> VehicleTextures<'a> {
    VehicleTextures {
        straight: build_vehicle_texture(canvas, creator, models::COLOR_VEHICLE_STRAIGHT),
        left: build_vehicle_texture(canvas, creator, models::COLOR_VEHICLE_LEFT),
        right: build_vehicle_texture(canvas, creator, models::COLOR_VEHICLE_RIGHT),
    }
}

fn build_vehicle_texture<'a>(
    canvas: &mut Canvas<Window>,
    creator: &'a TextureCreator<WindowContext>,
    color: (u8, u8, u8),
) -> Texture<'a> {
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGBA8888, TEX_W, TEX_H)
        .expect("create_texture_target");
    texture.set_blend_mode(BlendMode::Blend);

    let (r, g, b) = color;
    canvas
        .with_texture_canvas(&mut texture, |tc| {
            tc.set_draw_color(Color::RGBA(0, 0, 0, 0));
            tc.clear();

            // Body.
            tc.set_draw_color(Color::RGB(r, g, b));
            let _ = tc.fill_rect(Rect::new(2, 3, TEX_W - 4, TEX_H - 6));

            // Windshield near the front (top = facing "up", angle 0).
            let light = (
                r.saturating_add(50),
                g.saturating_add(50),
                b.saturating_add(50),
            );
            tc.set_draw_color(Color::RGB(light.0, light.1, light.2));
            let _ = tc.fill_rect(Rect::new(5, 7, TEX_W - 10, 12));

            // Rear bumper, darker.
            let dark = ((r as u32 * 2 / 3) as u8, (g as u32 * 2 / 3) as u8, (b as u32 * 2 / 3) as u8);
            tc.set_draw_color(Color::RGB(dark.0, dark.1, dark.2));
            let _ = tc.fill_rect(Rect::new(5, TEX_H as i32 - 12, TEX_W - 10, 6));
        })
        .expect("with_texture_canvas");

    texture
}

pub fn draw(
    canvas: &mut Canvas<Window>,
    layout: &models::Layout,
    vehicles: &[vehicle::Vehicle],
    textures: &VehicleTextures,
) {
    draw_background(canvas);
    draw_roads(canvas, layout);
    canvas.set_blend_mode(BlendMode::Blend);

    draw_lane_dividers(canvas, layout);
    draw_stop_lines(canvas, layout);
    draw_vehicles(canvas, layout, vehicles, textures);
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

fn draw_vehicles(
    canvas: &mut Canvas<Window>,
    layout: &models::Layout,
    vehicles: &[vehicle::Vehicle],
    textures: &VehicleTextures,
) {
    let w = layout.vehicle_width * 1.3;
    let h = layout.vehicle_length;
    for v in vehicles {
        let tex = match v.route {
            models::Route::Straight => &textures.straight,
            models::Route::Left => &textures.left,
            models::Route::Right => &textures.right,
        };
        let dest = rect(v.x - w / 2.0, v.y - h / 2.0, w, h);
        let _ = canvas.copy_ex(tex, None, Some(dest), v.angle as f64, None, false, false);
    }
}

/// Renders the end-of-simulation statistics screen.
pub fn draw_stats(canvas: &mut Canvas<Window>, w: u32, h: u32, stats: &Stats) {
    set_color(canvas, models::COLOR_GROUND);
    canvas.clear();

    let px = (w.min(h) as f32 / 800.0 * 3.0).max(2.0);
    let line_gap = font::text_height(px) + px * 3.0;
    let mut y = h as f32 * 0.14;
    let x = w as f32 * 0.12;
    let label_color = (235, 235, 235);
    let value_color = (120, 200, 255);

    let title = "SIMULATION STATISTICS";
    let title_px = px * 1.4;
    font::draw_text(
        canvas,
        (w as f32 - font::text_width(title, title_px)) / 2.0,
        y,
        title,
        (255, 210, 90),
        title_px,
    );
    y += font::text_height(title_px) + px * 6.0;

    let min_v = if stats.has_data() { stats.min_velocity } else { 0.0 };
    let min_t = if stats.has_data() { stats.min_time } else { 0.0 };

    let rows = [
        ("VEHICLES PASSED".to_string(), format!("{}", stats.vehicles_passed)),
        ("VEHICLES CREATED".to_string(), format!("{}", stats.vehicles_created)),
        ("MAX VELOCITY".to_string(), format!("{:.1}", stats.max_velocity)),
        ("MIN VELOCITY".to_string(), format!("{:.1}", min_v)),
        ("MAX TIME".to_string(), format!("{:.2}", stats.max_time)),
        ("MIN TIME".to_string(), format!("{:.2}", min_t)),
        ("CLOSE CALLS".to_string(), format!("{}", stats.close_calls)),
    ];

    for (label, value) in rows.iter() {
        font::draw_text(canvas, x, y, label, label_color, px);
        font::draw_text(canvas, x + w as f32 * 0.42, y, value, value_color, px);
        y += line_gap;
    }

    y += px * 6.0;
    let hint = "PRESS ESC OR ENTER TO EXIT";
    font::draw_text(
        canvas,
        (w as f32 - font::text_width(hint, px)) / 2.0,
        y,
        hint,
        (180, 180, 180),
        px,
    );

    canvas.present();
}
