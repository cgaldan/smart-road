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

struct Sprite<'a> {
    texture: Texture<'a>,
    frames: u32,
}

impl Sprite<'_> {
    fn frame_source(&self, clock: f32) -> Rect {
        let side = self.texture.query().height;
        let frame = if self.frames <= 1 {
            0
        } else {
            (clock * models::ENGINE_ANIMATION_FPS) as u32 % self.frames
        };
        Rect::new((frame * side) as i32, 0, side, side)
    }
}

struct ShipSprite<'a> {
    engine: Sprite<'a>,
    hull: Sprite<'a>,
}

pub struct VehicleTextures<'a> {
    ships: [ShipSprite<'a>; 6],
}

pub struct BackgroundTextures<'a> {
    nebula: Texture<'a>,
    stars: Texture<'a>,
}

pub struct MusicControlTextures<'a> {
    sound_on: Texture<'a>,
    muted: Texture<'a>,
}

pub const MUTE_BUTTON_SIZE: u32 = 44;
const MUTE_BUTTON_MARGIN: i32 = 12;

pub fn mute_button_rect(canvas_width: u32) -> Rect {
    Rect::new(
        canvas_width as i32 - MUTE_BUTTON_SIZE as i32 - MUTE_BUTTON_MARGIN,
        MUTE_BUTTON_MARGIN,
        MUTE_BUTTON_SIZE,
        MUTE_BUTTON_SIZE,
    )
}

pub fn build_vehicle_textures<'a>(
    creator: &'a TextureCreator<WindowContext>,
) -> VehicleTextures<'a> {
    VehicleTextures {
        ships: [
            load_ship(
                creator,
                "frigate",
                include_bytes!("../../assets/frigate_engine.png"),
                include_bytes!("../../assets/frigate.png"),
            ),
            load_ship(
                creator,
                "battlecruiser",
                include_bytes!("../../assets/battlecruiser_engine.png"),
                include_bytes!("../../assets/battlecruiser.png"),
            ),
            load_ship(
                creator,
                "scout",
                include_bytes!("../../assets/scout_engine.png"),
                include_bytes!("../../assets/scout.png"),
            ),
            load_ship(
                creator,
                "torpedo ship",
                include_bytes!("../../assets/torpedoShip_engine.png"),
                include_bytes!("../../assets/torpedoShip.png"),
            ),
            load_ship(
                creator,
                "dreadnought",
                include_bytes!("../../assets/dreadnought_engine.png"),
                include_bytes!("../../assets/dreadnought.png"),
            ),
            load_ship(
                creator,
                "battlecruiser 2",
                include_bytes!("../../assets/battlecruiser2_engine.png"),
                include_bytes!("../../assets/battlecruiser2.png"),
            ),
        ],
    }
}

pub fn build_background_textures<'a>(
    creator: &'a TextureCreator<WindowContext>,
) -> BackgroundTextures<'a> {
    let nebula = load_texture(
        creator,
        "nebula background",
        include_bytes!("../../assets/bg_nebula.png"),
    );
    let mut stars = load_texture(
        creator,
        "starfield background",
        include_bytes!("../../assets/bg_stars.png"),
    );
    stars.set_blend_mode(BlendMode::Add);
    BackgroundTextures { nebula, stars }
}

pub fn build_music_control_textures<'a>(
    creator: &'a TextureCreator<WindowContext>,
) -> MusicControlTextures<'a> {
    MusicControlTextures {
        sound_on: load_texture(
            creator,
            "sound on icon",
            include_bytes!("../../assets/volume.png"),
        ),
        muted: load_texture(
            creator,
            "muted icon",
            include_bytes!("../../assets/volume-mute.png"),
        ),
    }
}

fn load_ship<'a>(
    creator: &'a TextureCreator<WindowContext>,
    name: &str,
    engine_png: &[u8],
    hull_png: &[u8],
) -> ShipSprite<'a> {
    ShipSprite {
        engine: load_sprite(creator, &format!("{name} engine"), engine_png),
        hull: load_sprite(creator, name, hull_png),
    }
}

fn load_sprite<'a>(
    creator: &'a TextureCreator<WindowContext>,
    name: &str,
    png: &[u8],
) -> Sprite<'a> {
    let texture = load_texture(creator, name, png);
    let query = texture.query();
    assert!(
        query.height > 0 && query.width % query.height == 0,
        "{name} sprite must be a horizontal strip of square frames"
    );
    Sprite {
        texture,
        frames: query.width / query.height,
    }
}

fn load_texture<'a>(
    creator: &'a TextureCreator<WindowContext>,
    name: &str,
    png: &[u8],
) -> Texture<'a> {
    let image = image::load_from_memory_with_format(png, image::ImageFormat::Png)
        .unwrap_or_else(|error| panic!("decode {name} sprite: {error}"))
        .to_rgba8();
    let (width, height) = image.dimensions();
    let mut texture = creator
        .create_texture_static(PixelFormatEnum::ABGR8888, width, height)
        .unwrap_or_else(|error| panic!("create {name} texture: {error}"));
    texture
        .update(None, image.as_raw(), width as usize * 4)
        .unwrap_or_else(|error| panic!("upload {name} texture: {error}"));
    texture.set_blend_mode(BlendMode::Blend);
    texture
}

pub fn draw(
    canvas: &mut Canvas<Window>,
    layout: &models::Layout,
    vehicles: &[vehicle::Vehicle],
    textures: &VehicleTextures,
    background: &BackgroundTextures,
    music_controls: &MusicControlTextures,
    muted: bool,
    animation_time: f32,
) {
    draw_background(canvas, background, animation_time);
    draw_roads(canvas, layout);
    canvas.set_blend_mode(BlendMode::Blend);

    draw_lane_dividers(canvas, layout);
    draw_stop_lines(canvas, layout);
    draw_vehicles(canvas, layout, vehicles, textures, animation_time);
    draw_mute_button(canvas, music_controls, muted);
    canvas.present();
}

fn draw_background(
    canvas: &mut Canvas<Window>,
    background: &BackgroundTextures,
    animation_time: f32,
) {
    set_color(canvas, models::COLOR_GROUND);
    canvas.clear();
    scroll_background_layer(
        canvas,
        &background.nebula,
        models::BACKGROUND_NEBULA_TILE,
        models::BACKGROUND_SCROLL_X,
        models::BACKGROUND_SCROLL_Y,
        animation_time,
    );
    scroll_background_layer(
        canvas,
        &background.stars,
        models::BACKGROUND_STARS_TILE,
        models::BACKGROUND_SCROLL_X * models::BACKGROUND_STARS_PARALLAX,
        models::BACKGROUND_SCROLL_Y * models::BACKGROUND_STARS_PARALLAX,
        animation_time,
    );
}

fn scroll_background_layer(
    canvas: &mut Canvas<Window>,
    texture: &Texture,
    tile_size: u32,
    velocity_x: f32,
    velocity_y: f32,
    animation_time: f32,
) {
    let (canvas_width, canvas_height) = canvas.output_size().unwrap_or((0, 0));
    let tile = tile_size as i32;
    let offset_x = (animation_time * velocity_x).rem_euclid(tile_size as f32) as i32;
    let offset_y = (animation_time * velocity_y).rem_euclid(tile_size as f32) as i32;
    let columns = canvas_width as i32 / tile + 2;
    let rows = canvas_height as i32 / tile + 2;

    for row in 0..rows {
        for column in 0..columns {
            let destination = Rect::new(
                column * tile - offset_x,
                row * tile - offset_y,
                tile_size,
                tile_size,
            );
            let _ = canvas.copy(texture, None, Some(destination));
        }
    }
}

fn draw_roads(canvas: &mut Canvas<Window>, layout: &models::Layout) {
    let (red, green, blue) = models::COLOR_ROAD;
    canvas.set_draw_color(Color::RGBA(red, green, blue, models::ROAD_ALPHA));
    fill(canvas, rect(layout.box_x_left(), 0.0, layout.road_width, layout.h));
    fill(canvas, rect(0.0, layout.box_y_top(), layout.w, layout.road_width));
}

fn draw_mute_button(
    canvas: &mut Canvas<Window>,
    controls: &MusicControlTextures,
    muted: bool,
) {
    let (canvas_width, _) = canvas.output_size().unwrap_or((0, 0));
    let button = mute_button_rect(canvas_width);

    canvas.set_draw_color(Color::RGBA(230, 230, 235, 200));
    fill(canvas, button);

    let padding = MUTE_BUTTON_SIZE as i32 / 6;
    let icon = Rect::new(
        button.x() + padding,
        button.y() + padding,
        (MUTE_BUTTON_SIZE as i32 - 2 * padding).max(1) as u32,
        (MUTE_BUTTON_SIZE as i32 - 2 * padding).max(1) as u32,
    );
    let texture = if muted {
        &controls.muted
    } else {
        &controls.sound_on
    };
    let _ = canvas.copy(texture, None, Some(icon));
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
    animation_time: f32,
) {
    let size = layout.vehicle_length * models::VEHICLE_SPRITE_SCALE;
    for v in vehicles {
        let route_ship = match v.route {
            models::Route::Left => 0,
            models::Route::Straight => 1,
            models::Route::Right => 2,
        };
        let variant = (v.id as usize) % 2;
        let ship = &textures.ships[route_ship + variant * 3];
        let dest = rect(v.x - size / 2.0, v.y - size / 2.0, size, size);
        let engine_source = ship.engine.frame_source(animation_time);
        let hull_source = ship.hull.frame_source(animation_time);
        let _ = canvas.copy_ex(
            &ship.engine.texture,
            Some(engine_source),
            Some(dest),
            v.angle as f64,
            None,
            false,
            false,
        );
        let _ = canvas.copy_ex(
            &ship.hull.texture,
            Some(hull_source),
            Some(dest),
            v.angle as f64,
            None,
            false,
            false,
        );
    }
}

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
        ("COLLISIONS".to_string(), format!("{}", stats.collisions)),
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
