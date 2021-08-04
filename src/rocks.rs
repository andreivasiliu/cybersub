//! The grid which contains the shape of the world in which the submarine
//! exists and collides with. The cells are 16x larger than a single submarine
//! cell.

pub(crate) struct RockGrid {
    cells: Vec<RockCell>,
    width: usize,
    height: usize,
}

#[derive(Default, Clone)]
pub(crate) struct RockCell {
    rock_type: RockType,
}

#[derive(Clone, Copy)]
pub(crate) enum RockType {
    Empty = 0,          // □
    WallFilled = 1,     // ■
    WallLowerLeft = 2,  // ◢
    WallLowerRight = 3, // ◣
    WallUpperLeft = 4,  // ◤
    WallUpperRight = 5, // ◥
}

impl Default for RockType {
    fn default() -> Self {
        RockType::Empty
    }
}

impl RockGrid {
    /// Loads a rock grid from an Image that's twice the size.
    ///
    /// A cell is made from groups of 2x2 pixels, whose colors define a single
    /// cell's type.

    pub fn new(width: usize, height: usize) -> Self {
        let mut cells = Vec::new();
        cells.resize(width * height, RockCell::default());

        RockGrid {
            cells,
            width,
            height,
        }
    }

    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn cell(&self, x: usize, y: usize) -> &RockCell {
        debug_assert!(x < self.width);
        debug_assert!(y < self.height);

        &self.cells[y * self.width + x]
    }

    pub fn cell_mut(&mut self, x: usize, y: usize) -> &mut RockCell {
        debug_assert!(x < self.width);
        debug_assert!(y < self.height);

        &mut self.cells[y * self.width + x]
    }
}

impl RockCell {
    pub fn set_type(&mut self, rock_type: RockType) {
        self.rock_type = rock_type;
    }

    pub fn rock_type(&self) -> RockType {
        self.rock_type
    }

    pub fn is_wall(&self) -> bool {
        !matches!(self.rock_type, RockType::Empty)
    }
}