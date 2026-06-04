use rand::prelude::*;

use crate::models::Route;

pub fn random_route() -> Route {
    let mut rng = rand::rng();
    match rng.random_range(0..3) {
        0 => Route::Straight,
        1 => Route::Left,
        _ => Route::Right,
    }
}