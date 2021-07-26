use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct WaterGrid {
    cells: Vec<WaterCell>,
    width: usize,
    height: usize,
}

#[derive(Default, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct WaterCell {
    cell_type: CellType,
    velocity_x: i32,
    velocity_y: i32,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
enum CellType {
    Inside { level: u32 },
    Wall { wall_reflect: [u32; DIRECTIONS] },
    Sea,
}

impl Default for CellType {
    fn default() -> Self {
        CellType::Inside { level: 0 }
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
        let mut total = 0;

        for y in 0..self.height {
            for x in 0..self.width {
                let cell = self.cell(x, y);

                if let CellType::Inside { level } = cell.cell_type {
                    total += level;
                } else if let CellType::Wall { ref wall_reflect } = cell.cell_type {
                    for dir in 0..DIRECTIONS {
                        total += wall_reflect[dir];
                    }
                }
            }
        }

        total
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
                    CellType::Inside { level } => *level = 0,
                    CellType::Wall { wall_reflect } => *wall_reflect = [0; DIRECTIONS],
                    CellType::Sea => (),
                }
            }
        }
    }

    pub fn update(&mut self, enable_gravity: bool, enable_inertia: bool) {
        let old_grid = self.clone();

        for y in 1..old_grid.height - 1 {
            for x in 1..old_grid.width - 1 {
                let cell = self.cell_mut(x, y);

                if let CellType::Wall {
                    ref mut wall_reflect,
                } = cell.cell_type
                {
                    for (i, neighbour) in old_grid.neighbours(x, y).enumerate() {
                        wall_reflect[i] = neighbour.pressure_surplus();

                        wall_reflect[i] += neighbour.gravity_surplus(i);
                    }
                } else if let CellType::Inside { .. } = cell.cell_type {
                    let mut level = cell.level();

                    level = level.saturating_sub(cell.pressure_surplus() * DIRECTIONS as u32);
                    level = level.saturating_sub(cell.total_gravity_surplus());

                    let (mut velocity_x, mut velocity_y) = (0, 0);

                    for (i, neighbour) in old_grid.neighbours(x, y).enumerate() {
                        let incoming_water = if neighbour.is_wall() {
                            neighbour.wall_surplus(i)
                        } else {
                            neighbour.pressure_surplus() + neighbour.gravity_surplus(i)
                        };
                        level = level.saturating_add(incoming_water);

                        if enable_inertia {
                            let incoming_inertia = if neighbour.is_wall() {
                                incoming_water / 8
                            } else {
                                incoming_water
                            };

                            velocity_x += incoming_inertia as i32 * -NEIGHBOUR_OFFSETS[i].1;
                            velocity_y += incoming_inertia as i32 * -NEIGHBOUR_OFFSETS[i].0;
                        }
                    }

                    if enable_gravity && !old_grid.cell(x, y + 1).is_wall() {
                        velocity_y += 32;
                    }

                    let old_cell = old_grid.cell(x, y);
                    cell.velocity_x = (old_cell.velocity_x * 3 + velocity_x) / 4;
                    cell.velocity_y = (old_cell.velocity_y * 3 + velocity_y) / 4;
                    cell.set_level(level);
                }
            }
        }
    }
}

impl WaterCell {
    fn level(&self) -> u32 {
        match self.cell_type {
            CellType::Inside { level } => level,
            CellType::Wall { wall_reflect } => 0,
            CellType::Sea => SEA_LEVEL,
        }
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
        (self.velocity_x as f32, self.velocity_y as f32)
    }

    pub fn fill(&mut self) {
        if let CellType::Inside { ref mut level } = self.cell_type {
            *level += 16 * 1024;
        }
    }

    pub fn is_wall(&self) -> bool {
        matches!(self.cell_type, CellType::Wall { .. })
    }

    pub fn is_sea(&self) -> bool {
        matches!(self.cell_type, CellType::Sea)
    }

    pub fn make_wall(&mut self) {
        self.cell_type = CellType::Wall {
            wall_reflect: [0; DIRECTIONS],
        };
        self.velocity_x = 0;
        self.velocity_y = 0;
    }

    pub fn make_sea(&mut self) {
        self.cell_type = CellType::Sea
    }

    pub fn clear_wall(&mut self) {
        self.cell_type = CellType::Inside { level: 0 };
    }

    fn set_level(&mut self, new_level: u32) {
        match self.cell_type {
            CellType::Inside { ref mut level } => *level = new_level,
            CellType::Wall { wall_reflect } => (),
            CellType::Sea => (),
        }
    }

    fn pressure_surplus(&self) -> u32 {
        let level = self.level();
        if level > 1024 {
            (level - 1024) / DIRECTIONS as u32
        } else {
            0
        }
    }

    fn gravity_surplus(&self, opposite_direction: usize) -> u32 {
        let direction = (opposite_direction + (DIRECTIONS / 2)) % DIRECTIONS;

        // This amount of water can leave the cell
        let level = self.level().min(1024) as i32;

        // This amount of water should leave the cell
        let should_leave = self.velocity_x.abs() + self.velocity_y.abs();

        if should_leave == 0 {
            return 0;
        }

        let will_leave = should_leave.min(level);

        // This is how much will leave in a certain direction
        let mut velocity_x = self.velocity_x;
        let mut velocity_y = self.velocity_y;

        // velocity_x = (velocity_x.abs() as f32).log10() as i32 * velocity_x.signum();
        // velocity_y = (velocity_y.abs() as f32).log10() as i32 * velocity_y.signum();

        let total_velocity = velocity_x.abs() + velocity_y.abs();

        if total_velocity == 0 {
            return 0;
        }

        velocity_x = velocity_x * will_leave / total_velocity;
        velocity_y = velocity_y * will_leave / total_velocity;

        let leftover = will_leave - velocity_x.abs() - velocity_y.abs();
        velocity_y += leftover * velocity_y.signum();

        let surplus_x = (velocity_x * NEIGHBOUR_OFFSETS[direction].1).max(0) as u32;
        let surplus_y = (velocity_y * NEIGHBOUR_OFFSETS[direction].0).max(0) as u32;

        surplus_x + surplus_y
    }

    fn total_gravity_surplus(&self) -> u32 {
        let level = self.level().min(1024) as i32;

        // This amount of water should leave the cell
        let should_leave = self.velocity_x.abs() + self.velocity_y.abs();

        if should_leave == 0 {
            return 0;
        }

        let will_leave = should_leave.min(level);

        will_leave as u32
    }

    fn wall_surplus(&self, opposite_direction: usize) -> u32 {
        // let mut reflected = 0;

        // for near_direction in &[3, 4, 5] {
        //     let direction = (opposite_direction + near_direction) % 8;

        //     //reflected += self.wall_reflect[direction] / self.wall_open_cells[direction].max(1);

        //     // For the direct opposite, also send back the remainder, so it won't be lost.
        //     if *near_direction == 4 {
        //         reflected += self.wall_reflect[direction];
        //         //reflected += self.wall_reflect[direction] % self.wall_open_cells[direction].max(1);
        //     }
        // }

        // reflected

        if let CellType::Wall { ref wall_reflect } = self.cell_type {
            let direction = (opposite_direction + (DIRECTIONS / 2)) % DIRECTIONS;
            wall_reflect[direction]
        } else {
            0
        }
    }
}
