use rand::prelude::*;

use crate::models::{Direction, Route};

pub fn toggle_music(muted: bool) {
    if muted {
        sdl2::mixer::Music::pause();
    } else {
        sdl2::mixer::Music::resume();
    }
}

pub fn random_route() -> Route {
    let mut rng = rand::rng();
    match rng.random_range(0..3) {
        0 => Route::Straight,
        1 => Route::Left,
        _ => Route::Right,
    }
}

pub fn random_direction() -> Direction {
    let mut rng = rand::rng();
    match rng.random_range(0..4) {
        0 => Direction::N,
        1 => Direction::S,
        2 => Direction::E,
        _ => Direction::W,
    }
}
