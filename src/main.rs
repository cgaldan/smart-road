pub mod renderer;
pub mod models;
pub mod vehicle;
pub mod simulation;
pub mod helpers;
pub mod geometry;
pub mod font;

use std::thread::sleep;
use std::time::{Duration, Instant};

use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::video::FullscreenType;
use models::{Direction, Layout};

pub const WINDOW_W: u32 = 800;
pub const WINDOW_H: u32 = 800;
pub const WINDOW_TITLE: &str = "Smart Road";

#[derive(PartialEq, Eq)]
enum AppState {
    Running,
    Stats,
}

fn main() {
    let sdl_context = sdl2::init().unwrap();

    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window(WINDOW_TITLE, WINDOW_W, WINDOW_H)
        .fullscreen()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let vehicle_textures = renderer::build_vehicle_textures(&mut canvas, &texture_creator);
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut fullscreen = true;
    let mut state = AppState::Running;

    let mut sim = simulation::Simulation::new();
    let (w, h) = canvas.window().size();
    let mut layout = Layout::new(w, h);

    let mut last_frame = Instant::now();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode: Some(Keycode::Escape), repeat: false, .. } => match state {
                    AppState::Running => {
                        sim.finalize_stats();
                        state = AppState::Stats;
                    }
                    AppState::Stats => break 'running,
                },
                Event::KeyDown { keycode: Some(Keycode::Return), repeat: false, .. }
                    if state == AppState::Stats =>
                {
                    break 'running;
                }
                Event::KeyDown { keycode: Some(Keycode::F11), repeat: false, .. } => {
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
                Event::KeyDown { keycode: Some(Keycode::Up), repeat: false, .. }
                    if state == AppState::Running =>
                {
                    sim.try_spawn(Direction::N, helpers::random_route(), &layout);
                }
                Event::KeyDown { keycode: Some(Keycode::Down), repeat: false, .. }
                    if state == AppState::Running =>
                {
                    sim.try_spawn(Direction::S, helpers::random_route(), &layout);
                }
                Event::KeyDown { keycode: Some(Keycode::Left), repeat: false, .. }
                    if state == AppState::Running =>
                {
                    sim.try_spawn(Direction::W, helpers::random_route(), &layout);
                }
                Event::KeyDown { keycode: Some(Keycode::Right), repeat: false, .. }
                    if state == AppState::Running =>
                {
                    sim.try_spawn(Direction::E, helpers::random_route(), &layout);
                }
                Event::KeyDown { keycode: Some(Keycode::R), repeat: false, .. }
                    if state == AppState::Running =>
                {
                    sim.toggle_random();
                }
                _ => {}
            }
        }

        let now = Instant::now();
        let dt = (now - last_frame).as_secs_f32().min(0.1);
        last_frame = now;

        match state {
            AppState::Running => {
                sim.update(dt, &layout);
                renderer::draw(&mut canvas, &layout, &sim.vehicles, &vehicle_textures);
            }
            AppState::Stats => {
                let (w, h) = canvas.window().size();
                renderer::draw_stats(&mut canvas, w, h, &sim.stats);
            }
        }

        sleep(Duration::from_millis(16));
    }
}
