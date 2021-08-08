use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct WaterGrid {
    cells: Vec<WaterCell>,
    width: usize,
    height: usize,
    total_water: u32,
    edges: Vec<(usize, usize)>,
}

#[derive(Default, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct WaterCell {
    cell_type: CellType,
    planned_transfer: [u32; DIRECTIONS],
}

#[derive(Clone, Copy, Serialize, Deserialize)]
enum CellType {
    Inside {
        level: u32,
        velocity: (i32, i32),
        planned_remaining: u32,
    },
    Wall {
        wall_reflect: [u32; DIRECTIONS],
    },
    Sea,
}

impl Default for CellType {
    fn default() -> Self {
        CellType::Inside {
            level: 0,
            velocity: (0, 0),
            planned_remaining: 0,
        }
    }
}

// Currently static; will eventually be based on sub's depth
const SEA_LEVEL: u32 = 8192;

// Offsets: (y, x), x goes rightwards, y goes downwards
const NEIGHBOUR_OFFSETS: &[(i32, i32)] = &[
    (1, 0),
    // (1, 1),
    (0, 1),
    // (-1, 1),
    (-1, 0),
    // (-1, -1),
    (0, -1),
    // (1, -1),
];

const DIRECTIONS: usize = NEIGHBOUR_OFFSETS.len();

// 1 for 4 directions, 3 for 8 directions (there's three directions with e.g. a positive x)
const INERTIA_SPLIT: u32 = 1;

impl WaterGrid {
    pub fn new(width: usize, height: usize) -> Self {
        let mut cells = Vec::new();
        cells.resize(width * height, WaterCell::default());

        for x in 1..width - 1 {
            cells[(height - 2) * width + x].make_wall();
        }

        for y in (height * 50 / 100)..height - 1 {
            cells[y * width + 1].make_wall();
            cells[y * width + width - 2].make_wall();

            if y < height * 90 / 100 {
                cells[y * width + width * 3 / 4].make_wall();
            }
        }

        // Disabled: Needs 8 neighbours
        // for y in 1..height - 1 {
        //     for x in 1..width - 1 {
        //         if !cells[y * width + x].is_wall() {
        //             continue;
        //         }

        //         for direction in 0..DIRECTIONS {
        //             let mut open_cells = 0;
        //             for near_direction in &[DIRECTIONS - 1, 0, 1] {
        //                 let near_direction = (direction + near_direction) % DIRECTIONS;
        //                 let (y_offset, x_offset) = NEIGHBOUR_OFFSETS[near_direction];

        //                 let cell = &cells[(y as i32 + y_offset) as usize * width
        //                     + (x as i32 + x_offset) as usize];
        //                 if !cell.is_wall() {
        //                     open_cells += 1;
        //                 }
        //             }
        //             cells[y * width + x].wall_open_cells[direction] = open_cells;
        //         }
        //     }
        // }

        WaterGrid {
            cells,
            width,
            height,
            total_water: 0,
            edges: Vec::new(),
        }
    }

    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn cell(&self, x: usize, y: usize) -> &WaterCell {
        debug_assert!(x < self.width);
        debug_assert!(y < self.height);

        &self.cells[y * self.width + x]
    }

    pub fn cell_mut(&mut self, x: usize, y: usize) -> &mut WaterCell {
        debug_assert!(x < self.width);
        debug_assert!(y < self.height);

        &mut self.cells[y * self.width + x]
    }

    pub fn total_water(&self) -> u32 {
        self.total_water
    }

    fn neighbours(&self, x: usize, y: usize) -> impl Iterator<Item = &WaterCell> {
        NEIGHBOUR_OFFSETS.iter().map(move |(y_offset, x_offset)| {
            self.cell(
                (x as i32 + x_offset) as usize,
                (y as i32 + y_offset) as usize,
            )
        })
    }

    // pub fn debug_cell(&self, x: usize, y: usize) {
    //     let cell = self.cell(x, y);

    //     dbg!((cell.level, cell.velocity_x, cell.velocity_y));

    //     eprintln!(
    //         "Bottom: {}, right: {}, top: {}, left: {}",
    //         cell.gravity_surplus(0),
    //         cell.gravity_surplus(1),
    //         cell.gravity_surplus(2),
    //         cell.gravity_surplus(3),
    //     );
    // }

    pub fn clear(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let cell = self.cell_mut(x, y);

                match &mut cell.cell_type {
                    CellType::Inside {
                        level,
                        velocity,
                        planned_remaining,
                    } => {
                        *level = 0;
                        *velocity = (0, 0);
                        *planned_remaining = 0;
                    }
                    CellType::Wall { wall_reflect } => *wall_reflect = [0; DIRECTIONS],
                    CellType::Sea => (),
                }
            }
        }
    }

    pub fn update(&mut self, enable_gravity: bool, enable_inertia: bool) {
        let mut new_grid = WaterGrid::new(self.width, self.height);
        std::mem::swap(self, &mut new_grid);
        let old_grid = new_grid;

        let mut total_water = 0;

        for y in 1..old_grid.height - 1 {
            for x in 1..old_grid.width - 1 {
                let old_cell = old_grid.cell(x, y);
                let new_cell = self.cell_mut(x, y);

                match old_cell.cell_type {
                    CellType::Wall { .. } => {
                        let mut wall_reflect = [0; DIRECTIONS];

                        for (i, neighbour) in old_grid.neighbours(x, y).enumerate() {
                            if neighbour.is_inside() {
                                let opposite_direction = (i + DIRECTIONS / 2) % DIRECTIONS;
                                let incoming_water = neighbour.planned_transfer[opposite_direction];
                                wall_reflect[i] = incoming_water;
                                total_water += incoming_water;
                            }
                        }

                        new_cell.cell_type = CellType::Wall { wall_reflect };
                        new_cell.replan();
                    }
                    CellType::Sea => {
                        new_cell.cell_type = CellType::Sea;
                        new_cell.replan();
                    }
                    CellType::Inside {
                        velocity: old_velocity,
                        planned_remaining,
                        ..
                    } => {
                        let mut level = planned_remaining;
                        let mut velocity = (0, 0);

                        // Gather water from neighbouring cells
                        for (i, neighbour) in old_grid.neighbours(x, y).enumerate() {
                            let opposite_direction = (i + DIRECTIONS / 2) % DIRECTIONS;
                            let incoming_water = neighbour.planned_transfer[opposite_direction];
                            level += incoming_water;

                            if enable_inertia {
                                velocity.0 += incoming_water as i32 * -NEIGHBOUR_OFFSETS[i].1;
                                velocity.1 += incoming_water as i32 * -NEIGHBOUR_OFFSETS[i].0;
                            }
                        }

                        if enable_gravity && !old_grid.cell(x, y + 1).is_wall() {
                            velocity.1 += 32;
                        }

                        let velocity = (
                            (old_velocity.0 * 3 + velocity.0) / 4,
                            (old_velocity.1 * 3 + velocity.1) / 4,
                        );
                        new_cell.cell_type = CellType::Inside {
                            level,
                            velocity,
                            planned_remaining: 0,
                        };

                        // Plan water to be sent to neighbouring cells on next update
                        new_cell.replan();

                        total_water += level;
                    }
                }
            }
        }

        self.total_water = total_water;

        // The grid edges weren't processed by the above loop
        for x in 0..self.width {
            self.cell_mut(x, 0).make_sea();
            self.cell_mut(x, self.height - 1).make_sea();
        }

        for y in 0..self.height {
            self.cell_mut(0, y).make_sea();
            self.cell_mut(self.width - 1, y).make_sea();
        }

        // Edge walls (or walls in general) stay the same on a grid update
        self.edges = old_grid.edges;
    }

    pub fn update_edges(&mut self) {
        self.edges.clear();

        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                if self.cell(x, y).is_wall() {
                    let edge = self.neighbours(x, y).any(|cell| cell.is_sea());
                    if edge {
                        self.edges.push((x, y));
                    }
                }
            }
        }
    }

    pub fn edges(&self) -> &[(usize, usize)] {
        &self.edges
    }
}

impl WaterCell {
    fn level(&self) -> u32 {
        match self.cell_type {
            CellType::Inside { level, .. } => level,
            CellType::Wall { .. } => 0,
            CellType::Sea => SEA_LEVEL,
        }
    }

    // If the level changed, then recompute all transfer/reflect plans to match
    fn replan(&mut self) {
        match &mut self.cell_type {
            CellType::Wall { wall_reflect } => {
                self.planned_transfer = *wall_reflect;
            }
            CellType::Sea => {
                for direction in 0..DIRECTIONS {
                    self.planned_transfer[direction] = SEA_LEVEL / DIRECTIONS as u32;
                }
            }
            CellType::Inside {
                level,
                velocity,
                planned_remaining,
            } => {
                // This amount will leave the cell due to overpressure
                let pressure_surplus = level.max(&mut 1024).wrapping_sub(1024);

                for direction in 0..DIRECTIONS {
                    self.planned_transfer[direction] = pressure_surplus / DIRECTIONS as u32;
                }
                self.planned_transfer[0] += pressure_surplus % DIRECTIONS as u32;

                // This amount will leave the cell due to inertia/gravity
                let inertia = {
                    let should_leave = (velocity.0.abs() + velocity.1.abs()) as u32;
                    should_leave.min(*level - pressure_surplus)
                };

                // This amount will remain in the cell
                *planned_remaining = *level - pressure_surplus - inertia;

                // This is how much will leave in other directions
                let mut velocity_x = velocity.0;
                let mut velocity_y = velocity.1;

                let total_velocity = velocity_x.abs() + velocity_y.abs();

                if total_velocity != 0 {
                    velocity_x = velocity_x * inertia as i32 / total_velocity;
                    velocity_y = velocity_y * inertia as i32 / total_velocity;

                    let leftover = inertia as i32 - velocity_x.abs() - velocity_y.abs();
                    velocity_y += leftover * velocity_y.signum();

                    for (direction, neighbour_offset) in NEIGHBOUR_OFFSETS.iter().enumerate() {
                        let surplus_x =
                            (velocity_x * neighbour_offset.1).max(0) as u32 / INERTIA_SPLIT;
                        let surplus_y =
                            (velocity_y * neighbour_offset.0).max(0) as u32 / INERTIA_SPLIT;

                        self.planned_transfer[direction] += surplus_x + surplus_y;
                    }
                }
            }
        };
    }

    pub fn amount_filled(&self) -> f32 {
        self.level().min(1024) as f32 / 1024.0
    }

    pub fn amount_overfilled(&self) -> f32 {
        let level = self.level();

        if level > 1024 {
            (level - 1024).min(4096) as f32 / 4096.0
        } else {
            0.0
        }
    }

    pub fn velocity(&self) -> (f32, f32) {
        match self.cell_type {
            CellType::Inside { velocity, .. } => (velocity.0 as f32, velocity.1 as f32),
            CellType::Wall { .. } => (0.0, 0.0),
            CellType::Sea => (0.0, 0.0),
        }
    }

    pub fn fill(&mut self) {
        if let CellType::Inside { ref mut level, .. } = self.cell_type {
            *level += 16 * 1024;
        }
        self.replan();
    }

    pub fn is_wall(&self) -> bool {
        matches!(self.cell_type, CellType::Wall { .. })
    }

    pub fn is_inside(&self) -> bool {
        matches!(self.cell_type, CellType::Inside { .. })
    }

    pub fn is_sea(&self) -> bool {
        matches!(self.cell_type, CellType::Sea)
    }

    pub fn make_wall(&mut self) {
        self.cell_type = CellType::Wall {
            wall_reflect: [0; DIRECTIONS],
        };
    }

    pub fn make_sea(&mut self) {
        self.cell_type = CellType::Sea
    }

    pub fn make_inside(&mut self) {
        self.cell_type = CellType::Inside {
            level: 0,
            velocity: (0, 0),
            planned_remaining: 0,
        };
        self.replan();
    }

    pub fn clear_wall(&mut self) {
        if self.is_wall() {
            self.make_inside();
        }
    }

    pub fn add_level(&mut self, difference: i32) {
        match self.cell_type {
            CellType::Inside { ref mut level, .. } => {
                if difference >= 0 {
                    *level = level.saturating_add(difference as u32).min(8096);
                } else {
                    *level = level.saturating_sub(difference.abs() as u32);
                }
                self.replan();
            }
            CellType::Wall { .. } => (),
            CellType::Sea => (),
        }
    }
}
