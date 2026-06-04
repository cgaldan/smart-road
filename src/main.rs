pub mod renderer;
pub mod models;
pub mod vehicle;
pub mod simulation;
pub mod helpers;

use std::thread::sleep;
use std::time::Duration;

use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::video::FullscreenType;
use models::{Layout, Direction};
use helpers::random_route;

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
    
    let mut sim = simulation::Simulation::new();
    let (w, h) = canvas.window().size();
    let mut layout = Layout::new(w, h);

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
                Event::Window {
                    win_event: WindowEvent::SizeChanged(w, h),
                    ..
                } => {
                    let new_layout = Layout::new(w.max(1) as u32, h.max(1) as u32);
                    sim.resize(&layout, &new_layout);
                    layout = new_layout;   
                }
                Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
                    sim.spawn(Direction::N, random_route(), &layout);
                }
                Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
                    sim.spawn(Direction::S, random_route(), &layout);
                }
                Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
                    sim.spawn(Direction::W, random_route(), &layout);
                }
                Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
                    sim.spawn(Direction::E, random_route(), &layout);
                }
                _ => {}
            }
        }

        renderer::draw(&mut canvas, &layout, &sim.vehicles);
        sleep(Duration::from_millis(16));
    }

}
