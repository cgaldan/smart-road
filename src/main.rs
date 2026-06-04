pub mod renderer;
pub mod models;
pub mod vehicle;

use std::thread::sleep;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::FullscreenType;
use models::{Layout, Direction, Route};
use vehicle::Vehicle;

pub const WINDOW_W: u32 = 800;
pub const WINDOW_H: u32 = 800;
pub const WINDOW_TITLE: &str = "Smart Road";

fn main() {
    let sdl_context = sdl2::init().unwrap();
    
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window(WINDOW_TITLE, WINDOW_W, WINDOW_H)
        .fullscreen()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut fullscreen = true;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::F11),
                    ..
                } => {
                    let next_mode = if fullscreen {
                        FullscreenType::Off
                    } else {
                        FullscreenType::Desktop
                    };

                    if canvas.window_mut().set_fullscreen(next_mode).is_ok() {
                        fullscreen = !fullscreen;
                    }
                }
                _ => {}
            }
        }

        let (w, h) = canvas.window().size();
        let layout = Layout::new(w, h);
        let vehicles = vec![
            Vehicle::new(Direction::N, Route::Straight, &layout),
            Vehicle::new(Direction::N, Route::Left, &layout),
            Vehicle::new(Direction::N, Route::Right, &layout),
            Vehicle::new(Direction::S, Route::Straight, &layout),
            Vehicle::new(Direction::S, Route::Left, &layout),
            Vehicle::new(Direction::S, Route::Right, &layout),
            Vehicle::new(Direction::W, Route::Straight, &layout),
            Vehicle::new(Direction::W, Route::Left, &layout),
            Vehicle::new(Direction::W, Route::Right, &layout),
            Vehicle::new(Direction::E, Route::Straight, &layout),
            Vehicle::new(Direction::E, Route::Left, &layout),
            Vehicle::new(Direction::E, Route::Right, &layout),
        ];

        renderer::draw(&mut canvas, &layout, &vehicles);
        sleep(Duration::from_millis(16));
    }

}
