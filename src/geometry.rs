use crate::models::{Direction, Layout, Route};

pub fn box_entry(dir: Direction, off: f32, layout: &Layout) -> (f32, f32) {
    match dir {
        Direction::N => (layout.cx + off, layout.box_y_bottom()),
        Direction::S => (layout.cx - off, layout.box_y_top()),
        Direction::E => (layout.box_x_left(), layout.cy + off),
        Direction::W => (layout.box_x_right(), layout.cy - off),
    }
}

pub fn box_exit(dir: Direction, off: f32, layout: &Layout) -> (f32, f32) {
    match dir {
        Direction::N => (layout.cx + off, layout.box_y_top()),
        Direction::S => (layout.cx - off, layout.box_y_bottom()),
        Direction::E => (layout.box_x_right(), layout.cy + off),
        Direction::W => (layout.box_x_left(), layout.cy - off),
    }
}

pub fn spawn_point(dir: Direction, off: f32, size: f32, layout: &Layout) -> (f32, f32) {
    match dir {
        Direction::N => (layout.cx + off, layout.h - size),
        Direction::S => (layout.cx - off, size),
        Direction::E => (size, layout.cy + off),
        Direction::W => (layout.w - size, layout.cy - off),
    }
}

pub fn end_point(dir: Direction, off: f32, size: f32, layout: &Layout) -> (f32, f32) {
    match dir {
        Direction::N => (layout.cx + off, -size),
        Direction::S => (layout.cx - off, layout.h + size),
        Direction::E => (layout.w + size, layout.cy + off),
        Direction::W => (-size, layout.cy - off),
    }
}

pub fn exit_direction(dir: Direction, route: Route) -> Direction {
    match route {
        Route::Straight => dir,
        Route::Left => dir.turn_left(),
        Route::Right => dir.turn_right(),
    }
}

const CURVE_SAMPLES: usize = 16;

pub fn build_path(dir: Direction, route: Route, layout: &Layout) -> (Vec<(f32, f32)>, Vec<f32>) {
    let unit = layout.vehicle_width;
    let off = route.lane_offset(unit);
    let p0 = spawn_point(dir, off, unit, layout);
    let entry = box_entry(dir, off, layout);

    let mut pts = vec![p0];

    match route {
        Route::Straight => {
            let exit = box_exit(dir, off, layout);
            let end = end_point(dir, off, unit, layout);
            pts.push(entry);
            pts.push(exit);
            pts.push(end);
        }
        Route::Left | Route::Right => {
            let dir2 = exit_direction(dir, route);
            let exit = box_exit(dir2, off, layout);
            let end = end_point(dir2, off, unit, layout);

            let control = if dir.is_vertical() {
                (entry.0, exit.1)
            } else {
                (exit.0, entry.1)
            };

            for i in 0..=CURVE_SAMPLES {
                let t = i as f32 / CURVE_SAMPLES as f32;
                let mt = 1.0 - t;
                let x = mt * mt * entry.0 + 2.0 * mt * t * control.0 + t * t * exit.0;
                let y = mt * mt * entry.1 + 2.0 * mt * t * control.1 + t * t * exit.1;
                pts.push((x, y));
            }
            pts.push(end);
        }
    }

    let mut cum = vec![0.0f32; pts.len()];
    for i in 1..pts.len() {
        let (ax, ay) = pts[i - 1];
        let (bx, by) = pts[i];
        cum[i] = cum[i - 1] + ((bx - ax).powi(2) + (by - ay).powi(2)).sqrt();
    }

    (pts, cum)
}

fn resample(pts: &[(f32, f32)], cum: &[f32], n: usize) -> Vec<(f32, f32)> {
    let total = *cum.last().unwrap_or(&0.0);
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let target = total * i as f32 / (n.max(2) - 1) as f32;
        let mut seg = 0;
        while seg + 1 < cum.len() && cum[seg + 1] < target {
            seg += 1;
        }
        let seg_len = cum[seg + 1] - cum[seg];
        let t = if seg_len > 0.0 { (target - cum[seg]) / seg_len } else { 0.0 };
        let (ax, ay) = pts[seg];
        let (bx, by) = pts[seg + 1];
        out.push((ax + (bx - ax) * t, ay + (by - ay) * t));
    }
    out
}

pub fn build_conflict_table(threshold: f32) -> Vec<Vec<bool>> {
    let layout = Layout::new(crate::models::BASE_DIM, crate::models::BASE_DIM);
    let combos: Vec<(Direction, Route)> = Direction::ALL
        .iter()
        .flat_map(|d| Route::ALL.iter().map(move |r| (*d, *r)))
        .collect();

    let box_paths: Vec<Vec<(f32, f32)>> = combos
        .iter()
        .map(|(d, r)| {
            let (pts, cum) = build_path(*d, *r, &layout);
            let unit = layout.vehicle_width;
            let off = r.lane_offset(unit);
            let entry_arc = {
                let (ex, ey) = box_entry(*d, off, &layout);
                nearest_arc(&pts, &cum, (ex, ey))
            };
            let exit_arc = {
                let d2 = exit_direction(*d, *r);
                let (ex, ey) = box_exit(d2, off, &layout);
                nearest_arc(&pts, &cum, (ex, ey))
            };
            let sub = sub_path(&pts, &cum, entry_arc, exit_arc);
            resample(&sub.0, &sub.1, 24)
        })
        .collect();

    let n = combos.len();
    let mut table = vec![vec![false; n]; n];
    for i in 0..n {
        for j in (i + 1)..n {
            if combos[i].0 == combos[j].0 {
                continue;
            }
            let mut min_dist = f32::MAX;
            for a in &box_paths[i] {
                for b in &box_paths[j] {
                    let d = ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt();
                    if d < min_dist {
                        min_dist = d;
                    }
                }
            }
            let conflict = min_dist < threshold;
            table[i][j] = conflict;
            table[j][i] = conflict;
        }
    }
    table
}

fn nearest_arc(pts: &[(f32, f32)], cum: &[f32], target: (f32, f32)) -> f32 {
    let mut best_arc = 0.0;
    let mut best_dist = f32::MAX;
    for (i, p) in pts.iter().enumerate() {
        let d = (p.0 - target.0).powi(2) + (p.1 - target.1).powi(2);
        if d < best_dist {
            best_dist = d;
            best_arc = cum[i];
        }
    }
    best_arc
}

fn sub_path(pts: &[(f32, f32)], cum: &[f32], from_arc: f32, to_arc: f32) -> (Vec<(f32, f32)>, Vec<f32>) {
    let mut out_pts = Vec::new();
    let mut out_cum = Vec::new();
    for (i, &arc) in cum.iter().enumerate() {
        if arc >= from_arc && arc <= to_arc {
            out_pts.push(pts[i]);
            out_cum.push(arc - from_arc);
        }
    }
    if out_pts.is_empty() {
        out_pts.push(pts[0]);
        out_cum.push(0.0);
    }
    (out_pts, out_cum)
}
