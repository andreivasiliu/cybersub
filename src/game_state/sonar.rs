//! Scans the rocks in the world for edges, and runs visibility checks for
//! them.

use serde::{Deserialize, Serialize};

use crate::game_state::{rocks::RockGrid, state::Navigation};

#[derive(Default, Serialize, Deserialize, Clone)]
pub(crate) struct Sonar {
    visible_edge_cells: Vec<(i16, i16)>,
    pulse: usize,
}

impl Sonar {
    pub(crate) fn visible_edge_cells(&self) -> &[(i16, i16)] {
        &self.visible_edge_cells
    }

    pub(crate) fn increase_pulse(&mut self) {
        self.pulse = (self.pulse + 1) % (4 * 30);
    }

    pub(crate) fn pulse(&self) -> f32 {
        self.pulse as f32 / (4.0 * 30.0)
    }

    pub(crate) fn should_update(&self) -> bool {
        self.pulse == 0
    }
}

pub(crate) fn update_sonar(
    sonar: &mut Sonar,
    navigation: &Navigation,
    sub_size: (usize, usize),
    rock_grid: &RockGrid,
) -> bool {
    let center_x = (navigation.position.0 / 16 / 16) as usize;
    let center_y = (navigation.position.1 / 16 / 16) as usize;

    let sub_center_x = center_x + sub_size.0 / 2 / 16;
    let sub_center_y = center_y + sub_size.1 / 2 / 16;

    sonar.increase_pulse();

    if sonar.should_update() {
        find_visible_edge_cells(sonar, (sub_center_x, sub_center_y), rock_grid);
        true
    } else {
        false
    }
}

pub(crate) fn find_visible_edge_cells(
    sonar: &mut Sonar,
    center: (usize, usize),
    rock_grid: &RockGrid,
) {
    sonar.visible_edge_cells.clear();

    let (width, height) = rock_grid.size();
    let center = (center.0.min(width - 1), center.1.min(height - 1));

    let left_edge = center.0.saturating_sub(75);
    let right_edge = center.0.saturating_add(75).min(width - 1);

    let top_edge = center.1.saturating_sub(75);
    let bottom_edge = center.1.saturating_add(75).min(height - 1);

    // Look at the edge cells in region; this averages to checking around 300 cells.
    for y in top_edge..=bottom_edge {
        for x in left_edge..=right_edge {
            let cell = rock_grid.cell(x, y);

            if !cell.is_edge() || distance_squared(x, y, center.0, center.1) > 75 * 75 {
                continue;
            }

            let mut visibility_blocked = false;

            line_cells(x, y, center.0, center.1, |x2, y2| {
                if rock_grid.cell(x2, y2).is_wall() && !is_neighbour(x, y, x2, y2) {
                    visibility_blocked = true;
                }
            });

            if !visibility_blocked {
                sonar
                    .visible_edge_cells
                    .push((center.0 as i16 - x as i16, center.1 as i16 - y as i16));
            }
        }
    }
}

fn is_neighbour(x1: usize, y1: usize, x2: usize, y2: usize) -> bool {
    // Good enough???
    x1 / 4 == x2 / 4 && y1 / 4 == y2 / 4
}

fn distance_squared(x1: usize, y1: usize, x2: usize, y2: usize) -> usize {
    let (x1, x2) = sort(x1, x2);
    let (y1, y2) = sort(y1, y2);

    (x2 - x1) * (x2 - x1) + (y2 - y1) * (y2 - y1)
}

fn sort(a: usize, b: usize) -> (usize, usize) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

/// Bresenham's line algorithm used to find all cells between two points
fn line_cells(x1: usize, y1: usize, x2: usize, y2: usize, mut f: impl FnMut(usize, usize)) {
    let (x1, y1, x2, y2) = (x1 as i32, y1 as i32, x2 as i32, y2 as i32);

    let dx = (x1 - x2).abs();
    let dy = -(y1 - y2).abs();

    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };

    let mut err = dx + dy;

    let mut x = x1;
    let mut y = y1;

    loop {
        f(x as usize, y as usize);

        if x == x2 && y == y2 {
            return;
        }

        if err * 2 > dy {
            err += dy;
            x = x.wrapping_add(sx);
        } else if err * 2 < dx {
            err += dx;
            y = y.wrapping_add(sy);
        }
    }
}
