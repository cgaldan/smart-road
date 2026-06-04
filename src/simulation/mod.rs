use crate::vehicle::Vehicle;
use crate::models::{Direction, Route, Layout};

pub struct Simulation {
    pub vehicles: Vec<Vehicle>,
}

impl Simulation {
    pub fn new() -> Self {
        Self {
            vehicles: Vec::new(),
        }
    }

    pub fn spawn (&mut self, dir: Direction, route: Route, layout: &Layout) {
        self.vehicles.push(Vehicle::new(dir, route, layout));
    }

    pub fn resize(&mut self, old: &Layout, new: &Layout) {
        for v in &mut self.vehicles {
            let (x, y) = old.remap(new, v.x, v.y);
            v.x = x;
            v.y = y;
        }
    }
}